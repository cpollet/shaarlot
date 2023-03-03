use serde::{Deserialize, Serialize};

pub const URL_URLS: &str = "/api/urls/:url";

#[derive(Serialize, Deserialize)]
pub struct GetUrlResponse {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

pub enum GetUrlResult {
    Success(GetUrlResponse),
    Forbidden,
    ServerError,

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
                403 => Some(GetUrlResult::Forbidden),
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
            GetUrlResult::Forbidden => http::StatusCode::FORBIDDEN.into_response(),
            GetUrlResult::ServerError => http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            _ => panic!()
        }
    }
}
