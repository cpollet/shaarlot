use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const URL_BOOKMARKS: &str = "/api/bookmarks";
pub const URL_BOOKMARK: &str = "/api/bookmarks/:id";
pub const URL_BOOKMARK_QRCODE: &str = "/api/bookmarks/:id/qrcode";
pub const URL_URLS: &str = "/api/urls/:url";

#[derive(Serialize, Deserialize)]
pub struct BookmarkResponse {
    pub id: i32,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub creation_date: DateTime<Utc>,
    pub update_date: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateBookmarkRequest {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateBookmarkRequest {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UrlResponse {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}
