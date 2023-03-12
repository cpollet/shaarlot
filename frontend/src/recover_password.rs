use gloo_net::http::Request;
use rest_api::password_recoveries::create::{
    CreatePasswordRecoveryRequest, CreatePasswordRecoveryResult,
};
use rest_api::password_recoveries::URL_PASSWORD_RECOVERIES;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq, Default)]
struct State {
    status: Status,
    username_or_email: AttrValue,
}

#[derive(Clone, PartialEq)]
enum Status {
    Default,
    InProgress,
    ServerError,
    ClientError,
    Success,
}

impl Default for Status {
    fn default() -> Self {
        Self::Default
    }
}

#[function_component(RecoverPassword)]
pub fn recover_password() -> Html {
    let state = use_state(State::default);
    let username_input_ref = use_node_ref();

    let oninput_username_or_email = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let mut new_state = (*state).clone();
            let input: HtmlInputElement = e.target_unchecked_into();
            new_state.username_or_email = AttrValue::from(input.value());
            state.set(new_state);
        })
    };

    let onsubmit = {
        let state = state.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if state.username_or_email.is_empty() {
                let mut new_state = (*state).clone();
                new_state.status = Status::ClientError;
                state.set(new_state);
                return;
            }
            if state.status == Status::InProgress {
                return;
            }

            let state = state.clone();
            {
                let mut new_state = (*state).clone();
                new_state.status = Status::InProgress;
                state.set(new_state);
            }
            spawn_local(async move {
                let result = CreatePasswordRecoveryResult::from(
                    Request::post(URL_PASSWORD_RECOVERIES)
                        .json(&CreatePasswordRecoveryRequest {
                            username_or_email: state.username_or_email.to_string(),
                        })
                        .expect("could not set json")
                        .send()
                        .await,
                )
                .await;

                let mut new_state = (*state).clone();
                new_state.status = Status::default();

                match result {
                    Some(CreatePasswordRecoveryResult::Success) => {
                        new_state.status = Status::Success;
                    }
                    Some(CreatePasswordRecoveryResult::ServerError) => {
                        new_state.status = Status::ServerError;
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
                Status::ServerError => html! {
                    <div class="centered-box__error">
                        {"An error occurred, try again later"}
                    </div>
                },
                Status::ClientError => html! {
                    <div class="centered-box__error">
                        {"Username or email address must be set"}
                    </div>
                },
                Status::Success => html! {
                    <div class="centered-box__ok">
                        {"An email has been sent to the registered email address"}
                    </div>
                },
                _ => html! {
                    <></>
                },
            }}
            <form {onsubmit}>
                <p>
                    <input
                        ref={username_input_ref}
                        type="text"
                        placeholder="username or email address"
                        oninput={oninput_username_or_email}
                    />
                </p>
                <div class="centered-box__buttons">
                    <p>
                        <button type="submit" class={match state.status {
                            Status::InProgress => "button--disabled".to_string(),
                            _ => "button--action".to_string(),
                        }}>
                            {"Send"}
                        </button>
                    </p>
                </div>
            </form>
        </div>
    }
}
