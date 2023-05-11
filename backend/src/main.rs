use argon2::password_hash::rand_core::{OsRng, RngCore};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use axum_sessions::async_session::base64;
use axum_sessions::{PersistencePolicy, SameSite, SessionLayer};
use backend::database::Configuration;
use backend::mailer::Mailer;
use backend::rest::api_router;
use backend::sessions::RedisStore;
use backend::{database, AppState};
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::SmtpTransport;
use reqwest::Client;
use sea_orm_migration::MigratorTrait;
use secrecy::{ExposeSecret, SecretVec};
use std::env;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[cfg(not(debug_assertions))]
const STATIC_DIR: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../target/release/wasm");

const IGNORED_GET_PARAMS: &str = include_str!("query-params-registry.txt");

// todo better logging

#[tokio::main]
async fn main() {
    let filter = filter::Targets::new()
        .with_target("sqlx::postgres::notice", Level::WARN)
        // .with_target("sqlx::query", Level::DEBUG)
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
    let public_url =
        env::var("PUBLIC_URL").unwrap_or(format!("http://{}:{}", http_host, http_port));
    let database_host = env::var("DATABASE_HOST").unwrap_or("localhost".to_owned());
    let database_port = env::var("DATABASE_PORT").unwrap_or("5432".to_owned());
    let database_username = env::var("DATABASE_USERNAME").unwrap_or("postgres".to_owned());
    let database_password = env::var("DATABASE_PASSWORD").unwrap_or("password".to_owned());
    let database_name = env::var("DATABASE_NAME").unwrap_or("postgres".to_owned());
    let static_files_path = env::var("ROOT_PATH").unwrap_or(
        if cfg!(debug_assertions) {
            "./webroot"
        } else {
            "@"
        }
        .to_owned(),
    );
    let smtp_host = env::var("SMTP_HOST").unwrap_or("localhost".to_owned());
    let smtp_port = env::var("SMTP_PORT").unwrap_or("25".to_owned());
    let smtp_username = env::var("SMTP_USERNAME").unwrap_or("username".to_owned());
    let smtp_password = env::var("SMTP_PASSWORD").unwrap_or("password".to_owned());
    let smtp_from = env::var("SMTP_FROM").unwrap_or("rbm@localhost".to_owned());
    let redis_host = env::var("REDIS_HOST").unwrap_or("localhost".to_owned());
    let redis_port = env::var("REDIS_PORT").unwrap_or("6379".to_owned());
    let redis_db = env::var("REDIS_DB").unwrap_or("0".to_owned());
    let cookie_secret = env::var("COOKIE_SECRET")
        .map(|v| SecretVec::new(base64::decode(v).unwrap()))
        .unwrap_or_else(|_| {
            let mut cookie_secret = [0u8; 64];
            OsRng::default().fill_bytes(&mut cookie_secret);
            log::info!(
                "Cookie secret not set, using {}",
                base64::encode(cookie_secret)
            );
            SecretVec::new(cookie_secret.into())
        });
    let session_ttl = env::var("SESSION_TTL")
        .map(|s| Duration::from_secs(u64::from_str(&s).unwrap_or(60 * 60 * 24)))
        .unwrap_or(Duration::from_secs(60 * 60 * 24));

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

    let session_store = RedisStore::new(
        redis::Client::open(format!(
            "redis://{}:{}/{}",
            redis_host, redis_port, redis_db
        ))
        .unwrap()
        .get_multiplexed_tokio_connection()
        .await
        .unwrap(),
        session_ttl,
    );

    let configuration = backend::rest::Configuration {
        cookie_secret,
        session_store,
    };

    let creds = Credentials::new(smtp_username.to_owned(), smtp_password.to_owned());
    let smtp_transport = SmtpTransport::relay(&smtp_host)
        .unwrap()
        .port(u16::from_str(&smtp_port).unwrap())
        .credentials(creds)
        .build();

    let mailer = Mailer::new(
        smtp_transport,
        smtp_from.parse::<Mailbox>().unwrap(),
        public_url,
    );

    log::info!("Listening on http://{}:{}", http_host, http_port);

    axum::Server::bind(&format!("{}:{}", http_host, http_port).parse().unwrap())
        .serve(
            api_router(
                &configuration,
                AppState {
                    database,
                    mailer: mailer.clone(),
                    ignored_query_params: IGNORED_GET_PARAMS
                        .split('\n')
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<&str>>(),
                    http_client: Client::builder()
                        .timeout(Duration::from_secs(5))
                        .connect_timeout(Duration::from_secs(5))
                        .build()
                        .expect("Could not initialize HTTP client"),
                },
            )
            .route("/health", get(health))
            .layer(CompressionLayer::new())
            .merge(
                static_file_provider(&static_files_path).layer(
                    SessionLayer::new(
                        configuration.session_store.clone(),
                        configuration.cookie_secret.expose_secret().as_slice(),
                    )
                    .with_session_ttl(Some(session_ttl))
                    .with_persistence_policy(PersistencePolicy::Always)
                    .with_same_site_policy(SameSite::Lax),
                ),
            )
            .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
            .into_make_service(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    mailer.stop().await;
}

fn static_file_provider(static_files_path: &str) -> Router {
    #[cfg(not(debug_assertions))]
    {
        if static_files_path == "@" {
            log::info!("Serving static files from binary");
            return Router::new()
                .route("/", get(static_index_file))
                .route("/*path", get(static_file));
        }
    }

    log::info!("Serving static files from {}", static_files_path);
    Router::new().nest_service(
        "/",
        ServeDir::new(static_files_path)
            .not_found_service(ServeFile::new(format!("{}/index.html", static_files_path))),
    )
}

#[cfg(not(debug_assertions))]
async fn static_index_file(headers: axum::http::header::HeaderMap) -> impl IntoResponse {
    let compression = Compression::from(headers);

    let response_builder = axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(
            axum::http::header::CONTENT_TYPE,
            axum::http::header::HeaderValue::from_str("text/html").unwrap(),
        );

    compression
        .add_header(response_builder)
        .body(axum::body::boxed(axum::body::Full::from(
            STATIC_DIR
                .get_file(compression.append_suffix("index.html"))
                .unwrap()
                .contents(),
        )))
        .unwrap()
}

#[cfg(not(debug_assertions))]
async fn static_file(
    axum::extract::Path(path): axum::extract::Path<String>,
    headers: axum::http::header::HeaderMap,
) -> impl IntoResponse {
    let path = path.trim_start_matches('/');

    let compression = Compression::from(headers);
    let compressed_path = compression.append_suffix(path);
    log::info!("Serving {} as {}", path, compressed_path);

    let (file, mime_type, set_cache_header) = match STATIC_DIR.get_file(compressed_path) {
        None => (
            STATIC_DIR
                .get_file(compression.append_suffix("index.html"))
                .unwrap(),
            mime_guess::from_path("index.html").first_or_text_plain(),
            false,
        ),
        Some(file) => (
            file,
            mime_guess::from_path(path).first_or_text_plain(),
            true,
        ),
    };

    let response_builder = axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(
            axum::http::header::CONTENT_TYPE,
            axum::http::header::HeaderValue::from_str(mime_type.as_ref()).unwrap(),
        );

    let response_builder = if set_cache_header {
        response_builder.header(axum::http::header::CACHE_CONTROL, "private")
    } else {
        response_builder
    };

    compression
        .add_header(response_builder)
        .body(axum::body::boxed(axum::body::Full::from(file.contents())))
        .unwrap()
}

#[cfg(not(debug_assertions))]
enum Compression {
    Br,
    Gzip,
    None,
}

#[cfg(not(debug_assertions))]
impl Compression {
    fn append_suffix(&self, path: &str) -> String {
        match self {
            Compression::Br => format!("{}.br", path),
            Compression::Gzip => format!("{}.gzip", path),
            Compression::None => path.to_string(),
        }
    }

    fn add_header(&self, builder: axum::http::response::Builder) -> axum::http::response::Builder {
        match self {
            Compression::Br => builder.header(axum::http::header::CONTENT_ENCODING, "br"),
            Compression::Gzip => builder.header(axum::http::header::CONTENT_ENCODING, "gzip"),
            Compression::None => builder,
        }
    }
}

#[cfg(not(debug_assertions))]
impl From<axum::http::header::HeaderMap> for Compression {
    fn from(headers: axum::http::header::HeaderMap) -> Self {
        let accept_encoding = headers
            .get(axum::http::header::ACCEPT_ENCODING)
            .map(|h| h.to_str().unwrap_or_default().to_lowercase())
            .unwrap_or_default();
        if accept_encoding.contains("br") {
            return Compression::Br;
        }
        if accept_encoding.contains("gzip") {
            return Compression::Gzip;
        }
        return Compression::None;
    }
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
