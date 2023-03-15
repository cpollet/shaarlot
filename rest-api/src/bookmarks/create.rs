use crate::bookmarks::Access;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateBookmarkRequest {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateBookmarkResponse {
    pub id: i32,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub creation_date: DateTime<Utc>,
    pub update_date: Option<DateTime<Utc>>,
    pub user_id: i32,
    pub access: Access,
}

pub enum CreateBookmarkResult {
    Success(CreateBookmarkResponse),
    Forbidden,
    ServerError,

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

#[cfg(feature = "frontend")]
impl CreateBookmarkResult {
    pub async fn from(value: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        match value {
            Err(_) => Some(CreateBookmarkResult::BrowserError),
            Ok(response) => match response.status() {
                201 => match response.json::<CreateBookmarkResponse>().await {
                    Err(_) => Some(CreateBookmarkResult::DeserializationError),
                    Ok(payload) => Some(CreateBookmarkResult::Success(payload)),
                },
                403 => Some(CreateBookmarkResult::Forbidden),
                500 => Some(CreateBookmarkResult::ServerError),
                _ => {
                    // todo add log
                    None
                }
            },
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for CreateBookmarkResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            CreateBookmarkResult::Success(payload) => {
                (http::StatusCode::CREATED, axum::Json(payload)).into_response()
            }
            CreateBookmarkResult::Forbidden => http::StatusCode::FORBIDDEN.into_response(),
            CreateBookmarkResult::ServerError => {
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            _ => panic!(),
        }
    }
}
