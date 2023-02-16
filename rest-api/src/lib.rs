use serde::{Deserialize, Serialize};

pub const URL_BOOKMARKS: &str = "/api/bookmarks";
pub const URL_BOOKMARK: &str = "/api/bookmarks/:id";
pub const URL_BOOKMARK_QRCODE: &str = "/api/bookmarks/:id/qrcode";

#[derive(Serialize, Deserialize)]
pub struct BookmarkResponse {
    pub id: i32,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct CreateBookmarkRequest {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateBookmarkRequest {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}
