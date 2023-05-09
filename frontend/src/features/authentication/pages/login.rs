use crate::QueryParams as ParsedQueryParams;
use crate::Route;
use gloo_net::http::Request;
use rest_api::sessions::{CreateSessionRequest, CreateSessionResult, URL_SESSIONS};
use rest_api::RestPassword;
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_hooks::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub onlogin: Callback<AttrValue>,
    pub logged_in: bool,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Clone, Debug)]
pub struct QueryParams {
    pub redirect_to: Option<String>,
}

#[derive(Clone, PartialEq)]
enum Error {
    InvalidCredentials,
}

#[derive(Clone, Default)]
struct State {
    username: AttrValue,
    password: AttrValue,
    in_progress: bool,
    error: Option<Error>,
}

#[function_component(Login)]
pub fn login(props: &Props) -> Html {
    let state = use_state(State::default);
    let navigator = use_navigator().unwrap();
    let username_input_ref = use_node_ref();
    let query_params = yew_router::hooks::use_location()
        .unwrap()
        .query::<QueryParams>()
        .ok()
        .unwrap_or_default();

    {
        let logged_in = props.logged_in;
        let navigator = navigator.clone();
        use_effect(move || {
            if logged_in {
                navigator.push(&Route::Index);
            }
        });
    }
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
            let query_params = query_params.clone();
            spawn_local(async move {
                let result = CreateSessionResult::from(
                    Request::post(URL_SESSIONS)
                        .json(&CreateSessionRequest {
                            username: state.username.to_string(),
                            password: Secret::new(RestPassword(state.password.to_string())),
                        })
                        .expect("could not set json")
                        .send()
                        .await,
                )
                .await;

                let mut new_state = (*state).clone();
                new_state.in_progress = false;

                match result {
                    Some(CreateSessionResult::Success(_)) => {
                        props.onlogin.emit(state.username.clone());
                        if let Some(redirect_to) = query_params.redirect_to {
                            let route = match redirect_to.find('?') {
                                None => Route::recognize(&redirect_to).map(|r| (r, None)),
                                Some(index) => Route::recognize(&redirect_to[0..index])
                                    .map(|r| (r, Some(&redirect_to[index + 1..]))),
                            };

                            if let Some(route) = route {
                                match route
                                    .1
                                    .map(|q| route.0.parse_query_string(q))
                                    .unwrap_or_default()
                                {
                                    ParsedQueryParams::None => navigator.push(&route.0),
                                    ParsedQueryParams::AddBookmark(params) => {
                                        let _ = navigator.push_with_query(&route.0, &params);
                                    }
                                    ParsedQueryParams::Bookmarks(params) => {
                                        let _ = navigator.push_with_query(&route.0, &params);
                                    }
                                }
                            }
                        } else {
                            navigator.push(&Route::Index);
                        }
                    }
                    Some(CreateSessionResult::InvalidCredentials) => {
                        new_state.error = Some(Error::InvalidCredentials);
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
            { match state.error {
                Some(Error::InvalidCredentials) => html! {
                    <div class="centered-box__error">
                        {"Invalid credentials or account not active"}
                    </div>
                },
                None => html!{ <></> }
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
                <div class="centered-box__buttons">
                    <p>
                        <button type="submit" class={match state.in_progress {
                            true => "button--disabled".to_string(),
                            false => "button--action".to_string(),
                        }}>
                            {"Login"}
                        </button>
                    </p>
                    <p>{"or"}</p>
                    <p>
                        <a href={Route::SignupForm.to_path()} onclick={
                            let navigator = navigator.clone();
                            move |e:MouseEvent| {
                                e.prevent_default();
                                navigator.push(&Route::SignupForm);
                            }
                        }>
                            {"Create an account"}
                        </a>
                        {"\u{00a0}\u{ff5c}\u{00a0}"}
                        <a href={Route::RecoverPasswordStart.to_path()} onclick={
                            move |e:MouseEvent| {
                                e.prevent_default();
                                navigator.push(&Route::RecoverPasswordStart);
                            }
                        }>
                            {"I forgot my password"}
                        </a>
                    </p>
                </div>
            </form>
        </div>
    }
}
