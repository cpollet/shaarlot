use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub page_size: u64,
    pub on_change_page_size: Callback<u64>,
}

#[function_component(PageSize)]
pub fn page_size(props: &Props) -> Html {
    html! {
        <div class="bookmarks__page-size">
            {"links per page:\u{00a0}"}
            <a
                href="#"
                class={match props.page_size {
                    20 => "bookmarks__page-size-item--selected",
                    _ => "bookmarks__page-size-item"
                }}
                onclick={
                    let on_change_page_size = props.on_change_page_size.clone();
                    Callback::from(move |_| on_change_page_size.emit(20))
                }
            >
                {"20"}
            </a>
            <a
                href="#"
                class={match props.page_size {
                    50 => "bookmarks__page-size-item--selected",
                    _ => "bookmarks__page-size-item"
                }}
                onclick={
                    let on_change_page_size = props.on_change_page_size.clone();
                    Callback::from(move |_| on_change_page_size.emit(50))
                }
            >
                {"50"}
            </a>
            <a
                href="#"
                class={match props.page_size {
                    100 => "bookmarks__page-size-item--selected",
                    _ => "bookmarks__page-size-item"
                }}
                onclick={
                    let on_change_page_size = props.on_change_page_size.clone();
                    Callback::from(move |_| on_change_page_size.emit(100))
                }
            >
                {"100"}
            </a>
        </div>
    }
}
