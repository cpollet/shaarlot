use sea_orm::DatabaseConnection;

pub mod database;
pub mod rest;

#[derive(Clone)]
pub struct AppState {
    pub database: DatabaseConnection,
}
