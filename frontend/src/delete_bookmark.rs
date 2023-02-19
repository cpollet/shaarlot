use crate::data::Bookmark;
use crate::Route;
use gloo_net::http::Request;
use rest_api::{BookmarkResponse, URL_BOOKMARK};
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::hooks::use_navigator;

#[derive(Properties, PartialEq, Clone, Copy)]
pub struct Props {
    pub id: i32,
}

struct State {
    bookmark: Bookmark,
}

#[function_component(DeleteBookmark)]
pub fn delete_bookmark(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();
    let state = use_state(|| None);

    {
        let state = state.clone();
        let props = props.clone();
        use_effect(move || {
            if state.is_none() {
                spawn_local(async move {
                    let res = fetch_bookmark(props.id).await;
                    // todo implement a 500 page
                    if let Ok(bookmark) = res {
                        state.set(Some(State { bookmark }))
                    }
                });
            }

            || {}
        });
    }

    let onclick_no = {
        let navigator = navigator.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            navigator.push(&Route::Bookmarks);
        })
    };

    let onclick_yes = {
        let navigator = navigator.clone();
        let props = props.clone();
        Callback::from(move |e: MouseEvent| {
            let navigator = navigator.clone();
            e.prevent_default();
            spawn_local(async move {
                del_bookmark(props.id).await;
                navigator.push(&Route::Bookmarks);
            });
        })
    };

    html! {
        <div class="delete-bookmark">
            <h1 class="delete-bookmark__title">{"Delete bookmark?"}</h1>
            {state.as_ref().map(|s|{
                html! {
                    <>
                        <p><a href={s.bookmark.url.clone()}>{s.bookmark.title.clone()}</a></p>
                        <p>
                            <button type="button" onclick={onclick_no} class="delete-bookmark__submit--safe">{"Cancel"}</button>
                            {" "}
                            <button type="button" onclick={onclick_yes} class="delete-bookmark__submit--danger">{"Delete"}</button>
                        </p>
                    </>
                }
            })}
        </div>
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

async fn del_bookmark(id: i32) {
    // todo error handling?
    let _ = Request::delete(&URL_BOOKMARK.replace(":id", &id.to_string()))
        .send()
        .await;
}
