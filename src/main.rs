use backend::database;
use backend::rest::router;

#[tokio::main]
async fn main() {
    let database = database::connect().await;

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router(database).into_make_service())
        .await
        .unwrap();
}
