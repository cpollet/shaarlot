use super::super::super::super::data::{Bookmark, Tags};
use crate::components::tag_input::TagInput;
use crate::Route;
use gloo_net::http::Request;
use rest_api::bookmarks::create::{CreateBookmarkRequest, CreateBookmarkResult};
use rest_api::bookmarks::URL_BOOKMARKS;
use rest_api::urls::{GetUrlResult, URL_URLS};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use urlencoding::encode;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    tags: Rc<Tags>,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Clone, Debug)]
pub struct QueryParams {
    url: Option<String>,
}

#[derive(Copy, Clone, PartialEq)]
enum Step {
    Init,
    Details,
}

impl Default for Step {
    fn default() -> Self {
        Self::Init
    }
}

#[derive(Clone, PartialEq)]
enum Error {
    Forbidden,
    Other,
}

#[derive(Clone, PartialEq, Default)]
struct State {
    focused: bool,
    in_progress: bool,
    step: Step,
    bookmark: Bookmark,
    error: Option<Error>,
}

#[function_component(CreateBookmark)]
pub fn create_bookmark(props: &Props) -> Html {
    let state = use_state(State::default);
    let navigator = use_navigator().unwrap();
    let url_input_ref = use_node_ref();
    let title_input_ref = use_node_ref();
    let query_params = use_location()
        .unwrap()
        .query::<QueryParams>()
        .ok()
        .unwrap_or_default();

    {
        let url_input_ref = url_input_ref.clone();
        let title_input_ref = title_input_ref.clone();
        let state = state.clone();
        use_effect(move || match state.step {
            Step::Init => {
                if !state.focused {
                    let _ = url_input_ref.cast::<HtmlInputElement>().unwrap().focus();
                    let mut new_state = (*state).clone();
                    new_state.focused = true;
                    state.set(new_state);
                }
            }
            Step::Details => {
                if !state.focused {
                    let _ = title_input_ref.cast::<HtmlInputElement>().unwrap().focus();
                    let mut new_state = (*state).clone();
                    new_state.focused = true;
                    state.set(new_state);
                }
            }
        })
    }

    {
        let state = state.clone();
        let navigator = navigator.clone();
        use_effect_with_deps(
            move |query_params| {
                if let Some(url) = query_params.url.clone() {
                    fetch_url_data(&url, state, navigator);
                } else {
                    let mut new_state = (*state).clone();
                    new_state.step = Step::Init;
                    new_state.focused = false;
                    new_state.bookmark.url = AttrValue::default();
                    state.set(new_state);
                }
            },
            query_params,
        );
    }

    let onsubmit = {
        let state = state.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if state.in_progress {
                return;
            }
            match state.step {
                Step::Init => {
                    fetch_url_data(
                        state.bookmark.url.as_str(),
                        state.clone(),
                        navigator.clone(),
                    );
                }
                Step::Details => {
                    {
                        let mut new_state = (*state).clone();
                        new_state.in_progress = true;
                        state.set(new_state);
                    }
                    let navigator = navigator.clone();
                    let state = state.clone();
                    spawn_local(async move {
                        match CreateBookmarkResult::from(
                            Request::post(URL_BOOKMARKS)
                                .json(&CreateBookmarkRequest::from(&state.bookmark))
                                .expect("could not set json")
                                .send()
                                .await,
                        )
                        .await
                        {
                            Some(CreateBookmarkResult::Success(_)) => {
                                navigator.push(&Route::Bookmarks)
                            }
                            Some(CreateBookmarkResult::Forbidden) => {
                                let mut new_state = (*state).clone();
                                new_state.error = Some(Error::Forbidden);
                                state.set(new_state);
                            }
                            _ => {
                                let mut new_state = (*state).clone();
                                new_state.in_progress = false;
                                new_state.error = Some(Error::Other);
                                state.set(new_state);
                            }
                        }
                    });
                }
            }
        })
    };

    let oninput_url = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_state = (*state).clone();
            new_state.step = Step::Init;
            new_state.bookmark.url = AttrValue::from(input.value());
            state.set(new_state);
        })
    };
    let oninput_title = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            let mut new_state = (*state).clone();
            if value.is_empty() {
                new_state.bookmark.title = None;
            } else {
                new_state.bookmark.title = Some(AttrValue::from(value));
            }
            state.set(new_state);
        })
    };
    let oninput_description = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            let mut new_state = (*state).clone();
            if value.is_empty() {
                new_state.bookmark.description = None;
            } else {
                new_state.bookmark.description = Some(AttrValue::from(value));
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
    let onclick_private = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            new_state.bookmark.private = !new_state.bookmark.private;
            state.set(new_state);
        })
    };

    html! {
        <div class="centered-box">
            <h1 class="centered-box__title">{"Create bookmark"}</h1>
            { match state.error {
                Some(Error::Forbidden) => html! {
                    <div class="centered-box__error">
                        {"You don't have the right to create bookmarks"}
                    </div>
                },
                Some(_) => html! {
                    <div class="centered-box__error">
                        {"An error has occurred"}
                    </div>
                },
                None => html! {
                    <></>
                }
            }}
            <form {onsubmit}>
                <p>
                    <input
                        ref={url_input_ref}
                        type="text"
                        placeholder="URL"
                        value={state.bookmark.url.clone()}
                        oninput={oninput_url}
                    />
                </p>
                if state.step == Step::Details {
                <p>
                    <input
                        ref={title_input_ref}
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
                        placeholder="tags"
                        tags={vec![]}
                        available_tags={Some(Rc::new(
                            props.tags
                                .iter()
                                .map(|t| t.name.clone())
                                .collect::<Vec<AttrValue>>()
                        ))}
                        onupdate={onupdate_tags}
                    />
                </p>
                <p>
                    <input
                        type="checkbox"
                        id="private"
                        checked={state.bookmark.private}
                        onclick={onclick_private}
                    />
                    <label for="private">{"Private"}</label>
                </p>
                }
                <p class="centered-box__buttons">
                    <button class={match state.in_progress {
                        true => "button--disabled".to_string(),
                        false => "button--action".to_string(),
                    }}>
                        {"Add bookmark"}
                    </button>
                </p>
            </form>
        </div>
    }
}

fn fetch_url_data(url: &str, state: UseStateHandle<State>, navigator: Navigator) {
    {
        let mut new_state = (*state).clone();
        new_state.in_progress = true;
        new_state.bookmark.url = AttrValue::from(url.to_string());
        state.set(new_state);
    }

    let url = URL_URLS.replace(":url", encode(url).as_ref());
    spawn_local(async move {
        let payload = match GetUrlResult::from(Request::get(&url).send().await).await {
            Some(GetUrlResult::Success(payload)) => Some(payload),
            Some(GetUrlResult::Conflict(payload)) => {
                navigator.push(&Route::EditBookmark { id: payload.id });
                None
            }
            _ => None,
        };

        let mut new_state = (*state).clone();
        new_state.step = Step::Details;
        new_state.focused = false;
        new_state.in_progress = false;

        if let Some(payload) = payload {
            new_state.bookmark.url = AttrValue::from(payload.url);
            new_state.bookmark.title = payload.title.map(AttrValue::from);
            new_state.bookmark.description = payload.description.map(AttrValue::from)
        }
        state.set(new_state);
    });
}

#[function_component(CreateBookmarkHOC)]
pub fn create_bookmark_hoc() -> Html {
    let tags = use_context::<Rc<Tags>>().expect("no ctx found");

    html! {
        <CreateBookmark {tags}/>
    }
}
