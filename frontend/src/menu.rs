use crate::Route;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_router::Routable;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub username: Option<AttrValue>,
}

#[function_component(Menu)]
pub fn menu(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();

    html! {
        <div class="menu">
            <div class="menu__submenu">
                <a onclick={{let navigator = navigator.clone(); move |e:MouseEvent| {
                    navigator.push(&Route::Bookmarks);
                    e.prevent_default();
                }}} class="menu__item" href={Route::Bookmarks.to_path()}>{"Bookmarks"}</a>

                {"·"}

                <a onclick={{let navigator = navigator.clone(); move |e:MouseEvent| {
                    navigator.push(&Route::TagCloud);
                    e.prevent_default();
                }}} class="menu__item" href={Route::TagCloud.to_path()}>{"Tag cloud"}</a>

                {"·"}

                <a onclick={{let navigator = navigator.clone(); move |e:MouseEvent| {
                    navigator.push(&Route::AddBookmark);
                    e.prevent_default();
                }}} class="menu__item" href={Route::AddBookmark.to_path()}>{"Add"}</a>

                {"·"}

                <a onclick={{let navigator = navigator.clone(); move |e:MouseEvent| {
                    navigator.push(&Route::Tools);
                    e.prevent_default();
                }}} class="menu__item" href={Route::Tools.to_path()}>{"Tools"}</a>
            </div>
            <div class="menu__submenu--right">
                {if let Some(username)=&props.username {
                    html!{
                        <>
                            {username}
                            <a onclick={{let navigator = navigator.clone(); move |e:MouseEvent| {
                                navigator.push(&Route::Logout);
                                e.prevent_default();
                            }}} class="menu__item" href={Route::Logout.to_path()}>

                                <span class="material-icons-outlined">{"logout"}</span>
                            </a>
                        </>
                    }
                } else {
                    html! {
                        <a onclick={{let navigator = navigator.clone(); move |e:MouseEvent| {
                            navigator.push(&Route::Login);
                            e.prevent_default();
                        }}} class="menu__item" href={Route::Login.to_path()}>
                            <span class="material-icons-outlined">{"login"}</span>
                        </a>
                    }
                }}
            </div>
        </div>
    }
}
