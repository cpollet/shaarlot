use crate::error_response::ErrorResponse;
#[cfg(feature = "frontend")]
use std::str::FromStr;

pub enum DeleteBookmarkResult {
    Success,
    Forbidden,
    NotFound(i32, String),
    ServerError,

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

#[cfg(feature = "frontend")]
impl DeleteBookmarkResult {
    pub async fn from(value: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        match value {
            Err(_) => Some(DeleteBookmarkResult::BrowserError),
            Ok(response) => match response.status() {
                204 => Some(DeleteBookmarkResult::Success),
                403 => Some(DeleteBookmarkResult::Forbidden),
                404 => match response.json::<ErrorResponse>().await {
                    Err(_) => Some(DeleteBookmarkResult::DeserializationError),
                    Ok(payload) => match payload.data("id").and_then(|id| i32::from_str(id).ok()) {
                        None => Some(DeleteBookmarkResult::DeserializationError),
                        Some(id) => Some(DeleteBookmarkResult::NotFound(
                            id,
                            payload.message().to_string(),
                        )),
                    },
                },
                500 => Some(DeleteBookmarkResult::ServerError),
                _ => {
                    // todo add log
                    None
                }
            },
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for DeleteBookmarkResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            DeleteBookmarkResult::Success => http::StatusCode::NO_CONTENT.into_response(),
            DeleteBookmarkResult::Forbidden => http::StatusCode::FORBIDDEN.into_response(),
            DeleteBookmarkResult::NotFound(id, message) => (
                http::StatusCode::NOT_FOUND,
                axum::Json(
                    ErrorResponse::new("NOT_FOUND", &message).with_data("id", &format!("{}", id)),
                ),
            )
                .into_response(),
            DeleteBookmarkResult::ServerError => {
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
