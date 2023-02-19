use crate::bookmarks::qr_code_overlay::QrCodeOverlay;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: i32,
}

struct State {
    overlay_open: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            overlay_open: false,
        }
    }
}

#[function_component(QrCode)]
pub fn qr_code(props: &Props) -> Html {
    let state = use_state(|| State::default());

    let onclick = {
        let state = state.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            state.set(State { overlay_open: true })
        })
    };

    let close = {
        let state = state.clone();
        Callback::from(move |_| {
            state.set(State {
                overlay_open: false,
            })
        })
    };

    html! {
        <>
            <a
                class="material-icons-outlined md-18"
                {onclick}
                href="#"
            >
                {"qr_code"}
            </a>
            if state.overlay_open {
                <QrCodeOverlay
                    id={props.id}
                    onclick={close}
                />
            }
        </>
    }
}
