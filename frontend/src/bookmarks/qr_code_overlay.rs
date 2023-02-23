use rest_api::bookmarks::URL_BOOKMARK_QRCODE;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: i32,
    pub onclick: Callback<()>,
}

#[function_component(QrCodeOverlay)]
pub fn qr_code_overlay(props: &Props) -> Html {
    let onclick = {
        let callback = props.onclick.clone();
        Callback::from(move |_| {
            callback.emit(());
        })
    };

    html! {
        <div {onclick} class="overlay">
            <div class="overlay__content--center-middle">
                <div>
                    <img src={URL_BOOKMARK_QRCODE.replace(":id", &props.id.to_string())} />
                    <div class="qrcode__legend">
                        {"click to close"}
                    </div>
                </div>
            </div>
        </div>
    }
}
