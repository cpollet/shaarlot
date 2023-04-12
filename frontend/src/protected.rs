use crate::Route;
use yew::prelude::*;
use yew_router::hooks::use_navigator;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub logged_in: bool,
    pub children: Children,
}

#[function_component(Protected)]
pub fn protected(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();

    if props.logged_in {
        html! {
            <>{ props.children.clone() }</>
        }
    } else {
        navigator.push(&Route::Login);
        html! {
            <>{"redirect to login page"}</>
        }
    }
}
