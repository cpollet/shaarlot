mod bookmarks;
mod create_bookmark;
mod data;
mod delete_bookmark;
mod edit_bookmark;
mod menu;

use crate::bookmarks::bookmarks_provider::BookmarksProvider;
use crate::bookmarks::BookmarksHOC;
use crate::create_bookmark::CreateBookmark;
use crate::delete_bookmark::DeleteBookmark;
use crate::edit_bookmark::EditBookmark;
use crate::menu::Menu;
use yew::prelude::*;
use yew_router::prelude::*;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Index,

    #[at("/bookmarks")]
    Bookmarks,

    #[at("/bookmarks/~add")]
    AddBookmark,

    #[at("/bookmarks/:id/~delete")]
    DeleteBookmark { id: i32 },

    #[at("/bookmarks/:id/~edit")]
    EditBookmark { id: i32 },

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
        Route::Index | Route::Bookmarks => {
            html! {
                <BookmarksProvider>
                    <BookmarksHOC />
                </BookmarksProvider>
            }
        }
        Route::AddBookmark => {
            html! { <CreateBookmark /> }
        }
        Route::DeleteBookmark { id } => {
            html! {
                <DeleteBookmark {id} />
            }
        }
        Route::EditBookmark { id } => {
            html! {
                <EditBookmark {id} />
            }
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
