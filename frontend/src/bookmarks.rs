use crate::bookmark::*;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct BookmarksProps {
    pub bookmarks: Rc<Vec<BookmarkProps>>,
}

#[function_component(Bookmarks)]
pub fn bookmarks(props: &BookmarksProps) -> Html {
    html! {
        <ul class="bookmarks">
        {
            props.bookmarks.as_slice().into_iter().map(|b| html! {
                <Bookmark key={b.id} ..b.clone() />
            }).collect::<Html>()
        }
        </ul>
    }
}

#[function_component(BookmarksHOC)]
pub fn bookmarks_hoc() -> Html {
    let bookmarks = use_context::<BookmarksProps>().expect("no ctx found");
    html! {
        <Bookmarks ..bookmarks />
    }
}
