use crate::mailer::Mailer;
use sea_orm::DatabaseConnection;

pub mod database;
pub mod mailer;
pub mod rest;
pub mod sessions;

#[derive(Clone)]
pub struct AppState {
    pub database: DatabaseConnection,
    pub mailer: Mailer,
}
