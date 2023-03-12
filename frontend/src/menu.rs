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
    let menu_expanded = use_state(|| false);

    html! {
        <div class="menu">
            <div class="menu__left-submenu-trigger">
                <a class="menu__item" href="#" onclick={{
                    let menu_expanded = menu_expanded.clone();
                    move |e:MouseEvent| {
                        e.prevent_default();
                        let new_menu_expanded = !(*menu_expanded);
                        menu_expanded.set(new_menu_expanded);
                    }
                }}>
                    <span class="material-icons-outlined">{"menu"}</span>
                </a>
            </div>
            <ul class={if *menu_expanded { "menu__left-submenu--expanded" } else { "menu__left-submenu" }}>
                <li>
                    <a onclick={{
                        let navigator = navigator.clone();
                        let menu_expanded = menu_expanded.clone();
                        move |e:MouseEvent| {
                            e.prevent_default();
                            menu_expanded.set(false);
                            navigator.push(&Route::Bookmarks);
                        }
                    }} class="menu__item" href={Route::Bookmarks.to_path()}>{"Bookmarks"}</a>
                </li>
                <li>
                    <a onclick={{
                        let navigator = navigator.clone();
                        let menu_expanded = menu_expanded.clone();
                        move |e:MouseEvent| {
                            e.prevent_default();
                            menu_expanded.set(false);
                            navigator.push(&Route::TagCloud);
                        }
                    }} class="menu__item" href={Route::TagCloud.to_path()}>{"Tag cloud"}</a>
                </li>
                <li>
                    <a onclick={{
                        let navigator = navigator.clone();
                        let menu_expanded = menu_expanded.clone();
                        move |e:MouseEvent| {
                            e.prevent_default();
                            menu_expanded.set(false);
                            navigator.push(&Route::AddBookmark);
                        }
                    }} class="menu__item" href={Route::AddBookmark.to_path()}>{"Add"}</a>
                </li>
                <li>
                    <a onclick={{
                        let navigator = navigator.clone();
                        let menu_expanded = menu_expanded.clone();
                        move |e:MouseEvent| {
                            e.prevent_default();
                            menu_expanded.set(false);
                            navigator.push(&Route::Tools);
                        }
                    }} class="menu__item" href={Route::Tools.to_path()}>{"Tools"}</a>
                </li>
            </ul>
            <div class="menu__right-submenu">
                {if let Some(_username) = &props.username {
                    html!{
                        <>
                            <a onclick={{let navigator = navigator.clone(); move |e:MouseEvent| {
                                navigator.push(&Route::Profile);
                                e.prevent_default();
                            }}} class="menu__item" href={Route::Profile.to_path()}>
                                <span class="material-icons-outlined">{"account_circle"}</span>
                            </a>
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
