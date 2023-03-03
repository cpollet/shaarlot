use argon2::password_hash::rand_core::{OsRng, RngCore};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use axum_extra::routing::SpaRouter;
use axum_sessions::async_session::MemoryStore;
use axum_sessions::{PersistencePolicy, SessionLayer};
use backend::database::Configuration;
use backend::rest::router;
use backend::{database, AppState};
use sea_orm_migration::MigratorTrait;
use secrecy::{ExposeSecret, SecretVec};
use std::env;
use std::thread::sleep;
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    let filter = filter::Targets::new()
        .with_target("sqlx::postgres::notice", Level::WARN)
        .with_target("tower_http::trace::on_response", Level::DEBUG)
        .with_target("tower_http::trace::on_request", Level::INFO)
        .with_target("tower_http::trace::make_span", Level::TRACE)
        .with_default(Level::INFO);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    log::info!("Starting ...");
    log::info!(
        "Commit      {} ({}); dirty: {}",
        env!("VERGEN_GIT_SHA_SHORT"),
        env!("VERGEN_GIT_BRANCH"),
        env!("GIT_DIRTY")
    );
    log::info!("Build date  {}", env!("SOURCE_TIMESTAMP"));

    let http_host = env::var("HTTP_HOST").unwrap_or("0.0.0.0".to_owned());
    let http_port = env::var("HTTP_PORT").unwrap_or("3000".to_owned());
    let database_host = env::var("DATABASE_HOST").unwrap_or("localhost".to_owned());
    let database_port = env::var("DATABASE_PORT").unwrap_or("5432".to_owned());
    let database_username = env::var("DATABASE_USERNAME").unwrap_or("postgres".to_owned());
    let database_password = env::var("DATABASE_PASSWORD").unwrap_or("password".to_owned());
    let database_name = env::var("DATABASE_NAME").unwrap_or("postgres".to_owned());
    let static_files_path = env::var("ROOT_PATH").unwrap_or("./webroot".to_owned());
    let assets_url = env::var("ASSETS_URL").unwrap_or("/assets".to_owned());

    let database = {
        let config = Configuration {
            host: database_host.clone(),
            port: database_port.clone(),
            username: database_username,
            password: database_password,
            database: database_name.clone(),
        };
        loop {
            log::info!(
                "Connecting to database {} @ {}:{}...",
                database_name,
                database_host,
                database_port
            );
            if let Ok(connection) = database::connect(&config).await {
                break connection;
            }
            sleep(Duration::from_secs(5))
        }
    };
    log::info!("Connected to database");

    migration::Migrator::up(&database, None)
        .await
        .expect("Could not migrate database");

    let mut cookie_secret = [0u8; 64];
    OsRng::default().fill_bytes(&mut cookie_secret);
    let cookie_secret = SecretVec::new(cookie_secret.into());

    let configuration = backend::rest::Configuration {
        cookie_secret,
        session_store: MemoryStore::new(),
    };

    log::info!("Serving {} under {}", static_files_path, assets_url);
    log::info!("Listening on http://{}:{}", http_host, http_port);

    axum::Server::bind(&format!("{}:{}", http_host, http_port).parse().unwrap())
        .serve(
            router(&configuration, AppState { database })
                .merge(
                    Router::new()
                        .merge(SpaRouter::new(&assets_url, static_files_path))
                        .layer(
                            SessionLayer::new(
                                configuration.session_store.clone(),
                                configuration.cookie_secret.expose_secret().as_slice(),
                            )
                            .with_persistence_policy(PersistencePolicy::Always),
                        ),
                )
                .route("/health", get(health))
                .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
                .into_make_service(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn health() -> impl IntoResponse {
    format!(
        "OK; commit:{}; branch:{}; dirty:{}; build_date:{}",
        env!("VERGEN_GIT_SHA_SHORT"),
        env!("VERGEN_GIT_BRANCH"),
        env!("GIT_DIRTY"),
        env!("SOURCE_TIMESTAMP")
    )
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    log::info!("signal received, starting graceful shutdown");
}
