use crate::infrastructure::mailer::Mailer;
use reqwest::Client;
use sea_orm::DatabaseConnection;

pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod url;

#[derive(Clone)]
pub struct AppState {
    pub database: DatabaseConnection,
    pub mailer: Mailer,
    pub ignored_query_params: Vec<&'static str>,
    pub http_client: Client,
    pub demo: bool,
}
