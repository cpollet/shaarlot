use crate::error_response::ErrorResponse;
use serde::{Deserialize, Serialize};

pub const URL_TAGS: &str = "/api/tags";

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Tag {
    pub name: String,
    pub count: i32,
}

pub type GetTagsResponse = Vec<Tag>;

pub enum GetTagsResult {
    Success(GetTagsResponse),
    ServerError,
    InvalidParameter(String),

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

#[cfg(feature = "frontend")]
impl GetTagsResult {
    pub async fn from(value: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        match value {
            Err(_) => Some(GetTagsResult::BrowserError),
            Ok(response) => match response.status() {
                200 => match response.json::<GetTagsResponse>().await {
                    Err(_) => Some(GetTagsResult::DeserializationError),
                    Ok(payload) => Some(GetTagsResult::Success(payload)),
                },
                400 => match response.json::<ErrorResponse>().await {
                    Err(_) => Some(GetTagsResult::DeserializationError),
                    Ok(payload) => match payload.code() {
                        "INVALID_PARAMETER" => Some(GetTagsResult::InvalidParameter(
                            payload.message().to_owned(),
                        )),
                        _ => Some(GetTagsResult::DeserializationError),
                    },
                },
                500 => Some(GetTagsResult::ServerError),
                _ => {
                    // todo add log
                    None
                }
            },
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for GetTagsResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            GetTagsResult::Success(payload) => axum::Json(payload).into_response(),
            GetTagsResult::ServerError => http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            GetTagsResult::InvalidParameter(message) => (
                http::StatusCode::BAD_REQUEST,
                axum::Json(ErrorResponse::new("INVALID_PARAMETER", &message)),
            )
                .into_response(),
            _ => panic!(),
        }
    }
}
