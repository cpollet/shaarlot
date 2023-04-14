use crate::bookmarks::get_one::GetBookmarkResponse;
use crate::error_response::ErrorResponse;
use serde::{Deserialize, Serialize};
#[cfg(feature = "frontend")]
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
pub struct UpdateBookmarkRequest {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub private: bool,
}

pub type UpdateBookmarkResponse = GetBookmarkResponse;

pub enum UpdateBookmarkResult {
    Success(UpdateBookmarkResponse),
    Forbidden,
    NotFound(i32, String),
    ServerError,

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

#[cfg(feature = "frontend")]
impl UpdateBookmarkResult {
    pub async fn from(value: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        match value {
            Err(_) => Some(UpdateBookmarkResult::BrowserError),
            Ok(response) => match response.status() {
                200 => match response.json::<UpdateBookmarkResponse>().await {
                    Err(_) => Some(UpdateBookmarkResult::DeserializationError),
                    Ok(payload) => Some(UpdateBookmarkResult::Success(payload)),
                },
                403 => Some(UpdateBookmarkResult::Forbidden),
                404 => match response.json::<ErrorResponse>().await {
                    Err(_) => Some(UpdateBookmarkResult::DeserializationError),
                    Ok(payload) => match payload.data("id").and_then(|id| i32::from_str(id).ok()) {
                        None => Some(UpdateBookmarkResult::DeserializationError),
                        Some(id) => Some(UpdateBookmarkResult::NotFound(
                            id,
                            payload.message().to_string(),
                        )),
                    },
                },
                500 => Some(UpdateBookmarkResult::ServerError),
                _ => {
                    // todo add log
                    None
                }
            },
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for UpdateBookmarkResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            UpdateBookmarkResult::Success(payload) => axum::Json(payload).into_response(),
            UpdateBookmarkResult::Forbidden => http::StatusCode::FORBIDDEN.into_response(),
            UpdateBookmarkResult::NotFound(id, message) => (
                http::StatusCode::NOT_FOUND,
                axum::Json(
                    ErrorResponse::new("NOT_FOUND", &message).with_data("id", &format!("{}", id)),
                ),
            )
                .into_response(),
            UpdateBookmarkResult::ServerError => {
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            _ => panic!(),
        }
    }
}
