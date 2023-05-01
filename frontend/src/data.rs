use chrono::{DateTime, Local};
use rest_api::bookmarks::create::CreateBookmarkRequest;
use rest_api::bookmarks::get_one::GetBookmarkResponse;
use rest_api::bookmarks::update::UpdateBookmarkRequest;
use rest_api::bookmarks::Access;
use yew::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Bookmark {
    pub id: i32,
    pub url: AttrValue,
    pub title: Option<AttrValue>,
    pub description: Option<AttrValue>,
    pub tags: Vec<AttrValue>,
    pub creation_date: DateTime<Local>,
    pub update_date: Option<DateTime<Local>>,
    pub access: Access,
    pub private: bool,
}

impl Default for Bookmark {
    fn default() -> Self {
        Self {
            id: 0,
            url: AttrValue::from(""),
            title: None,
            description: None,
            tags: Vec::default(),
            creation_date: DateTime::default(),
            update_date: None,
            access: Access::Read,
            private: true,
        }
    }
}

impl From<GetBookmarkResponse> for Bookmark {
    fn from(value: GetBookmarkResponse) -> Self {
        Bookmark {
            id: value.id,
            url: AttrValue::from(value.url),
            title: value.title.map(AttrValue::from),
            description: value.description.map(AttrValue::from),
            tags: value
                .tags
                .into_iter()
                .map(AttrValue::from)
                .collect::<Vec<AttrValue>>(),
            creation_date: DateTime::from(value.creation_date),
            update_date: value.update_date.map(DateTime::from),
            access: value.access,
            private: value.private,
        }
    }
}

impl From<&Bookmark> for UpdateBookmarkRequest {
    fn from(bookmark: &Bookmark) -> Self {
        UpdateBookmarkRequest {
            url: bookmark.url.to_string(),
            title: bookmark.title.as_ref().map(|v| v.to_string()),
            description: bookmark.description.as_ref().map(|v| v.to_string()),
            tags: bookmark
                .tags
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
            private: bookmark.private,
        }
    }
}

impl From<&Bookmark> for CreateBookmarkRequest {
    fn from(bookmark: &Bookmark) -> Self {
        Self {
            url: bookmark.url.to_string(),
            title: bookmark.title.as_ref().map(|v| v.to_string()),
            description: bookmark.description.as_ref().map(|v| v.to_string()),
            tags: Some(bookmark
                .tags
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()),
            private: Some(bookmark.private),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Tag {
    pub name: AttrValue,
    pub count: i32,
}

impl From<rest_api::tags::Tag> for Tag {
    fn from(value: rest_api::tags::Tag) -> Self {
        Self {
            name: AttrValue::from(value.name),
            count: value.count,
        }
    }
}

pub type Tags = Vec<Tag>;
