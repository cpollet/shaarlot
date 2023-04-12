use super::data::Bookmark as BookmarkData;
use super::qr_code::QrCode;
use crate::Route;
use chrono::{DateTime, Local};
use rest_api::bookmarks::Access;
use std::rc::Rc;
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
        let props = props.clone();
        Callback::from(move |e: MouseEvent| {
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
                <ul class="bookmark__tags-list">
                {
                    props.bookmark.tags.as_slice().iter().map(|t| html! {
                        <li>{t}</li>
                    }).collect::<Html>()
                }
                </ul>
                <div class="bookmark__actions">
                    {display_date(&props.bookmark)}
                    {"\u{00a0}\u{ff5c}\u{00a0}"}
                    <a
                         class="material-icons-outlined md-16"
                        onclick={onclick_permalink}
                        href={Route::ViewBookmark {id: props.bookmark.id}.to_path()}
                    >
                        {"link"}
                    </a>
                    {"\u{00a0}\u{ff5c}\u{00a0}"}
                    <QrCode id={props.bookmark.id} />
                    { if props.bookmark.access == Access::Write {
                        html!{
                            <>
                                {"\u{00a0}\u{ff5c}\u{00a0}"}
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
                            </>
                        }
                    } else { html!{<></>} } }
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