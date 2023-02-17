use axum_extra::routing::SpaRouter;
use backend::database::Configuration;
use backend::rest::router;
use backend::{database, AppState};
use sea_orm_migration::MigratorTrait;
use std::env;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    let http_host = env::var("HTTP_HOST").unwrap_or("0.0.0.0".to_owned());
    let http_port = env::var("HTTP_PORT").unwrap_or("3000".to_owned());
    let database_host = env::var("DATABASE_HOST").unwrap_or("localhost".to_owned());
    let database_port = env::var("DATABASE_PORT").unwrap_or("5432".to_owned());
    let database_username = env::var("DATABASE_USERNAME").unwrap_or("postgres".to_owned());
    let database_password = env::var("DATABASE_PASSWORD").unwrap_or("password".to_owned());
    let database_name = env::var("DATABASE_NAME").unwrap_or("postgres".to_owned());
    let static_files_path = env::var("ROOT_PATH").unwrap_or("./dist".to_owned());

    let database = {
        let config = Configuration {
            host: database_host,
            port: database_port,
            username: database_username,
            password: database_password,
            database: database_name,
        };
        database::connect(&config).await
    };

    migration::Migrator::up(&database, None)
        .await
        .expect("Could not migrate database");

    env::set_var("RUST_LOG", "debug,hyper=info,mio=info");
    tracing_subscriber::fmt::init();

    log::info!("listening on http://{}:{}", http_host, http_port);

    axum::Server::bind(&format!("{}:{}", http_host, http_port).parse().unwrap())
        .serve(
            router(AppState { database })
                .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
                .merge(SpaRouter::new("/", static_files_path))
                .into_make_service(),
        )
        .await
        .unwrap();
}
