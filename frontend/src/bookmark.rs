use yew::prelude::*;
use rest_api::BookmarkResponse;


#[derive(Properties, Clone, PartialEq)]
pub struct BookmarkProps {
    pub url: AttrValue,
    pub title: Option<AttrValue>,
    pub description: Option<AttrValue>,
}

impl From<BookmarkResponse> for BookmarkProps {
    fn from(value: BookmarkResponse) -> Self {
        BookmarkProps {
            url: AttrValue::from(value.url),
            title: value.title.map(|v|AttrValue::from(v)),
            description: value.description.map(|v|AttrValue::from(v)),
        }
    }
}

#[function_component(Bookmark)]
pub fn bookmark(props: &BookmarkProps) -> Html {
    html! {
        <li>
            <a href={props.url.clone()}>
                {props.title.clone().unwrap_or_else(|| props.url.clone())}
            </a>
        </li>
    }
}