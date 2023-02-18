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
    step: Step,
    url: AttrValue,
    title: AttrValue,
    description: AttrValue,
}

impl Default for State {
    fn default() -> Self {
        Self {
            step: Step::Init,
            url: AttrValue::from(""),
            title: AttrValue::from(""),
            description: AttrValue::from(""),
        }
    }
}

#[function_component(CreateBookmark)]
pub fn create_bookmark() -> Html {
    let state = use_state(|| State::default());
    let navigator = use_navigator().unwrap();

    let onsubmit = {
        let state = state.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            match state.step {
                Step::Init => {
                    let state = state.clone();
                    spawn_local(async move {
                        let url = URL_URLS.replace(":url", encode(state.url.as_str()).as_ref());
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
                        if let Ok(info) = info {
                            new_state.url = AttrValue::from(info.url);
                            new_state.title = AttrValue::from(info.title.unwrap_or("".to_string()));
                            new_state.description =
                                AttrValue::from(info.description.unwrap_or("".to_string()))
                        }
                        state.set(new_state);
                    });
                }
                Step::Details => {
                    let bookmark = CreateBookmarkRequest {
                        url: state.url.to_string(),
                        title: Some(state.title.to_string()),
                        description: Some(state.description.to_string()),
                    };
                    let navigator = navigator.clone();
                    spawn_local(async move {
                        // TODO finish this
                        let _todo = Request::post(URL_BOOKMARKS)
                            .json(&bookmark)
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
            new_state.url = AttrValue::from(input.value());
            state.set(new_state);
        })
    };
    let oninput_title = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_state = (*state).clone();
            new_state.title = AttrValue::from(input.value());
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
                        class="create-bookmark__url-input"
                        type="text"
                        value={state.url.clone()}
                        oninput={oninput_url}
                    />
                </p>
                if state.step == Step::Details {
                <p>{"Title"}</p>
                <p>
                    <input
                        class="create-bookmark__title-input"
                        type="text"
                        value={state.title.clone()}
                        oninput={oninput_title}
                    />
                </p>
                <p>{"Description"}</p>
                <p>
                    <textarea class="create-bookmark__description-input" value={state.description.clone()} />
                </p>
                }
                <p><button class="create-bookmark__submit">{"Add bookmark"}</button></p>
            </form>
        </div>
    }
}
