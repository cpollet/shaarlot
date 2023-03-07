use crate::Route;
use gloo_net::http::Request;
use rest_api::sessions::URL_SESSIONS_CURRENT;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_hooks::use_effect_once;
use yew_router::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub onlogout: Callback<()>,
}

#[function_component(Logout)]
pub fn logout(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();

    let props = props.clone();
    use_effect_once(move || {
        let props = props.clone();
        spawn_local(async move {
            // TODO finish this (error cases)
            let result = Request::delete(URL_SESSIONS_CURRENT).send().await;

            if let Ok(response) = result {
                if response.ok() {
                    props.onlogout.emit(());
                }
            }

            navigator.push(&Route::Index);
        });

        || {}
    });

    html! {
        <div class="centered-box">
            <h1 class="centered-box__title">{"Logout...n"}</h1>
        </div>
    }
}
