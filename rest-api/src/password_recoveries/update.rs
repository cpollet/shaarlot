use crate::error_response::ErrorResponse;
use crate::{RestPassword, RestToken};
use secrecy::Secret;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UpdatePasswordRecoveryRequest {
    pub id: String,
    pub token: Secret<RestToken>,
    pub password: Secret<RestPassword>,
    pub password_verif: Secret<RestPassword>,
}

pub enum UpdatePasswordRecoveryResult {
    Success,
    InvalidToken,
    InvalidPassword,
    ServerError,

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

const ERR_INVALID_NEW_PASSWORD: &str = "INVALID_NEW_PASSWORD";
const ERR_INVALID_TOKEN: &str = "ERR_INVALID_TOKEN";

#[cfg(feature = "frontend")]
impl UpdatePasswordRecoveryResult {
    pub async fn from(response: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        if response.is_err() {
            return Some(UpdatePasswordRecoveryResult::BrowserError);
        }
        let response = response.unwrap();
        match response.status() {
            200 => Some(UpdatePasswordRecoveryResult::Success),
            400 => match response.json::<ErrorResponse>().await {
                Err(_) => Some(UpdatePasswordRecoveryResult::DeserializationError),
                Ok(payload) => match payload.code() {
                    ERR_INVALID_NEW_PASSWORD => Some(UpdatePasswordRecoveryResult::InvalidPassword),
                    ERR_INVALID_TOKEN => Some(UpdatePasswordRecoveryResult::InvalidToken),
                    _ => Some(UpdatePasswordRecoveryResult::DeserializationError),
                },
            },
            500 => Some(UpdatePasswordRecoveryResult::ServerError),
            _ => {
                // todo add log?
                None
            }
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for UpdatePasswordRecoveryResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            UpdatePasswordRecoveryResult::Success => http::StatusCode::OK.into_response(),
            UpdatePasswordRecoveryResult::InvalidPassword => (
                http::StatusCode::BAD_REQUEST,
                axum::Json(ErrorResponse::new(
                    ERR_INVALID_NEW_PASSWORD,
                    "New password is not acceptable",
                )),
            )
                .into_response(),
            UpdatePasswordRecoveryResult::InvalidToken => (
                http::StatusCode::BAD_REQUEST,
                axum::Json(ErrorResponse::new(
                    ERR_INVALID_TOKEN,
                    "The provided token is invalid",
                )),
            )
                .into_response(),
            UpdatePasswordRecoveryResult::ServerError => {
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            _ => panic!(),
        }
    }
}
