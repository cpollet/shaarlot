use crate::bookmarks::bookmark::*;
use crate::data::Bookmark as BookmarkData;
use std::rc::Rc;
use yew::prelude::*;

mod bookmark;
pub mod bookmarks_provider;
mod qr_code;
mod qr_code_overlay;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub bookmarks: Rc<Vec<BookmarkData>>,
}

#[function_component(Bookmarks)]
pub fn bookmarks(props: &Props) -> Html {
    html! {
        <ul class="bookmarks">
        {
            props.bookmarks.as_slice().into_iter().map(|b| html! {
                <Bookmark key={b.id} bookmark={b.clone()} />
            }).collect::<Html>()
        }
        </ul>
    }
}

#[function_component(BookmarksHOC)]
pub fn bookmarks_hoc() -> Html {
    let bookmarks = use_context::<Props>().expect("no ctx found");
    html! {
        <Bookmarks ..bookmarks />
    }
}
