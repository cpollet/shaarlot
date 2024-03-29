use super::super::super::super::data::Bookmark as BookmarkData;
use super::super::bookmark::Bookmark;
use crate::Route;
use std::rc::Rc;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_router::Routable;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub bookmark: Rc<BookmarkData>,
}

#[function_component(ViewBookmark)]
pub fn view_bookmark(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();

    let onclick = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        navigator.push(&Route::Bookmarks);
    });

    html! {
        <div>
            <div class="bookmarks-header">
                <a {onclick} class="material-icons-outlined md-18" href={Route::Bookmarks.to_path()} >{"home"}</a>
            </div>
            <ul class="bookmarks">
                <li>
                    <Bookmark bookmark={props.bookmark.clone()} on_select_tag_filter={Callback::from(move |_| {})} />
                </li>
            </ul>
        </div>
    }
}

#[function_component(ViewBookmarkHOC)]
pub fn view_bookmark_hoc() -> Html {
    let bookmark = use_context::<Rc<BookmarkData>>().expect("no ctx found");

    html! {
        <ViewBookmark {bookmark} />
    }
}
