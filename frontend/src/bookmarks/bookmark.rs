use std::rc::Rc;
use chrono::{DateTime, Local};
use crate::bookmarks::qr_code::QrCode;
use crate::data::Bookmark as BookmarkData;
use crate::Route;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_router::Routable;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub bookmark: Rc<BookmarkData>,
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

    let onclick_permalink = {
        let navigator = navigator.clone();
        let props = props.clone();
        Callback::from(move |e:MouseEvent| {
            e.prevent_default();
            navigator.push(&Route::ViewBookmark {
                id: props.bookmark.id,
            })
        })
    };

    html! {
        <li class="bookmark">
            <div class="bookmark__title">
                <a href={props.bookmark.url.clone()}>
                    <span class="material-icons-outlined bookmark__title-icon">{"open_in_new"}</span> {props.bookmark.title.clone().unwrap_or_else(|| props.bookmark.url.clone())}
                </a>
            </div>
            { props.bookmark.description.as_ref().map(|d| html! {
                <div class="bookmark__description">{d.clone()}</div>
            })}
            <div class="bookmark__footer">
                <div class="bookmark__tags-list">
                {"todo\u{00a0}·\u{00a0}tags"}
                </div>
                <div class="bookmark__actions">
                    <a
                        class="material-icons-outlined md-16 blue"
                        onclick={onclick_edit}
                        href={Route::EditBookmark {id: props.bookmark.id}.to_path()}
                    >
                        {"edit"}
                    </a>
                    {"\u{00a0}\u{ff5c}\u{00a0}"}
                    <a
                        class="material-icons-outlined md-16 red"
                        onclick={onclick_delete}
                        href={Route::DeleteBookmark {id: props.bookmark.id}.to_path()}
                    >
                        {"delete"}
                    </a>
                    {"\u{00a0}\u{ff5c}\u{00a0}"}
                    {display_date(&props.bookmark)}
                    // {props.bookmark.creation_date.format("%h %e, %Y at %T %Z")}
                    {"\u{00a0}\u{ff5c}\u{00a0}"}
                    <a
                        onclick={onclick_permalink}
                        href={Route::ViewBookmark {id: props.bookmark.id}.to_path()}
                    >
                        {"permalink"}
                    </a>
                    {"\u{00a0}\u{ff5c}\u{00a0}"}
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

fn display_date(bookmark: &BookmarkData) -> Html {
    fn display(date: &DateTime<Local>) -> String {
            date.format("%h %e, %Y at %T %Z").to_string()
    }
    match bookmark.update_date {
        None => {
            html! {
              display(&bookmark.creation_date)
                }
        }
        Some(date) => {
            html! {
                <span class="bookmark__date--updated" title={format!("updated: {}", display(&date))}>
                     {display(&bookmark.creation_date)}{" *"}
                </span>
            }
        }
    }
}

#[function_component(BookmarkHOC)]
pub fn bookmark_hoc() -> Html {
    let bookmark = use_context::<Rc<BookmarkData>>().expect("no ctx found");
    let navigator =use_navigator().unwrap();

    let onclick = Callback::from(move |e:MouseEvent| {
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
                    <Bookmark bookmark={bookmark.clone()} />
                </li>
            </ul>
        </div>
    }
}