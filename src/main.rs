use axum_extra::routing::SpaRouter;
use backend::database;
use backend::rest::router;
use sea_orm_migration::MigratorTrait;
use std::env;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", format!("debug,hyper=info,mio=info"));

    let database_host = env::var("DATABASE_HOST").unwrap_or("localhost".to_owned());

    let database = database::connect(&database_host).await;

    migration::Migrator::up(&database, None)
        .await
        .expect("Could not migrate database");

    tracing_subscriber::fmt::init();

    log::info!("listening on http://0.0.0.0:3000");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(
            router(database)
                .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
                .merge(SpaRouter::new("/", "./dist"))
                .into_make_service(),
        )
        .await
        .unwrap();
}
