use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub page: u64,
    pub page_count: u64,
    pub on_previous: Callback<()>,
    pub on_next: Callback<()>,
}

#[function_component(Nav)]
pub fn nav(props: &Props) -> Html {
    let on_previous = {
        let on_previous = props.on_previous.clone();
        Callback::from(move |_| on_previous.emit(()))
    };
    let on_next = {
        let on_next = props.on_next.clone();
        Callback::from(move |_| on_next.emit(()))
    };

    html! {
        <div class="bookmarks__nav">
            <span class="material-icons-outlined md-18 bookmarks__nav--prev" onclick={on_previous}>
             {"navigate_before"}
            </span>
            {props.page}{" / "}{props.page_count}
            <span class="material-icons-outlined md-18 bookmarks__nav--next" onclick={on_next}>
                {"navigate_next"}
            </span>
        </div>
    }
}
