use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetUserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
}

pub enum GetUserResult {
    Success(GetUserResponse),
    Forbidden,
    ServerError,

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

#[cfg(feature = "frontend")]
impl GetUserResult {
    pub async fn from(response: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        if response.is_err() {
            return Some(GetUserResult::BrowserError);
        }
        let response = response.unwrap();
        match response.status() {
            200 => {
                let payload = response.json::<GetUserResponse>().await;
                if payload.is_err() {
                    return Some(GetUserResult::DeserializationError);
                }
                Some(GetUserResult::Success(payload.unwrap()))
            }
            403 => Some(GetUserResult::Forbidden),
            500 => Some(GetUserResult::ServerError),
            _ => {
                // todo add log?
                None
            }
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for GetUserResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            GetUserResult::Success(payload) => {
                (http::StatusCode::OK, axum::Json(payload)).into_response()
            }
            GetUserResult::Forbidden => http::StatusCode::FORBIDDEN.into_response(),
            GetUserResult::ServerError => http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            _ => panic!(),
        }
    }
}
