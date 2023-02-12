use crate::rest::router;

mod rest;

#[tokio::main]
async fn main() {
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router().into_make_service())
        .await
        .unwrap();
}
