mod bookmark;
mod bookmarks;
mod bookmarks_provider;

use yew::prelude::*;
use crate::bookmarks_provider::BookmarksProvider;
use crate::bookmarks::BookmarksHOC;

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
