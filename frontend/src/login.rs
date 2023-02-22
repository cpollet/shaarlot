use crate::Route;
use gloo_net::http::Request;
use rest_api::{CreateSessionRequest, RestPassword, URL_SESSIONS};
use secrecy::Secret;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub onlogin: Callback<AttrValue>,
}

#[derive(Clone)]
struct State {
    focused: bool,
    username: AttrValue,
    password: AttrValue,
}

impl Default for State {
    fn default() -> Self {
        Self {
            focused: false,
            username: AttrValue::default(),
            password: AttrValue::default(),
        }
    }
}

#[function_component(Login)]
pub fn login(props: &Props) -> Html {
    let state = use_state(|| State::default());
    let navigator = use_navigator().unwrap();
    let username_input_ref = use_node_ref();

    {
        let url_input_ref = username_input_ref.clone();
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
        let props = props.clone();
        let navigator = navigator.clone();
        let state = state.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let props = props.clone();
            let navigator = navigator.clone();
            let state = state.clone();
            spawn_local(async move {
                // TODO finish this (error cases)
                let result = Request::post(&URL_SESSIONS)
                    .json(&CreateSessionRequest {
                        username: state.username.to_string(),
                        password: Secret::new(RestPassword(state.password.to_string())),
                    })
                    .expect("could not set json")
                    .send()
                    .await;

                if let Ok(response) = result {
                    if response.ok() {
                        props.onlogin.emit(state.username.clone());
                    }
                }

                navigator.push(&Route::Index);
            })
        })
    };

    let oninput_username = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let mut new_state = (*state).clone();
            let input: HtmlInputElement = e.target_unchecked_into();
            new_state.username = AttrValue::from(input.value());
            state.set(new_state);
        })
    };
    let oninput_password = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let mut new_state = (*state).clone();
            let input: HtmlInputElement = e.target_unchecked_into();
            new_state.password = AttrValue::from(input.value());
            state.set(new_state);
        })
    };

    html! {
        <div class="edit-bookmark">
            <h1 class="edit-bookmark__title">{"Login"}</h1>
            <form {onsubmit}>
                <p>
                    <input
                        ref={username_input_ref}
                        class="edit-bookmark__url-input"
                        type="text"
                        placeholder="username"
                        value={state.username.clone()}
                        oninput={oninput_username}
                    />
                </p>
                <p>
                    <input
                        class="edit-bookmark__title-input"
                        type="password"
                        placeholder="password"
                        value={state.password.clone()}
                        oninput={oninput_password}
                    />
                </p>
                <p>
                    <button type="submit" class="edit-bookmark__submit--action">{"Login"}</button>
                </p>
            </form>
        </div>
    }
}
