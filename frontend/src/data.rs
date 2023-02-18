use rest_api::BookmarkResponse;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Bookmark {
    pub id: i32,
    pub url: AttrValue,
    pub title: Option<AttrValue>,
    pub description: Option<AttrValue>,
}

impl From<BookmarkResponse> for Bookmark {
    fn from(value: BookmarkResponse) -> Self {
        Bookmark {
            id: value.id,
            url: AttrValue::from(value.url),
            title: value.title.map(|v| AttrValue::from(v)),
            description: value.description.map(|v| AttrValue::from(v)),
        }
    }
}
