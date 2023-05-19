pub mod accounts;
pub mod bookmarks;
pub mod bookmarks_tags;
pub mod password_recoveries;
pub mod pins;
pub mod tags;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

pub struct Configuration {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

pub async fn connect(configuration: &Configuration) -> Result<DatabaseConnection, DbErr> {
    let mut connect_options = ConnectOptions::new(format!(
        "postgres://{}:{}@{}:{}/{}",
        configuration.username,
        configuration.password,
        configuration.host,
        configuration.port,
        configuration.database,
    ));
    connect_options.sqlx_logging_level(log::LevelFilter::Debug);

    Database::connect(connect_options).await
}
