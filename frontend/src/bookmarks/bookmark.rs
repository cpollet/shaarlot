use crate::bookmarks::qr_code::QrCode;
use crate::data::Bookmark as BookmarkData;
use crate::Route;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_router::Routable;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub bookmark: BookmarkData,
}

#[function_component(Bookmark)]
pub fn bookmark(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();

    let onclick_delete = {
        let navigator = navigator.clone();
        let props = props.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            navigator.push(&Route::DeleteBookmark {
                id: props.bookmark.id,
            });
        })
    };

    let onclick_edit = {
        let navigator = navigator.clone();
        let props = props.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            navigator.push(&Route::EditBookmark {
                id: props.bookmark.id,
            });
        })
    };

    html! {
        <li class="bookmark">
            <div class="bookmark__title">
                <a href={props.bookmark.url.clone()}>
                    {props.bookmark.title.clone().unwrap_or_else(|| props.bookmark.url.clone())}
                </a>
            </div>
            <div class="bookmark__description">
                <div class="bookmark__tags-list">
                {"tag\u{00a0}·\u{00a0}other_tag"}
                </div>
                <div class="bookmark__actions">
                    <a
                        class="material-icons-outlined md-16 blue"
                        onclick={onclick_edit}
                        href={Route::EditBookmark {id: props.bookmark.id}.to_path()}
                    >
                        {"edit"}
                    </a>
                    {"\u{00a0}|\u{00a0}"}
                    <a
                        class="material-icons-outlined md-16 red"
                        onclick={onclick_delete}
                        href={Route::DeleteBookmark {id: props.bookmark.id}.to_path()}
                    >
                        {"delete"}
                    </a>
                    {"\u{00a0}|\u{00a0}"}
                    {"2023-02-15 21:37"}
                    {"\u{00a0}|\u{00a0}"}
                    <a href="#">{"permalink"}</a>
                    {"\u{00a0}|\u{00a0}"}
                    <QrCode id={props.bookmark.id} />
                </div>
                <div class="bookmark__link">
                    <a href={props.bookmark.url.clone()}>
                        {props.bookmark.url.clone()}
                    </a>
                </div>
            </div>
        </li>
    }
}
