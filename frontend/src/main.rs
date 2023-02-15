mod bookmark;
mod bookmarks;
mod bookmarks_provider;

use yew::prelude::*;
use crate::bookmarks_provider::BookmarksProvider;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BookmarksProvider />
    }
}
