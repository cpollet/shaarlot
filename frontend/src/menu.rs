use crate::Route;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_router::Routable;

#[function_component(Menu)]
pub fn menu() -> Html {
    let navigator = use_navigator().unwrap();

    html! {
        <div class="menu">
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
    }
}
