use serde::{Deserialize, Serialize};

pub const URL_APPLICATION: &str = "/api/application";

pub enum GetApplicationResult {
    Success(GetApplicationResponse),

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

#[derive(Serialize, Deserialize)]
pub struct GetApplicationResponse {
    pub commit: String,
    pub build_date: String,
}

#[cfg(feature = "frontend")]
impl GetApplicationResult {
    pub async fn from(value: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        match value {
            Err(_) => Some(GetApplicationResult::BrowserError),
            Ok(response) => match response.status() {
                200 => match response.json::<GetApplicationResponse>().await {
                    Err(_) => Some(GetApplicationResult::DeserializationError),
                    Ok(payload) => Some(GetApplicationResult::Success(payload)),
                },
                _ => {
                    // todo add log
                    None
                }
            },
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for GetApplicationResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            GetApplicationResult::Success(payload) => {
                (http::StatusCode::OK, axum::Json(payload)).into_response()
            }
            _ => panic!(),
        }
    }
}
