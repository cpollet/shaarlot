pub mod bookmarks;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub async fn connect(database_host: &str) -> DatabaseConnection {
    let connect_options = ConnectOptions::new(format!(
        "postgres://postgres:password@{}:5432/postgres",
        database_host
    ));
    Database::connect(connect_options)
        .await
        .expect("Cannot open connection to database")
}
