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
            <PageSizeItem
                size={20}
                selected={props.page_size == 20}
                onclick={props.on_change_page_size.clone()}
            />
            <PageSizeItem
                size={50}
                selected={props.page_size == 50}
                onclick={props.on_change_page_size.clone()}
            />
            <PageSizeItem
                size={100}
                selected={props.page_size == 100}
                onclick={props.on_change_page_size.clone()}
            />
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct ItemProps {
    selected: bool,
    size: u64,
    onclick: Callback<u64>,
}

#[function_component(PageSizeItem)]
fn page_size_item(props: &ItemProps) -> Html {
    let onclick = {
        let props = props.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            props.onclick.emit(props.size);
        })
    };

    html! {
        <a
            href="#"
            class={if props.selected {
                "bookmarks__page-size-item--selected"
            } else {
                "bookmarks__page-size-item"
            } }
            {onclick}
        >
            {props.size}
        </a>
    }
}