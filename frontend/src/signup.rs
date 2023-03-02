use crate::Route;
use common::{PasswordFlags, PasswordRules};
use gloo_net::http::Request;
use rest_api::users::{CreateUserRequest, CreateUserResult, URL_USERS};
use rest_api::RestPassword;
use secrecy::Secret;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_hooks::use_effect_once;
use yew_router::hooks::use_navigator;

#[derive(Clone, PartialEq)]
enum Error {
    InvalidPassword,
}

#[derive(Clone, PartialEq)]
struct State {
    email: AttrValue,
    username: AttrValue,
    password: AttrValue,
    password_verif: AttrValue,
    /// form submission in progress
    in_progress: bool,
    /// live password check result
    password_flags: PasswordFlags,
    error: Option<Error>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            email: AttrValue::default(),
            username: AttrValue::default(),
            password: AttrValue::default(),
            password_verif: AttrValue::default(),
            in_progress: false,
            password_flags: PasswordFlags::default(),
            error: None,
        }
    }
}

#[function_component(Signup)]
pub fn signup() -> Html {
    let navigator = use_navigator().unwrap();
    let state = use_state(State::default);
    let email_input_ref = use_node_ref();
    let password_input_ref = use_node_ref();

    {
        let url_input_ref = email_input_ref.clone();
        use_effect_once(move || {
            let _ = url_input_ref.cast::<HtmlInputElement>().unwrap().focus();
            || ()
        });
    }

    let onsubmit = {
        let navigator = navigator.clone();
        let state = state.clone();
        let password_input_ref = password_input_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if state.in_progress || !state.password_flags.is_valid() {
                return;
            }

            let navigator = navigator.clone();
            let state = state.clone();
            let password_input_ref = password_input_ref.clone();
            {
                let mut new_state = (*state).clone();
                new_state.in_progress = true;
                state.set(new_state);
            }
            spawn_local(async move {
                let result = CreateUserResult::from(
                    Request::post(&URL_USERS)
                        .json(&CreateUserRequest {
                            email: state.email.to_string(),
                            username: state.username.to_string(),
                            password: Secret::new(RestPassword(state.password.to_string())),
                            password_verif: Secret::new(RestPassword(
                                state.password_verif.to_string(),
                            )),
                        })
                        .expect("could not set json")
                        .send()
                        .await,
                )
                .await;

                let mut new_state = (*state).clone();
                new_state.in_progress = false;

                match result {
                    Some(CreateUserResult::Success(_)) => {
                        navigator.push(&Route::Login);
                    }
                    Some(CreateUserResult::InvalidPassword) => {
                        new_state.password = AttrValue::default();
                        new_state.password_verif = AttrValue::default();
                        new_state.password_flags = PasswordFlags::default();
                        new_state.error = Some(Error::InvalidPassword);
                        let _ = password_input_ref
                            .cast::<HtmlInputElement>()
                            .unwrap()
                            .focus();
                    }
                    _ => {}
                }

                state.set(new_state);
            })
        })
    };

    let oninput_email = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let mut new_state = (*state).clone();
            let input: HtmlInputElement = e.target_unchecked_into();
            new_state.email = AttrValue::from(input.value());
            state.set(new_state);
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
            new_state = password_check(new_state);
            state.set(new_state);
        })
    };
    let oninput_password_verif = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let mut new_state = (*state).clone();
            let input: HtmlInputElement = e.target_unchecked_into();
            new_state.password_verif = AttrValue::from(input.value());
            new_state = password_check(new_state);
            state.set(new_state);
        })
    };

    fn password_check(mut s: State) -> State {
        s.password_flags =
            PasswordRules::default().validate(s.password.as_str(), s.password_verif.as_str());
        s
    }

    html! {
        <div class="centered-box">
           <h1 class="centered-box__title">{"Create account"}</h1>
            { match state.error {
                Some(Error::InvalidPassword) => html! {
                    <div class="centered-box__error">
                        {"Invalid password"}
                    </div>
                },
                None => html!{ <></> }
            }}
            <form {onsubmit}>
                <p>
                    <input
                        ref={email_input_ref}
                        type="text"
                        placeholder="email@example.com"
                        value={state.email.clone()}
                        oninput={oninput_email}
                    />
                </p>
                <p>
                    <input
                        type="text"
                        placeholder="username"
                        value={state.username.clone()}
                        oninput={oninput_username}
                    />
                </p>
                <p>
                    <input
                        ref={password_input_ref}
                        type="password"
                        placeholder="password"
                        value={state.password.clone()}
                        oninput={oninput_password}
                    />
                </p>
                <p>
                    <input
                        type="password"
                        placeholder="password verif."
                        value={state.password_verif.clone()}
                        oninput={oninput_password_verif}
                    />
                </p>
                <ul class="password-checks">
                    <li class={if !state.password_flags.length { "centered-box__error" } else { "centered-box__ok" } }>
                        {if !state.password_flags.length { "\u{2717}" } else { "\u{2713}" } }{" min. 8 characters"}
                    </li>
                    <li class={if !state.password_flags.lower_case { "centered-box__error" } else { "centered-box__ok" } }>
                        {if !state.password_flags.lower_case { "\u{2717}" } else { "\u{2713}" } }{" at least one lower case letter"}
                    </li>
                    <li class={if !state.password_flags.upper_case { "centered-box__error" } else { "centered-box__ok" } }>
                        {if !state.password_flags.upper_case { "\u{2717}" } else { "\u{2713}" } }{" at least one upper case letter"}
                    </li>
                    <li class={if !state.password_flags.digits { "centered-box__error" } else { "centered-box__ok" } }>
                        {if !state.password_flags.digits { "\u{2717}" } else { "\u{2713}" } }{" at least one digit"}
                    </li>
                    <li class={if !state.password_flags.symbols { "centered-box__error" } else { "centered-box__ok" } }>
                        {if !state.password_flags.symbols { "\u{2717}" } else { "\u{2713}" } }{" at least one symbol"}
                    </li>
                    <li class={if !state.password_flags.same { "centered-box__error" } else { "centered-box__ok" } }>
                        {if !state.password_flags.same { "\u{2717}" } else { "\u{2713}" } }{" passwords match"}
                    </li>
                </ul>
                <p class="centered-box__buttons">
                <button class={match state.in_progress||!state.password_flags.is_valid() {
                        true => "button--disabled".to_string(),
                        false => "button--action".to_string(),
                }}>
                    {"Create account"}
                </button>
                </p>
            </form>
        </div>
    }
}
