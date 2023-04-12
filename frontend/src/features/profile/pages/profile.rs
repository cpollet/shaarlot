use common::{PasswordFlags, PasswordRules};
use gloo_net::http::Request;
use rest_api::users::get::GetUserResult;
use rest_api::users::update::{UpdateUserRequest, UpdateUserResult};
use rest_api::users::URL_CURRENT_USER;
use rest_api::RestPassword;
use secrecy::Secret;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_hooks::use_effect_once;

#[derive(Clone, PartialEq)]
enum Status {
    Default,
    InProgress,
}

impl Default for Status {
    fn default() -> Self {
        Status::Default
    }
}

// todo merge with status
#[derive(Clone, PartialEq)]
enum Error {
    Success,
    InvalidCurrentPassword,
    InvalidNewPassword,
    InvalidEmailAddress,
    Other,
}

#[derive(Clone, PartialEq)]
struct State {
    email: AttrValue,
    current_password: AttrValue,
    new_password: AttrValue,
    new_password_verif: AttrValue,
    status: Status,
    password_flags: PasswordFlags,
    error: Option<Error>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            email: Default::default(),
            current_password: Default::default(),
            new_password: Default::default(),
            new_password_verif: Default::default(),
            status: Default::default(),
            password_flags: PasswordFlags::valid(),
            error: None,
        }
    }
}

#[function_component(Profile)]
pub fn profile() -> Html {
    let state = use_state(State::default);
    let email_input_ref = use_node_ref();

    {
        let email_input_ref = email_input_ref.clone();
        let state = state.clone();
        use_effect_once(move || {
            let _ = email_input_ref.cast::<HtmlInputElement>().unwrap().focus();

            spawn_local(async move {
                let mut new_state = (*state).clone();
                new_state.status = Status::InProgress;
                state.set(new_state);

                let result = GetUserResult::from(Request::get(URL_CURRENT_USER).send().await).await;

                let mut new_state = (*state).clone();
                new_state.status = Status::Default;
                if let Some(GetUserResult::Success(payload)) = result {
                    new_state.email = AttrValue::from(payload.email);
                }

                state.set(new_state);
            });

            || ()
        });
    }

    let oninput_email = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let mut new_state = (*state).clone();
            let input: HtmlInputElement = e.target_unchecked_into();
            new_state.email = AttrValue::from(input.value());
            state.set(new_state);
        })
    };
    let oninput_current_password = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let mut new_state = (*state).clone();
            let input: HtmlInputElement = e.target_unchecked_into();
            new_state.current_password = AttrValue::from(input.value());
            state.set(new_state);
        })
    };
    let oninput_new_password = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let mut new_state = (*state).clone();
            let input: HtmlInputElement = e.target_unchecked_into();
            new_state.new_password = AttrValue::from(input.value());
            new_state = password_check(new_state);
            state.set(new_state);
        })
    };
    let oninput_new_password_verif = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let mut new_state = (*state).clone();
            let input: HtmlInputElement = e.target_unchecked_into();
            new_state.new_password_verif = AttrValue::from(input.value());
            new_state = password_check(new_state);
            state.set(new_state);
        })
    };

    fn password_check(mut s: State) -> State {
        if s.new_password.is_empty() && s.new_password_verif.is_empty() {
            s.password_flags = PasswordFlags::valid();
        } else {
            s.password_flags = PasswordRules::default()
                .validate(s.new_password.as_str(), s.new_password_verif.as_str());
        }
        s
    }

    let onsubmit = {
        let state = state.clone();
        // let password_input_ref = password_input_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if state.status == Status::InProgress || !state.password_flags.is_valid() {
                return;
            }

            {
                let mut new_state = (*state).clone();
                new_state.status = Status::InProgress;
                state.set(new_state);
            }

            let state = state.clone();
            spawn_local(async move {
                let new_password = if state.new_password.is_empty() {
                    None
                } else {
                    Some(Secret::new(RestPassword(state.new_password.to_string())))
                };
                let new_password_verif = if state.new_password_verif.is_empty() {
                    None
                } else {
                    Some(Secret::new(RestPassword(
                        state.new_password_verif.to_string(),
                    )))
                };
                let result = UpdateUserResult::from(
                    Request::post(URL_CURRENT_USER)
                        .json(&UpdateUserRequest {
                            email: state.email.to_string(),
                            current_password: Secret::new(RestPassword(
                                state.current_password.to_string(),
                            )),
                            new_password,
                            new_password_verif,
                        })
                        .expect("could not set json")
                        .send()
                        .await,
                )
                .await;

                let mut new_state = (*state).clone();
                new_state.status = Status::Default;

                match result {
                    Some(UpdateUserResult::Success(_)) => {
                        new_state.error = Some(Error::Success);
                    }
                    Some(UpdateUserResult::InvalidCurrentPassword) => {
                        new_state.new_password = AttrValue::default();
                        new_state.new_password_verif = AttrValue::default();
                        new_state.password_flags = PasswordFlags::valid();
                        new_state.error = Some(Error::InvalidCurrentPassword);
                    }
                    Some(UpdateUserResult::InvalidNewPassword) => {
                        new_state.new_password = AttrValue::default();
                        new_state.new_password_verif = AttrValue::default();
                        new_state.password_flags = PasswordFlags::valid();
                        new_state.error = Some(Error::InvalidNewPassword);
                    }
                    Some(UpdateUserResult::InvalidEmailAddress) => {
                        new_state.error = Some(Error::InvalidEmailAddress);
                    }
                    _ => {
                        new_state.error = Some(Error::Other);
                    }
                }

                state.set(new_state);
            })
        })
    };

    html! {
        <div class="centered-box">
           <h1 class="centered-box__title">{"My profile"}</h1>
            { match state.error {
                Some(Error::Success) => html! {
                    <div class="centered-box__ok">
                        {"Profile updated successfully"}
                    </div>
                },
                Some(Error::InvalidCurrentPassword) => html! {
                    <div class="centered-box__error">
                        {"Current password was incorrect"}
                    </div>
                },
                Some(Error::InvalidNewPassword) => html! {
                    <div class="centered-box__error">
                        {"New password was not acceptable"}
                    </div>
                },
                Some(Error::InvalidEmailAddress) => html! {
                    <div class="centered-box__error">
                        {"Email address was not valid"}
                    </div>
                },
                Some(Error::Other) => html! {
                    <div class="centered-box__error">
                        {"An error has occurred, try again later"}
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
                        type="password"
                        placeholder="new password (empty to keep current password)"
                        value={state.new_password.clone()}
                        oninput={oninput_new_password}
                    />
                </p>
                { if !(state.new_password.is_empty() && state.new_password_verif.is_empty()) {
                    html! {
                        <>
                            <p>
                                <input
                                    type="password"
                                    placeholder="new password verif."
                                    value={state.new_password_verif.clone()}
                                    oninput={oninput_new_password_verif}
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
                        </>
                    }
                } else { html! { <></> } } }

                <p>{"Your current password is mandatory to perform changes"}</p>
                <p>
                    <input
                        type="password"
                        placeholder="current password"
                        value={state.current_password.clone()}
                        oninput={oninput_current_password}
                    />
                </p>

                <p class="centered-box__buttons">
                    <button class={match state.status == Status::Default && state.password_flags.is_valid() {
                            true => "button--action".to_string(),
                            false => "button--disabled".to_string(),
                    }}>
                        {"Save"}
                    </button>
                </p>
            </form>
        </div>
    }
}
