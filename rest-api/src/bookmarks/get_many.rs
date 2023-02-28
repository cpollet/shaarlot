use crate::bookmarks::get_one::GetBookmarkResponse;
use crate::error_response::ErrorResponse;

pub type GetBookmarksResponse = Vec<GetBookmarkResponse>;

pub enum GetBookmarksResult {
    Success(GetBookmarksResponse),
    InvalidParameter(String),
    ServerError,

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

#[cfg(feature = "frontend")]
impl GetBookmarksResult {
    pub async fn from(response: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        if let Err(_) = response {
            return Some(GetBookmarksResult::BrowserError);
        }
        let response = response.unwrap();
        match response.status() {
            200 => match response.json::<GetBookmarksResponse>().await {
                Err(_) => Some(GetBookmarksResult::DeserializationError),
                Ok(payload) => Some(GetBookmarksResult::Success(payload)),
            },
            401 => match response.json::<ErrorResponse>().await {
                Err(_) => Some(GetBookmarksResult::DeserializationError),
                Ok(payload) => match payload.code() {
                    "INVALID_PARAMETER" => Some(GetBookmarksResult::InvalidParameter(
                        payload.message().to_owned(),
                    )),
                    _ => Some(GetBookmarksResult::DeserializationError),
                },
            },
            500 => Some(GetBookmarksResult::ServerError),
            _ => {
                // todo add log
                None
            }
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for GetBookmarksResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            GetBookmarksResult::Success(payload) => axum::Json(payload).into_response(),
            GetBookmarksResult::InvalidParameter(message) => (
                http::StatusCode::BAD_REQUEST,
                axum::Json(ErrorResponse::new("INVALID_PARAMETER", &message)),
            )
                .into_response(),
            GetBookmarksResult::ServerError => {
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
