use backend::database;
use backend::rest::router;
use sea_orm_migration::MigratorTrait;

#[tokio::main]
async fn main() {
    let database = database::connect().await;

    migration::Migrator::up(&database, None)
        .await
        .expect("Could not migrate database");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router(database).into_make_service())
        .await
        .unwrap();
}
