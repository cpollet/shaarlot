pub mod bookmarks;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub async fn connect() -> DatabaseConnection {
    let connect_options =
        ConnectOptions::new("postgres://postgres:password@localhost:5432/postgres".to_owned());
    Database::connect(connect_options)
        .await
        .expect("Cannot open connection to database")
}
