use serde::{Deserialize, Serialize};

pub mod create;
pub mod delete;
pub mod get_many;
pub mod get_one;
pub mod update;

pub const URL_BOOKMARKS: &str = "/api/bookmarks";
pub const URL_BOOKMARK: &str = "/api/bookmarks/:id";
pub const URL_BOOKMARK_QRCODE: &str = "/api/bookmarks/:id/qrcode";

#[derive(Serialize, Deserialize, Clone, PartialEq)]
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
