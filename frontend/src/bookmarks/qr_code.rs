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
        Callback::from(move |_| state.set(State { overlay_open: true }))
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
            <img {onclick} class="bookmark__icon" src="/assets/qrcode_16.png" />
            if state.overlay_open {
                <QrCodeOverlay
                    id={props.id}
                    onclick={close}
                />
            }
        </>
    }
}
