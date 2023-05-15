use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreatePasswordRecoveryRequest {
    pub username_or_email: String,
}

pub enum CreatePasswordRecoveryResult {
    Success,
    NotImplemented,
    ServerError,

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

#[cfg(feature = "frontend")]
impl CreatePasswordRecoveryResult {
    pub async fn from(response: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        if response.is_err() {
            return Some(CreatePasswordRecoveryResult::BrowserError);
        }
        let response = response.unwrap();
        match response.status() {
            201 => Some(CreatePasswordRecoveryResult::Success),
            500 => Some(CreatePasswordRecoveryResult::ServerError),
            501 => Some(CreatePasswordRecoveryResult::NotImplemented),
            _ => {
                // todo add log
                None
            }
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for CreatePasswordRecoveryResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            CreatePasswordRecoveryResult::Success => http::StatusCode::CREATED.into_response(),
            CreatePasswordRecoveryResult::NotImplemented => {
                http::StatusCode::NOT_IMPLEMENTED.into_response()
            }
            CreatePasswordRecoveryResult::ServerError => {
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            _ => panic!(),
        }
    }
}
