use crate::RestPassword;
use secrecy::Secret;
use serde::{Deserialize, Serialize};

pub const URL_SESSIONS: &str = "/api/sessions";
pub const URL_SESSIONS_CURRENT: &str = "/api/sessions/current";

#[derive(Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub username: String,
    pub password: Secret<RestPassword>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub username: String,
}

pub enum CreateSessionResult {
    Success(CreateSessionResponse),
    InvalidCredentials,
    ServerError,

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

#[cfg(feature = "frontend")]
impl CreateSessionResult {
    pub async fn from(response: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        if response.is_err() {
            return Some(CreateSessionResult::BrowserError);
        }
        let response = response.unwrap();
        match response.status() {
            200 => {
                let payload = response.json::<CreateSessionResponse>().await;
                if payload.is_err() {
                    return Some(CreateSessionResult::DeserializationError);
                }
                Some(CreateSessionResult::Success(payload.unwrap()))
            }
            401 => Some(CreateSessionResult::InvalidCredentials),
            500 => Some(CreateSessionResult::ServerError),
            _ => {
                // todo add log
                None
            }
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for CreateSessionResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            CreateSessionResult::Success(payload) => axum::Json(payload).into_response(),
            CreateSessionResult::InvalidCredentials => {
                http::StatusCode::UNAUTHORIZED.into_response()
            }
            CreateSessionResult::ServerError => {
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            _ => panic!(),
        }
    }
}
