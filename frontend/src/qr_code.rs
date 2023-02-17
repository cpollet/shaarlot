use crate::qr_code_overlay::QrCodeOverlay;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct QrCodeProps {
    pub id: i32,
}

struct QrCodeState {
    overlay_open: bool,
}

impl Default for QrCodeState {
    fn default() -> Self {
        Self {
            overlay_open: false,
        }
    }
}

#[function_component(QrCode)]
pub fn qr_code(props: &QrCodeProps) -> Html {
    let state = use_state(|| QrCodeState::default());

    let onclick = {
        let state = state.clone();
        Callback::from(move |_| state.set(QrCodeState { overlay_open: true }))
    };

    let close = {
        let state = state.clone();
        Callback::from(move |_| {
            state.set(QrCodeState {
                overlay_open: false,
            })
        })
    };

    html! {
        <>
            <img {onclick} class="bookmarks__qrcode-icon" src="/assets/qrcode_16.png" />
            if state.overlay_open {
                <QrCodeOverlay
                    id={props.id}
                    onclick={close}
                />
            }
        </>
    }
}
