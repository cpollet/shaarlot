use crate::data::Bookmark;
use gloo_net::http::Request;
use rest_api::{BookmarkResponse, URL_BOOKMARK};
use std::rc::Rc;
use yew::platform::spawn_local;
use yew::prelude::*;

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
        use_effect(move || {
            if state.is_none() {
                let props = props.clone();
                spawn_local(async move {
                    let res = fetch_bookmark(props.id).await;
                    // todo implement a 500 page
                    if let Ok(bookmark) = res {
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

async fn fetch_bookmark(id: i32) -> Result<Bookmark, String> {
    match Request::get(&URL_BOOKMARK.replace(":id", &id.to_string()))
        .send()
        .await
    {
        Err(_) => Err("Error fetching data".to_string()),
        Ok(resp) => {
            if !resp.ok() {
                Err(format!(
                    "Error fetching data: {} ({})",
                    resp.status(),
                    resp.status_text()
                ))
            } else {
                resp.json::<BookmarkResponse>()
                    .await
                    .map_err(|err| err.to_string())
                    .map(Bookmark::from)
            }
        }
    }
}
