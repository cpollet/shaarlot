use chrono::{DateTime, Utc};
use secrecy::{DebugSecret, Secret, SerializableSecret, Zeroize};
use serde::{Deserialize, Serialize};

pub const URL_USERS: &str = "/api/users";
pub const URL_SESSIONS: &str = "/api/sessions";
pub const URL_SESSIONS_CURRENT: &str = "/api/sessions/current";
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

#[derive(Serialize, Deserialize, Clone)]
pub struct RestPassword(pub String);

impl SerializableSecret for RestPassword {}

impl Zeroize for RestPassword {
    fn zeroize(&mut self) {
        self.0.zeroize()
    }
}

impl DebugSecret for RestPassword {}

impl<'t> From<&'t RestPassword> for &'t [u8] {
    fn from(value: &'t RestPassword) -> Self {
        value.0.as_bytes()
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: Secret<RestPassword>,
}

#[derive(Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub username: String,
    pub password: Secret<RestPassword>,
}
