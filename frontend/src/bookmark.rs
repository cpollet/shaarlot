use crate::qr_code::QrCode;
use rest_api::BookmarkResponse;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct BookmarkProps {
    pub id: i32,
    pub url: AttrValue,
    pub title: Option<AttrValue>,
    pub description: Option<AttrValue>,
}

impl From<BookmarkResponse> for BookmarkProps {
    fn from(value: BookmarkResponse) -> Self {
        BookmarkProps {
            id: value.id,
            url: AttrValue::from(value.url),
            title: value.title.map(|v| AttrValue::from(v)),
            description: value.description.map(|v| AttrValue::from(v)),
        }
    }
}

#[function_component(Bookmark)]
pub fn bookmark(props: &BookmarkProps) -> Html {
    html! {
        <li class="bookmark">
            <div class="bookmark__title">
                <a href={props.url.clone()}>
                    {props.title.clone().unwrap_or_else(|| props.url.clone())}
                </a>
            </div>
            <div class="bookmark__description">
                <div class="bookmark__tags-list">
                {"tag\u{00a0}·\u{00a0}other_tag"}
                </div>
                <div class="bookmark__actions">
                    <a href="#">{"edit"}</a>
                    {"\u{00a0}|\u{00a0}"}
                    <a href="#">{"delete"}</a>
                    {"\u{00a0}|\u{00a0}"}
                    {"2023-02-15 21:37"}
                    {"\u{00a0}|\u{00a0}"}
                    <a href="#">{"permalink"}</a>
                    {"\u{00a0}|\u{00a0}"}
                    <QrCode id={props.id} />
                </div>
                <div class="bookmark__link">
                    <a href={props.url.clone()}>
                        {props.url.clone()}
                    </a>
                </div>
            </div>
        </li>
    }
}
