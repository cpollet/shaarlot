use super::super::data::{Bookmark, Tags};
use crate::components::tag_input::TagInput;
use crate::Route;
use gloo_net::http::Request;
use rest_api::bookmarks::update::{UpdateBookmarkRequest, UpdateBookmarkResult};
use rest_api::bookmarks::{Access, URL_BOOKMARK};
use std::rc::Rc;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub bookmark: Rc<Bookmark>,
    pub tags: Rc<Tags>,
}

#[derive(Clone, PartialEq)]
enum Error {
    Forbidden,
    NotFound,
    Other,
}

#[derive(Clone, PartialEq)]
struct State {
    focused: bool,
    bookmark: Bookmark,
    // todo implement in_progress
    error: Option<Error>,
}

#[function_component(EditBookmark)]
pub fn edit_bookmark(props: &Props) -> Html {
    let state = use_state(|| State {
        focused: false,
        bookmark: (*props.bookmark).clone(),
        error: None,
    });
    let navigator = use_navigator().unwrap();

    let url_input_ref = use_node_ref();

    {
        let url_input_ref = url_input_ref.clone();
        let state = state.clone();
        use_effect(move || {
            if !state.focused {
                let _ = url_input_ref.cast::<HtmlInputElement>().unwrap().focus();
                let mut new_state = (*state).clone();
                new_state.focused = true;
                state.set(new_state);
            }
        });
    }

    let onsubmit = {
        let state = state.clone();
        let navigator = navigator.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let state = state.clone();
            let navigator = navigator.clone();
            spawn_local(async move {
                match UpdateBookmarkResult::from(
                    Request::put(&URL_BOOKMARK.replace(":id", &state.bookmark.id.to_string()))
                        .json(&UpdateBookmarkRequest::from(&state.bookmark))
                        .expect("could not set json")
                        .send()
                        .await,
                )
                .await
                {
                    Some(UpdateBookmarkResult::Success(_)) => navigator.push(&Route::Bookmarks),
                    Some(UpdateBookmarkResult::Forbidden) => {
                        let mut new_state = (*state).clone();
                        new_state.error = Some(Error::Forbidden);
                        state.set(new_state);
                    }
                    Some(UpdateBookmarkResult::NotFound(_, _)) => {
                        let mut new_state = (*state).clone();
                        new_state.error = Some(Error::NotFound);
                        state.set(new_state);
                    }
                    _ => {
                        let mut new_state = (*state).clone();
                        new_state.error = Some(Error::Other);
                        state.set(new_state);
                    }
                }
            })
        })
    };
    let onclick_cancel = {
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            navigator.push(&Route::Bookmarks);
        })
    };

    let oninput_url = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let mut new_state = (*state).clone();
            let input: HtmlInputElement = e.target_unchecked_into();
            new_state.bookmark.url = AttrValue::from(input.value());
            state.set(new_state);
        })
    };
    let oninput_title = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let mut new_state = (*state).clone();
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            if value.is_empty() {
                new_state.bookmark.title = None
            } else {
                new_state.bookmark.title = Some(AttrValue::from(value))
            }
            state.set(new_state);
        })
    };
    let oninput_description = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let mut new_state = (*state).clone();
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            if value.is_empty() {
                new_state.bookmark.description = None
            } else {
                new_state.bookmark.description = Some(AttrValue::from(value))
            }
            state.set(new_state);
        })
    };
    let onupdate_tags = {
        let state = state.clone();
        Callback::from(move |tags: Vec<AttrValue>| {
            let mut new_state = (*state).clone();
            new_state.bookmark.tags = tags;
            state.set(new_state);
        })
    };

    match state.bookmark.access {
        Access::Read => html! {
            <div class="centered-box">
                <h1 class="centered-box__title">{"Edit bookmark"}</h1>
                <div class="centered-box__error">
                    {"You don't have the right to update this bookmark"}
                </div>
            </div>
        },
        Access::Write => html! {
            <div class="centered-box">
                <h1 class="centered-box__title">{"Edit bookmark"}</h1>
                { match state.error {
                    Some(Error::Forbidden) => html! {
                        <div class="centered-box__error">
                            {"You don't have the right to update this bookmark"}
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
                <form {onsubmit}>
                    <p>
                        <input
                            ref={url_input_ref}
                            type="text"
                            placeholder="url"
                            value={state.bookmark.url.clone()}
                            oninput={oninput_url}
                        />
                    </p>
                    <p>
                        <input
                            type="text"
                            placeholder="title"
                            value={state.bookmark.title.clone()}
                            oninput={oninput_title}
                        />
                    </p>
                    <p>
                        <textarea
                            placeholder="description"
                            value={state.bookmark.description.clone()}
                            oninput={oninput_description}
                        />
                    </p>
                    <p>
                        <TagInput
                            tags={
                                state.bookmark.tags.clone()
                            }
                            available_tags={Some(Rc::new(
                                props.tags
                                    .iter()
                                    .map(|t| t.name.clone())
                                    .collect::<Vec<AttrValue>>()
                            ))}
                            onupdate={onupdate_tags}
                        />
                    </p>
                    <p class="centered-box__buttons">
                        <button type="button" onclick={onclick_cancel} class="button--safe">{"Cancel"}</button>
                        {" "}
                        <button type="submit" class="button--action">{"Update"}</button>
                    </p>
                </form>
            </div>
        },
    }
}

#[function_component(EditBookmarkHOC)]
pub fn edit_bookmark_hoc() -> Html {
    let bookmark = use_context::<Rc<Bookmark>>().expect("no ctx found");
    let tags = use_context::<Rc<Tags>>().expect("no ctx found");

    html! {
        <EditBookmark {bookmark} {tags} />
    }
}
