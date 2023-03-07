pub const URL_EMAIL: &str = "/api/emails/:uuid";

pub enum ValidateEmailResult {
    Success,
    InvalidToken,
    ServerError,

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

#[cfg(feature = "frontend")]
impl ValidateEmailResult {
    pub async fn from(response: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        if response.is_err() {
            return Some(ValidateEmailResult::BrowserError);
        }
        let response = response.unwrap();
        match response.status() {
            204 => Some(ValidateEmailResult::Success),
            404 => Some(ValidateEmailResult::InvalidToken),
            500 => Some(ValidateEmailResult::ServerError),
            _ => {
                // todo add log?
                None
            }
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for ValidateEmailResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            ValidateEmailResult::Success => http::StatusCode::NO_CONTENT.into_response(),
            ValidateEmailResult::InvalidToken => http::StatusCode::NOT_FOUND.into_response(),
            ValidateEmailResult::ServerError => {
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            _ => panic!(),
        }
    }
}
