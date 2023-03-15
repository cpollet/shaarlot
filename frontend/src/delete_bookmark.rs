use crate::data::Bookmark;
use crate::Route;
use gloo_net::http::Request;
use rest_api::bookmarks::delete::DeleteBookmarkResult;
use rest_api::bookmarks::{Access, URL_BOOKMARK};
use std::rc::Rc;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::hooks::use_navigator;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub bookmark: Rc<Bookmark>,
}

#[derive(Clone, PartialEq)]
enum Error {
    Forbidden,
    NotFound,
    Other,
}

#[derive(Clone, PartialEq)]
struct State {
    bookmark: Bookmark,
    // todo implement in_progress
    error: Option<Error>,
}

#[function_component(DeleteBookmark)]
pub fn delete_bookmark(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();
    let state = use_state(|| State {
        bookmark: (*props.bookmark).clone(),
        error: None,
    });

    let onclick_no = {
        let navigator = navigator.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            navigator.push(&Route::Bookmarks);
        })
    };

    let onclick_yes = {
        let id = props.bookmark.id;
        let state = state.clone();
        Callback::from(move |e: MouseEvent| {
            let navigator = navigator.clone();
            let state = state.clone();
            e.prevent_default();
            spawn_local(async move {
                match DeleteBookmarkResult::from(
                    Request::delete(&URL_BOOKMARK.replace(":id", &id.to_string()))
                        .send()
                        .await,
                )
                .await
                {
                    Some(DeleteBookmarkResult::Success) => navigator.push(&Route::Bookmarks),
                    Some(DeleteBookmarkResult::Forbidden) => {
                        let mut new_state = (*state).clone();
                        new_state.error = Some(Error::Forbidden);
                        state.set(new_state);
                    }
                    Some(DeleteBookmarkResult::NotFound(_, _)) => {
                        let mut new_state = (*state).clone();
                        new_state.error = Some(Error::NotFound);
                        state.set(new_state);
                    }
                    _ => {
                        let mut new_state = (*state).clone();
                        new_state.error = Some(Error::Other);
                        state.set(new_state);
                    }
                };
            });
        })
    };

    match state.bookmark.access {
        Access::Read => html! {
            <div class="centered-box">
                <h1 class="centered-box__title">{"Delete bookmark"}</h1>
                <div class="centered-box__error">
                    {"You don't have the right to delete this bookmark"}
                </div>
            </div>
        },
        Access::Write => html! {
            <div class="centered-box">
                <h1 class="delete-bookmark__title">{"Delete bookmark?"}</h1>
                { match state.error {
                    Some(Error::Forbidden) => html! {
                        <div class="centered-box__error">
                            {"You don't have the right to delete this bookmark"}
                        </div>
                    },
                    Some(Error::NotFound) => html! {
                        <div class="centered-box__error">
                            {"Bookmark not found"}
                        </div>
                    },
                    Some(_) => html! {
                        <div class="centered-box__error">
                            {"An error has occurred"}
                        </div>
                    },
                    None => html!{ <></> }
                }}
                <p>
                    <a href={state.bookmark.url.clone()}>
                        {state.bookmark.title.as_ref().cloned().unwrap_or(state.bookmark.url.clone())}
                    </a>
                </p>
                <p class="centered-box__buttons">
                    <button type="button" onclick={onclick_no} class="button--safe">{"Cancel"}</button>
                    {" "}
                    <button type="button" onclick={onclick_yes} class="button--danger">{"Delete"}</button>
                </p>
            </div>
        },
    }
}

#[function_component(DeleteBookmarkHOC)]
pub fn edit_bookmark_hoc() -> Html {
    let bookmark = use_context::<Rc<Bookmark>>().expect("no ctx found");
    html! {
        <DeleteBookmark bookmark={bookmark} />
    }
}
