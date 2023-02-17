mod bookmark;
mod bookmarks;
mod bookmarks_provider;
mod qr_code;
mod qr_code_overlay;

use crate::bookmarks::BookmarksHOC;
use crate::bookmarks_provider::BookmarksProvider;
use yew::prelude::*;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <div class="view">
            <BookmarksProvider>
                <BookmarksHOC />
            </BookmarksProvider>
        </div>
    }
}
