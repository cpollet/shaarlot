use crate::Route;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_router::Routable;

#[function_component(Tools)]
pub fn tools() -> Html {
    let navigator = use_navigator().unwrap();

    html! {
        <div class="centered-box">
            <h1 class="centered-box__title">{"Tools"}</h1>
            <ul>
                <li>
                    <a
                        href={Route::ToolImportShaarliApi.to_path()}
                        onclick={
                            let navigator = navigator.clone();
                            Callback::from(move |e: MouseEvent| {
                                e.prevent_default();
                                navigator.push(&Route::ToolImportShaarliApi)
                            })
                        }
                    >
                        {"Import from Shaarli's API"}
                    </a>
                </li>
            </ul>
        </div>
    }
}
