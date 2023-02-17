pub mod bookmarks;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub struct Configuration {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

pub async fn connect(configuration: &Configuration) -> DatabaseConnection {
    let connect_options = ConnectOptions::new(format!(
        "postgres://{}:{}@{}:{}/{}",
        configuration.username,
        configuration.password,
        configuration.host,
        configuration.port,
        configuration.database,
    ));
    Database::connect(connect_options)
        .await
        .expect("Cannot open connection to database")
}
