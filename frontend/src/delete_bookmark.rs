use crate::data::Bookmark;
use crate::Route;
use gloo_net::http::Request;
use rest_api::bookmarks::URL_BOOKMARK;
use std::rc::Rc;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::hooks::use_navigator;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub bookmark: Rc<Bookmark>,
}

struct State {
    bookmark: Bookmark,
}

#[function_component(DeleteBookmark)]
pub fn delete_bookmark(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();
    let state = use_state(|| State {
        bookmark: (*props.bookmark).clone(),
    });

    let onclick_no = {
        let navigator = navigator.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            navigator.push(&Route::Bookmarks);
        })
    };

    let onclick_yes = {
        let navigator = navigator.clone();
        let id = props.bookmark.id;
        Callback::from(move |e: MouseEvent| {
            let navigator = navigator.clone();
            e.prevent_default();
            spawn_local(async move {
                del_bookmark(id).await;
                navigator.push(&Route::Bookmarks);
            });
        })
    };

    html! {
        <div class="centered-box">
            <h1 class="delete-bookmark__title">{"Delete bookmark?"}</h1>
            <p>
                <a href={state.bookmark.url.clone()}>
                    {state.bookmark.title.as_ref().map(|t|t.clone()).unwrap_or(state.bookmark.url.clone())}
                </a>
            </p>
            <p>
                <button type="button" onclick={onclick_no} class="button--safe">{"Cancel"}</button>
                {" "}
                <button type="button" onclick={onclick_yes} class="button--danger">{"Delete"}</button>
            </p>
        </div>
    }
}
async fn del_bookmark(id: i32) {
    // todo error handling?
    let _ = Request::delete(&URL_BOOKMARK.replace(":id", &id.to_string()))
        .send()
        .await;
}

#[function_component(DeleteBookmarkHOC)]
pub fn edit_bookmark_hoc() -> Html {
    let bookmark = use_context::<Rc<Bookmark>>().expect("no ctx found");
    html! {
        <DeleteBookmark bookmark={bookmark.clone()} />
    }
}
