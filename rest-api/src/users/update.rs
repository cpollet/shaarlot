use crate::error_response::ErrorResponse;
use crate::RestPassword;
use secrecy::Secret;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub email: String,
    pub current_password: Secret<RestPassword>,
    pub new_password: Option<Secret<RestPassword>>,
    pub new_password_verif: Option<Secret<RestPassword>>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateUserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
}

pub enum UpdateUserResult {
    Success(UpdateUserResponse),
    InvalidCurrentPassword,
    InvalidNewPassword,
    InvalidEmailAddress,
    /// returned when user cannot be found from session or when current user does not have access to
    /// updated user
    Forbidden,
    ServerError,

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

const ERR_INVALID_NEW_PASSWORD: &str = "INVALID_NEW_PASSWORD";

const ERR_INVALID_NEW_EMAIL_ADDRESS: &str = "INVALID_NEW_EMAIL_ADDRESS";

#[cfg(feature = "frontend")]
impl UpdateUserResult {
    pub async fn from(response: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        if response.is_err() {
            return Some(UpdateUserResult::BrowserError);
        }
        let response = response.unwrap();
        match response.status() {
            200 => {
                let payload = response.json::<UpdateUserResponse>().await;
                if payload.is_err() {
                    return Some(UpdateUserResult::DeserializationError);
                }
                Some(UpdateUserResult::Success(payload.unwrap()))
            }
            400 => match response.json::<ErrorResponse>().await {
                Err(_) => Some(UpdateUserResult::DeserializationError),
                Ok(payload) => match payload.code() {
                    ERR_INVALID_NEW_PASSWORD => Some(UpdateUserResult::InvalidNewPassword),
                    ERR_INVALID_NEW_EMAIL_ADDRESS => Some(UpdateUserResult::InvalidEmailAddress),
                    _ => Some(UpdateUserResult::DeserializationError),
                },
            },
            401 => Some(UpdateUserResult::InvalidCurrentPassword),
            403 => Some(UpdateUserResult::Forbidden),
            500 => Some(UpdateUserResult::ServerError),
            _ => {
                // todo add log?
                None
            }
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for UpdateUserResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            UpdateUserResult::Success(payload) => {
                (http::StatusCode::OK, axum::Json(payload)).into_response()
            }
            UpdateUserResult::InvalidCurrentPassword => {
                http::StatusCode::UNAUTHORIZED.into_response()
            }
            UpdateUserResult::InvalidNewPassword => (
                http::StatusCode::BAD_REQUEST,
                axum::Json(ErrorResponse::new(
                    ERR_INVALID_NEW_PASSWORD,
                    "New password is not acceptable",
                )),
            )
                .into_response(),
            UpdateUserResult::InvalidEmailAddress => (
                http::StatusCode::BAD_REQUEST,
                axum::Json(ErrorResponse::new(
                    ERR_INVALID_NEW_EMAIL_ADDRESS,
                    "New email address is invalid",
                )),
            )
                .into_response(),
            UpdateUserResult::Forbidden => http::StatusCode::FORBIDDEN.into_response(),
            UpdateUserResult::ServerError => {
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            _ => panic!(),
        }
    }
}
