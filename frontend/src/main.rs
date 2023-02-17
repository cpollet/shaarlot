mod bookmarks;
mod create_bookmark;
mod menu;

use crate::bookmarks::bookmarks_provider::BookmarksProvider;
use crate::bookmarks::BookmarksHOC;
use crate::create_bookmark::CreateBookmark;
use crate::menu::Menu;
use yew::prelude::*;
use yew_router::prelude::*;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Index,
    #[at("/bookmarks")]
    Bookmarks,
    #[at("/bookmarks/new")]
    AddBookmark,
    #[at("/tags")]
    TagCloud,
    #[at("/tools")]
    Tools,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Menu />
            <div class="content">
                <Switch<Route> render={switch} />
            </div>
        </BrowserRouter>
    }
}

fn switch(route: Route) -> Html {
    match route {
        Route::Index | Route::Bookmarks => html! {
            <BookmarksProvider>
                <BookmarksHOC />
            </BookmarksProvider>
        },
        Route::AddBookmark => {
            html! { <CreateBookmark /> }
        }
        Route::TagCloud => {
            html! {
                {"todo: tag cloud"}
            }
        }
        Route::Tools => {
            html! {
                {"todo: tools"}
            }
        }
        Route::NotFound => {
            html! {
                <h1>{"404 Not Found"}</h1>
            }
        }
    }
}
