use crate::Route;
use gloo_net::http::Request;
use rest_api::URL_SESSIONS_CURRENT;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub onlogout: Callback<()>,
}

#[function_component(Logout)]
pub fn logout(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();

    let props = props.clone();
    use_effect(move || {
        let props = props.clone();
        spawn_local(async move {
            // TODO finish this (error cases)
            let result = Request::delete(&URL_SESSIONS_CURRENT).send().await;

            if let Ok(response) = result {
                if response.ok() {
                    props.onlogout.emit(());
                }
            }

            navigator.push(&Route::Index);
        })
    });

    html! {
        <div class="edit-bookmark">
            <h1 class="edit-bookmark__title">{"Logout...n"}</h1>
        </div>
    }
}
