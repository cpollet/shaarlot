use crate::data::Bookmark;
use crate::Route;
use gloo_net::http::Request;
use rest_api::bookmarks::{CreateBookmarkRequest, URL_BOOKMARKS};
use rest_api::urls::{UrlResponse, URL_URLS};
use urlencoding::encode;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::hooks::use_navigator;

#[derive(Copy, Clone, PartialEq)]
enum Step {
    Init,
    Details,
}

#[derive(Clone, PartialEq)]
struct State {
    focused: bool,
    in_progress: bool,
    has_error: bool,
    step: Step,
    bookmark: Bookmark,
}

impl Default for State {
    fn default() -> Self {
        Self {
            focused: false,
            in_progress: false,
            has_error: false,
            step: Step::Init,
            bookmark: Bookmark::default(),
        }
    }
}

#[function_component(CreateBookmark)]
pub fn create_bookmark() -> Html {
    let state = use_state(State::default);
    let navigator = use_navigator().unwrap();
    let url_input_ref = use_node_ref();
    let title_input_ref = use_node_ref();

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

    let onsubmit = {
        let state = state.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if state.in_progress {
                return;
            }
            {
                let mut new_state = (*state).clone();
                new_state.in_progress = true;
                state.set(new_state);
            }
            match state.step {
                Step::Init => {
                    let state = state.clone();
                    spawn_local(async move {
                        let url =
                            URL_URLS.replace(":url", encode(state.bookmark.url.as_str()).as_ref());

                        let info = match Request::get(&url).send().await {
                            Ok(response) => {
                                if response.ok() {
                                    match response.json::<UrlResponse>().await {
                                        Ok(url) => Some(url),
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                }
                            }
                            Err(_) => None,
                        };

                        let mut new_state = (*state).clone();
                        new_state.step = Step::Details;
                        new_state.focused = false;
                        new_state.in_progress = false;

                        if let Some(info) = info {
                            new_state.bookmark.url = AttrValue::from(info.url);
                            new_state.bookmark.title = info.title.map(|v| AttrValue::from(v));
                            new_state.bookmark.description =
                                info.description.map(|v| AttrValue::from(v))
                        }
                        state.set(new_state);
                    });
                }
                Step::Details => {
                    let navigator = navigator.clone();
                    let state = state.clone();
                    spawn_local(async move {
                        match Request::post(URL_BOOKMARKS)
                            .json(&CreateBookmarkRequest::from(&state.bookmark))
                            .expect("could not set json")
                            .send()
                            .await
                        {
                            Ok(response) if response.ok() => navigator.push(&Route::Bookmarks),
                            _ => {
                                let mut new_state = (*state).clone();
                                new_state.in_progress = false;
                                new_state.has_error = true;
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
            let value = e
                .target_unchecked_into::<HtmlInputElement>()
                .value()
                .to_string();
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
            let value = e
                .target_unchecked_into::<HtmlInputElement>()
                .value()
                .to_string();
            let mut new_state = (*state).clone();
            if value.is_empty() {
                new_state.bookmark.description = None;
            } else {
                new_state.bookmark.description = Some(AttrValue::from(value));
            }
            state.set(new_state);
        })
    };

    html! {
        <div class="centered-box">
            <h1 class="centered-box__title">{"Create bookmark"}</h1>
            { match state.has_error {
                true => html! {
                    <div class="centered-box__error">
                        {"Failed to create bookmark"}
                    </div>
                },
                false => html! {
                    <></>
                }
            }}
            <form {onsubmit}>
                <p>{"URL"}</p>
                <p>
                    <input
                        ref={url_input_ref}
                        type="text"
                        value={state.bookmark.url.clone()}
                        oninput={oninput_url}
                    />
                </p>
                if state.step == Step::Details {
                <p>{"Title"}</p>
                <p>
                    <input
                        ref={title_input_ref}
                        type="text"
                        value={state.bookmark.title.clone()}
                        oninput={oninput_title}
                    />
                </p>
                <p>{"Description"}</p>
                <p>
                    <textarea
                        value={state.bookmark.description.clone()}
                        oninput={oninput_description}
                    />
                </p>
                }
                <p>
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
