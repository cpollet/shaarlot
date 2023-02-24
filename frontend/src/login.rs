use crate::Route;
use gloo_net::http::Request;
use rest_api::authentication::sessions::{
    CreateSessionRequest, CreateSessionResponseCode, URL_SESSIONS,
};
use rest_api::authentication::RestPassword;
use secrecy::Secret;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_hooks::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub onlogin: Callback<AttrValue>,
}

#[derive(Clone)]
struct State {
    username: AttrValue,
    password: AttrValue,
    invalid_credentials: bool,
    in_progress: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            username: AttrValue::default(),
            password: AttrValue::default(),
            invalid_credentials: false,
            in_progress: false,
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
        use_effect_once(move || {
            let _ = url_input_ref.cast::<HtmlInputElement>().unwrap().focus();
            || ()
        });
    }

    let onsubmit = {
        let props = props.clone();
        let navigator = navigator.clone();
        let state = state.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if state.in_progress {
                return;
            }

            let props = props.clone();
            let navigator = navigator.clone();
            let state = state.clone();
            {
                let mut new_state = (*state).clone();
                new_state.in_progress = true;
                state.set(new_state);
            }
            spawn_local(async move {
                let result = Request::post(&URL_SESSIONS)
                    .json(&CreateSessionRequest {
                        username: state.username.to_string(),
                        password: Secret::new(RestPassword(state.password.to_string())),
                    })
                    .expect("could not set json")
                    .send()
                    .await;

                let mut new_state = (*state).clone();
                new_state.in_progress = false;

                match result
                    .map(|r| CreateSessionResponseCode::from(r))
                    .unwrap_or(CreateSessionResponseCode::Other)
                {
                    CreateSessionResponseCode::Success => {
                        props.onlogin.emit(state.username.clone());
                        navigator.push(&Route::Index);
                    }
                    CreateSessionResponseCode::InvalidCredentials => {
                        new_state.invalid_credentials = true;
                    }
                    _ => {}
                }

                state.set(new_state);
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
        <div class="centered-box">
            <h1 class="centered-box__title">{"Login"}</h1>
            { match state.invalid_credentials {
                true => html! {
                    <div class="centered-box__error">
                        {"Invalid credentials"}
                    </div>
                },
                false => html!{ <></> }
            }}
            <form {onsubmit}>
                <p>
                    <input
                        ref={username_input_ref}
                        type="text"
                        placeholder="username"
                        value={state.username.clone()}
                        oninput={oninput_username}
                    />
                </p>
                <p>
                    <input
                        type="password"
                        placeholder="password"
                        value={state.password.clone()}
                        oninput={oninput_password}
                    />
                </p>
                <p>
                    <button type="submit" class={match state.in_progress {
                        true => "button--disabled".to_string(),
                        false => "button--action".to_string(),
                    }}>
                        {"Login"}
                    </button>
                </p>
            </form>
        </div>
    }
}
