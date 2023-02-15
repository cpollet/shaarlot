use yew::prelude::*;
use rest_api::BookmarkResponse;


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
            title: value.title.map(|v|AttrValue::from(v)),
            description: value.description.map(|v|AttrValue::from(v)),
        }
    }
}

#[function_component(Bookmark)]
pub fn bookmark(props: &BookmarkProps) -> Html {
    html! {
        <li>
            <div>
                <a href={props.url.clone()}>
                    {props.title.clone().unwrap_or_else(|| props.url.clone())}
                </a>
            </div>
            <div>
                <div>
                {"tag\u{00a0}·\u{00a0}other_tag"}
                </div>
                <div>
                    <a href="#">{"edit"}</a>
                    {"\u{00a0}|\u{00a0}"}
                    <a href="#">{"delete"}</a>
                    {"\u{00a0}|\u{00a0}"}
                    {"2023-02-15 21:37"}
                    {"\u{00a0}|\u{00a0}"}
                    <a href="#">{"permalink"}</a>
                    {"\u{00a0}|\u{00a0}"}
                    <a href="#">
                        <img src="https://fakeimg.pl/20x20/" />
                    </a>
                </div>
                <div>
                    <a href={props.url.clone()}>
                        {props.url.clone()}
                    </a>
                </div>
            </div>
        </li>
    }
}