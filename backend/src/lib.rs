use sea_orm::DatabaseConnection;

pub mod database;
pub mod rest;
mod session;

#[derive(Clone)]
pub struct AppState {
    pub database: DatabaseConnection,
}
