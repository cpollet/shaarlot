use serde::{Deserialize, Serialize};

pub mod create;
pub mod delete;
pub mod get_many;
pub mod get_one;
pub mod update;

pub const URL_BOOKMARKS: &str = "/api/bookmarks";
pub const URL_BOOKMARK: &str = "/api/bookmarks/:id";
pub const URL_BOOKMARK_QRCODE: &str = "/api/bookmarks/:id/qrcode";
// todo merge with URL_BOOKMARKS
pub const URL_BOOKMARKS_STATS: &str = "/api/bookmarks-stats";

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Access {
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "write")]
    Write,
}

impl Default for Access {
    fn default() -> Self {
        Self::Read
    }
}

pub enum GetBookmarksStatsResult {
    Success(GetBookmarksStatsResponse),
    ServerError,

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct GetBookmarksStatsResponse {
    pub count_total: u64,
    pub count_private: u64,
}

#[cfg(feature = "frontend")]
impl GetBookmarksStatsResult {
    pub async fn from(value: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        match value {
            Err(_) => Some(GetBookmarksStatsResult::BrowserError),
            Ok(response) => match response.status() {
                200 => match response.json::<GetBookmarksStatsResponse>().await {
                    Err(_) => Some(GetBookmarksStatsResult::DeserializationError),
                    Ok(payload) => Some(GetBookmarksStatsResult::Success(payload)),
                },
                500 => Some(GetBookmarksStatsResult::ServerError),
                _ => {
                    // todo add log
                    None
                }
            },
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for GetBookmarksStatsResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            GetBookmarksStatsResult::Success(payload) => {
                (http::StatusCode::OK, axum::Json(payload)).into_response()
            }
            GetBookmarksStatsResult::ServerError => {
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            _ => panic!(),
        }
    }
}
