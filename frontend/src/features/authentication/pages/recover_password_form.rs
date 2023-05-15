use crate::Route;
use common::{PasswordFlags, PasswordRules};
use gloo_net::http::Request;
use rest_api::password_recoveries::update::{
    UpdatePasswordRecoveryRequest, UpdatePasswordRecoveryResult,
};
use rest_api::password_recoveries::URL_PASSWORD_RECOVERIES;
use rest_api::{RestPassword, RestToken};
use secrecy::Secret;
use serde::Deserialize;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    id: AttrValue,
    token: AttrValue,
}

#[derive(Clone, PartialEq, Default)]
struct State {
    password: AttrValue,
    password_verif: AttrValue,
    password_flags: PasswordFlags,
    status: Status,
}

#[derive(Clone, PartialEq)]
enum Status {
    Default,
    Success,
    InProgress,
    InvalidToken,
    InvalidPassword,
    ServerError,
    NotAvailable,
}

impl Default for Status {
    fn default() -> Self {
        Self::Default
    }
}

#[function_component(RecoverPasswordForm)]
pub fn recover_password_form(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();
    let state = use_state(State::default);
    let password_input_ref = use_node_ref();

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

    let onsubmit = {
        let state = state.clone();
        let props = props.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if state.status == Status::InProgress || !state.password_flags.is_valid() {
                return;
            }

            let state = state.clone();
            {
                let mut new_state = (*state).clone();
                new_state.status = Status::InProgress;
                state.set(new_state);
            }

            let props = props.clone();
            spawn_local(async move {
                let result = UpdatePasswordRecoveryResult::from(
                    Request::put(URL_PASSWORD_RECOVERIES)
                        .json(&UpdatePasswordRecoveryRequest {
                            id: props.id.to_string(),
                            token: Secret::new(RestToken(props.token.to_string())),
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
                new_state.status = Status::default();

                match result {
                    Some(UpdatePasswordRecoveryResult::Success) => {
                        new_state.status = Status::Success;
                    }
                    Some(UpdatePasswordRecoveryResult::InvalidToken) => {
                        new_state.status = Status::InvalidToken;
                    }
                    Some(UpdatePasswordRecoveryResult::InvalidPassword) => {
                        new_state.status = Status::InvalidPassword;
                    }
                    Some(UpdatePasswordRecoveryResult::ServerError) => {
                        new_state.status = Status::ServerError;
                    }
                    Some(UpdatePasswordRecoveryResult::NotImplemented) => {
                        new_state.status = Status::NotAvailable;
                    }
                    _ => {}
                }

                state.set(new_state);
            })
        })
    };

    html! {
        <div class="centered-box">
            <h1 class="centered-box__title">{"Password recovery"}</h1>
            { match state.status {
                Status::Success => html! {
                    <>
                        <div class="centered-box__ok">
                            {"Password modification successful."}
                        </div>
                        <div>
                            {"You may now "}
                            <a
                                onclick={{
                                    let navigator = navigator.clone();
                                    move |e:MouseEvent| {
                                        e.prevent_default();
                                        navigator.push(&Route::Login);
                                    }
                                }}
                                href={Route::Login.to_path()}
                            >
                                {"log in"}
                            </a>{"."}
                        </div>
                    </>
                },
                Status::ServerError => html! {
                    <div class="centered-box__error">
                        {"An error occurred, try again later"}
                    </div>
                },
                Status::InvalidToken => html! {
                    <div class="centered-box__error">
                        {"The provided token is invalid or expired"}
                    </div>
                },
                Status::InvalidPassword => html! {
                    <div class="centered-box__ok">
                        {"The provided password was not acceptable"}
                    </div>
                },
                Status::NotAvailable => html! {
                    <div class="centered-box__error">
                        {"Not available in demo mode"}
                    </div>
                },
                _ => html! {
                    <></>
                },
            }}
            { if state.status != Status::Success {
                html! {
                    <form {onsubmit}>
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
                        <div class="centered-box__buttons">
                            <p>
                                <button type="submit" class={match state.status {
                                    Status::InProgress => "button--disabled".to_string(),
                                    _ => "button--action".to_string(),
                                }}>
                                    {"Change password"}
                                </button>
                            </p>
                        </div>
                    </form>
                }
            } else { html! { <></> } }}
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct HocProps {
    pub id: AttrValue,
}

#[derive(Deserialize, Debug)]
struct Query {
    token: String,
}

#[function_component(RecoverPasswordFormHOC)]
pub fn recover_password_form_hoc(props: &HocProps) -> Html {
    let token = use_location()
        .unwrap()
        .query::<Query>()
        .map(|q| q.token)
        .ok()
        .unwrap_or_default();
    html! {
        <RecoverPasswordForm id={props.id.clone()} {token} />
    }
}
