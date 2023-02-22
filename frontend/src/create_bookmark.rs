use crate::data::Bookmark;
use crate::Route;
use gloo_net::http::Request;
use rest_api::{CreateBookmarkRequest, UrlResponse, URL_BOOKMARKS, URL_URLS};
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
    step: Step,
    bookmark: Bookmark,
}

impl Default for State {
    fn default() -> Self {
        Self {
            focused: false,
            step: Step::Init,
            bookmark: Bookmark::default(),
        }
    }
}

#[function_component(CreateBookmark)]
pub fn create_bookmark() -> Html {
    let state = use_state(|| State::default());
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
            match state.step {
                Step::Init => {
                    let state = state.clone();
                    spawn_local(async move {
                        let url =
                            URL_URLS.replace(":url", encode(state.bookmark.url.as_str()).as_ref());
                        let info = match Request::get(&url).send().await {
                            Err(_) => Err("Error fetching data".to_string()),
                            Ok(resp) => {
                                if !resp.ok() {
                                    Err(format!(
                                        "Error fetching data: {} ({})",
                                        resp.status(),
                                        resp.status_text()
                                    ))
                                } else {
                                    match resp.json::<UrlResponse>().await {
                                        Ok(url) => Ok(url),
                                        Err(err) => Err(err.to_string()),
                                    }
                                }
                            }
                        };

                        let mut new_state = (*state).clone();
                        new_state.step = Step::Details;
                        new_state.focused = false;
                        if let Ok(info) = info {
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
                    let bookmark = state.bookmark.clone();
                    spawn_local(async move {
                        // TODO finish this
                        let _todo = Request::post(URL_BOOKMARKS)
                            .json(&CreateBookmarkRequest::from(&bookmark))
                            .expect("could not set json")
                            .send()
                            .await;
                        navigator.push(&Route::Bookmarks);
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
        <div class="create-bookmark">
            <h1 class="create-bookmark__title">{"Create bookmark"}</h1>
            <form {onsubmit}>
                <p>{"URL"}</p>
                <p>
                    <input
                        ref={url_input_ref}
                        class="create-bookmark__url-input"
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
                        class="create-bookmark__title-input"
                        type="text"
                        value={state.bookmark.title.clone()}
                        oninput={oninput_title}
                    />
                </p>
                <p>{"Description"}</p>
                <p>
                    <textarea
                        class="create-bookmark__description-input"
                        value={state.bookmark.description.clone()}
                        oninput={oninput_description}
                    />
                </p>
                }
                <p><button class="create-bookmark__submit">{"Add bookmark"}</button></p>
            </form>
        </div>
    }
}
