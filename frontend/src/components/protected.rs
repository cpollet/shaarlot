use crate::features::authentication::pages::login::QueryParams;
use crate::Route;
use yew::prelude::*;
use yew_router::hooks::{use_navigator, use_route};
use yew_router::Routable;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub logged_in: bool,
    pub children: Children,
}

#[function_component(Protected)]
pub fn protected(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();
    let route = use_route::<Route>().unwrap();

    if props.logged_in {
        html! {
            <>{ props.children.clone() }</>
        }
    } else {
        let _ = navigator.push_with_query(
            &Route::Login,
            &QueryParams {
                redirect_to: Some(route.to_path()),
            },
        );
        html! { <></> }
    }
}
