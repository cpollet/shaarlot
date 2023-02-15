use std::rc::Rc;
use yew::html::IntoPropValue;
use yew::prelude::*;
use crate::bookmark::*;

#[derive(Properties, Clone, PartialEq)]
pub struct BookmarksProps {
    pub bookmarks: Rc<Vec<BookmarkProps>>,
}

impl IntoPropValue<Rc<Vec<BookmarkProps>>> for BookmarksProps {
    fn into_prop_value(self) -> Rc<Vec<BookmarkProps>> {
        self.bookmarks.clone()
    }
}

#[function_component(Bookmarks)]
pub fn bookmarks(props: &BookmarksProps) -> Html {
    html! {
        <ul>
        {
            props.bookmarks.as_slice().into_iter().map(|b| html! {
                <Bookmark ..b.clone() />
            }).collect::<Html>()
        }
        </ul>
    }
}
