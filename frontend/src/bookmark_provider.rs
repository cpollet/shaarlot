use crate::data::Bookmark;
use gloo_net::http::Request;
use rest_api::bookmarks::get_one::GetBookmarkResult;
use rest_api::bookmarks::URL_BOOKMARK;
use std::rc::Rc;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_hooks::use_effect_once;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub id: i32,
    pub children: Children,
}

#[function_component(BookmarkProvider)]
pub fn bookmark_provider(props: &Props) -> Html {
    let state = use_state(|| None);

    {
        let state = state.clone();
        let props = props.clone();
        use_effect_once(move || {
            if state.is_none() {
                let props = props.clone();
                spawn_local(async move {
                    if let Some(bookmark) = fetch_bookmark(props.id).await {
                        state.set(Some(Rc::new(bookmark)))
                    }
                });
            }

            || {}
        });
    }

    match state.as_ref() {
        Some(bookmark) => html! {
            <ContextProvider<Rc<Bookmark>> context={(*bookmark).clone()}>
                { props.children.clone() }
            </ContextProvider<Rc<Bookmark>>>
        },
        None => html! {
            <div>{"No data"}</div>
        },
    }
}

async fn fetch_bookmark(id: i32) -> Option<Bookmark> {
    match GetBookmarkResult::from(
        Request::get(&URL_BOOKMARK.replace(":id", &id.to_string()))
            .send()
            .await,
    )
    .await
    {
        Some(GetBookmarkResult::Success(payload)) => Some(Bookmark::from(payload)),
        _ => {
            // todo handle error
            None
        }
    }
}
