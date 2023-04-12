use crate::Route;
use gloo_net::http::Request;
use rest_api::validate_email::{ValidateEmailResult, URL_EMAIL};
use uuid::Uuid;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_hooks::use_effect_once;
use yew_router::hooks::use_navigator;
use yew_router::Routable;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub uuid: Uuid,
}

enum State {
    Loading,
    Success,
    InvalidToken,
    ServerError,
}

#[function_component(ValidateEmail)]
pub fn validate_email(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();
    let state = use_state(|| State::Loading);

    {
        let props = props.clone();
        let state = state.clone();
        use_effect_once(move || {
            spawn_local(async move {
                let result = ValidateEmailResult::from(
                    Request::put(&URL_EMAIL.replace(":uuid", &props.uuid.to_string()))
                        .send()
                        .await,
                )
                .await
                .map(|r| match r {
                    ValidateEmailResult::Success => State::Success,
                    ValidateEmailResult::InvalidToken => State::InvalidToken,
                    ValidateEmailResult::ServerError => State::ServerError,
                    _ => State::Loading,
                })
                .unwrap_or(State::Loading);

                state.set(result);
            });

            || ()
        });
    }

    html!(
        <div class="centered-box">
            <h1 class="centered-box__title">{"Email validation"}</h1>
            { match *state {
                State::Success => html! {
                    <>
                        <div class="centered-box__ok">
                            {"Email verification successful."}
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
                State::InvalidToken => html! {
                    <div class="centered-box__error">
                        {"The token is invalid."}
                    </div>
                },
                _ => html! {
                    <div>
                        {"loading..."}
                    </div>
                },
            }}
        </div>
    )
}
