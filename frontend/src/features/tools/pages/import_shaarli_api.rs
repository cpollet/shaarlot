use crate::Route;
use gloo_net::http::Request;
use rest_api::import_shaarli_api::{
    ShaarliApiKey, ShaarliImportApiRequest, ShaarliImportApiResult, URL_SHAARLI_IMPORT_API,
};
use secrecy::Secret;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, PartialEq, Default)]
struct State {
    status: Status,
    api_url: AttrValue,
    api_key: AttrValue,
}

#[derive(Clone, PartialEq, Default)]
enum Status {
    #[default]
    Default,
    Importing,
    Forbidden,
    ShaarliError,
    GenericError,
}

#[function_component(ToolImportShaarliApi)]
pub fn tool_import_shaarli_api() -> Html {
    let navigator = use_navigator().unwrap();
    let state = use_state(State::default);

    let base_url_input_ref = use_node_ref();

    let onsubmit = {
        let state = state.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if state.status != Status::Importing {
                let mut new_state = (*state).clone();
                new_state.status = Status::Importing;
                state.set(new_state);

                let state = state.clone();
                let navigator = navigator.clone();
                spawn_local(async move {
                    gloo_console::info!(format!("submit {} {}", state.api_url, state.api_key));
                    match ShaarliImportApiResult::from(
                        Request::post(URL_SHAARLI_IMPORT_API)
                            .json(&ShaarliImportApiRequest {
                                url: state.api_url.to_string(),
                                key: Secret::new(ShaarliApiKey(state.api_key.to_string())),
                            })
                            .expect("could not set json")
                            .send()
                            .await,
                    )
                    .await
                    {
                        Some(ShaarliImportApiResult::Success) => navigator.push(&Route::Bookmarks),
                        Some(ShaarliImportApiResult::Forbidden) => {
                            let mut new_state = (*state).clone();
                            new_state.status = Status::Forbidden;
                            state.set(new_state);
                        }
                        Some(ShaarliImportApiResult::ShaarliError) => {
                            let mut new_state = (*state).clone();
                            new_state.status = Status::ShaarliError;
                            state.set(new_state);
                        }
                        _ => {
                            let mut new_state = (*state).clone();
                            new_state.status = Status::GenericError;
                            state.set(new_state);
                        }
                    }
                });
            }
        })
    };

    let oninput_base_url = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let api_url = AttrValue::from(input.value());
            let mut new_state = (*state).clone();
            new_state.api_url = api_url;
            state.set(new_state);
        })
    };

    let oninput_api_key = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let api_key = AttrValue::from(input.value());
            let mut new_state = (*state).clone();
            new_state.api_key = api_key;
            state.set(new_state);
        })
    };

    html! {
        <div class="centered-box">
            <h1 class="centered-box__title">{"Import from Shaarli API"}</h1>
            { match state.status {
                Status::Default | Status::Importing => html! {
                    <></>
                },
                Status::Forbidden => html! {
                    <div class="centered-box__error">
                        {"You don't have the right to create bookmarks"}
                    </div>
                },
                Status::ShaarliError => html! {
                    <div class="centered-box__error">
                        {"An error has occurred while contacting Shaarli"}
                    </div>
                },
                Status::GenericError => html! {
                    <div class="centered-box__error">
                        {"An error has occurred"}
                    </div>
                },
            }}
            <form {onsubmit}>
                <p>
                    <input
                        ref={base_url_input_ref}
                        type="text"
                        placeholder="base API url"
                        oninput={oninput_base_url}
                    />
                </p>
                <p>
                    <input
                        type="text"
                        placeholder="API key"
                        oninput={oninput_api_key}
                    />
                </p>
                <div class="centered-box__buttons">
                    <p>
                        <button type="submit" class={match state.status {
                            Status::Importing => "button--disabled".to_string(),
                            _ => "button--action".to_string(),
                        }}>
                            {"Import"}
                        </button>
                    </p>
                </div>
            </form>
        </div>
    }
}
