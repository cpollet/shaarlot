use crate::mailer::Mailer;
use reqwest::Client;
use sea_orm::DatabaseConnection;

pub mod database;
pub mod mailer;
pub mod rest;
pub mod sessions;
pub mod url;

#[derive(Clone)]
pub struct AppState {
    pub database: DatabaseConnection,
    pub mailer: Mailer,
    pub ignored_query_params: Vec<&'static str>,
    pub http_client: Client,
}
