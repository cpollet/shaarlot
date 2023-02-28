use chrono::{DateTime, Local};
use rest_api::bookmarks::create::CreateBookmarkRequest;
use rest_api::bookmarks::get_one::GetBookmarkResponse;
use rest_api::bookmarks::update::UpdateBookmarkRequest;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Bookmark {
    pub id: i32,
    pub url: AttrValue,
    pub title: Option<AttrValue>,
    pub description: Option<AttrValue>,
    pub creation_date: DateTime<Local>,
    pub update_date: Option<DateTime<Local>>,
}

impl Default for Bookmark {
    fn default() -> Self {
        Self {
            id: 0,
            url: AttrValue::from(""),
            title: None,
            description: None,
            creation_date: DateTime::default(),
            update_date: None,
        }
    }
}

impl From<GetBookmarkResponse> for Bookmark {
    fn from(value: GetBookmarkResponse) -> Self {
        Bookmark {
            id: value.id,
            url: AttrValue::from(value.url),
            title: value.title.map(|v| AttrValue::from(v)),
            description: value.description.map(|v| AttrValue::from(v)),
            creation_date: DateTime::from(value.creation_date),
            update_date: value.update_date.map(|d| DateTime::from(d)),
        }
    }
}

impl From<&Bookmark> for UpdateBookmarkRequest {
    fn from(bookmark: &Bookmark) -> Self {
        UpdateBookmarkRequest {
            url: bookmark.url.to_string(),
            title: bookmark.title.as_ref().map(|v| v.to_string()),
            description: bookmark.description.as_ref().map(|v| v.to_string()),
        }
    }
}

impl From<&Bookmark> for CreateBookmarkRequest {
    fn from(bookmark: &Bookmark) -> Self {
        Self {
            url: bookmark.url.to_string(),
            title: bookmark.title.as_ref().map(|v| v.to_string()),
            description: bookmark.description.as_ref().map(|v| v.to_string()),
        }
    }
}
