use crate::bookmarks::get_one::GetBookmarkResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateBookmarkRequest {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub private: bool,
}

pub type CreateBookmarkResponse = GetBookmarkResponse;

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
