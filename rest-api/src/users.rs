use crate::RestPassword;
use secrecy::Secret;
use serde::{Deserialize, Serialize};

pub const URL_USERS: &str = "/api/users";

#[derive(Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub username: String,
    pub password: Secret<RestPassword>,
    pub password_verif: Secret<RestPassword>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserResponse {
    pub id: i32,
    pub username: String,
}

pub enum CreateUserResult {
    Success(CreateUserResponse),
    InvalidPassword,
    ServerError,

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

#[cfg(feature = "frontend")]
impl CreateUserResult {
    pub async fn from(response: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        if let Err(_) = response {
            return Some(CreateUserResult::BrowserError);
        }
        let response = response.unwrap();
        match response.status() {
            201 => {
                let payload = response.json::<CreateUserResponse>().await;
                if let Err(_) = payload {
                    return Some(CreateUserResult::DeserializationError);
                }
                Some(CreateUserResult::Success(payload.unwrap()))
            }
            400 => Some(CreateUserResult::InvalidPassword),
            500 => Some(CreateUserResult::ServerError),
            _ => {
                // todo add log?
                None
            }
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for CreateUserResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            CreateUserResult::Success(payload) => {
                (http::StatusCode::CREATED, axum::Json(payload)).into_response()
            }
            CreateUserResult::InvalidPassword => http::StatusCode::BAD_REQUEST.into_response(),
            CreateUserResult::ServerError => {
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            _ => panic!()
        }
    }
}
