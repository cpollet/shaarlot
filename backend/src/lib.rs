use moka::future::Cache;
use oauth2::basic::{BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse, BasicTokenResponse, BasicTokenType};
use oauth2::{Client, StandardRevocableToken};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

pub mod database;
pub mod rest;

#[derive(Clone)]
pub struct AppState {
    pub database: DatabaseConnection,
    pub oauth_client: Client<BasicErrorResponse, BasicTokenResponse, BasicTokenType, BasicTokenIntrospectionResponse, StandardRevocableToken, BasicRevocationErrorResponse>,
    pub cache: Cache<String, String>,
}
