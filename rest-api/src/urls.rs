use serde::{Deserialize, Serialize};

pub const URL_URLS: &str = "/api/urls/:url";

#[derive(Serialize, Deserialize)]
pub struct GetUrlResponse {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetUrlConflictResponse {
    pub id: i32,
}

pub enum GetUrlResult {
    Success(GetUrlResponse),
    Conflict(GetUrlConflictResponse),
    Forbidden,
    ServerError,
    InvalidUrl,

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

#[cfg(feature = "frontend")]
impl GetUrlResult {
    pub async fn from(value: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        match value {
            Err(_) => Some(GetUrlResult::BrowserError),
            Ok(response) => match response.status() {
                200 => match response.json::<GetUrlResponse>().await {
                    Err(_) => Some(GetUrlResult::DeserializationError),
                    Ok(payload) => Some(GetUrlResult::Success(payload)),
                },
                400 => Some(GetUrlResult::InvalidUrl),
                403 => Some(GetUrlResult::Forbidden),
                409 => match response.json::<GetUrlConflictResponse>().await {
                    Err(_) => Some(GetUrlResult::DeserializationError),
                    Ok(payload) => Some(GetUrlResult::Conflict(payload)),
                },
                500 => Some(GetUrlResult::ServerError),
                _ => {
                    // todo add log
                    None
                }
            },
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for GetUrlResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            GetUrlResult::Success(payload) => axum::Json(payload).into_response(),
            GetUrlResult::Conflict(payload) => {
                (http::StatusCode::CONFLICT, axum::Json(payload)).into_response()
            }
            GetUrlResult::InvalidUrl => http::StatusCode::BAD_REQUEST.into_response(),
            GetUrlResult::Forbidden => http::StatusCode::FORBIDDEN.into_response(),
            GetUrlResult::ServerError => http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            _ => panic!(),
        }
    }
}
