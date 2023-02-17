use crate::bookmarks::bookmark::BookmarkProps;
use crate::bookmarks::BookmarksProps;
use gloo_net::http::Request;
use rest_api::{BookmarkResponse, URL_BOOKMARKS};
use std::rc::Rc;
use yew::platform::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct BookmarksProviderProps {
    pub children: Children,
}

#[function_component(BookmarksProvider)]
pub fn bookmarks_provider(props: &BookmarksProviderProps) -> Html {
    let bookmarks = use_state(|| None);

    {
        let bookmarks = bookmarks.clone();
        use_effect(move || {
            if bookmarks.is_none() {
                spawn_local(async move {
                    bookmarks.set(Some(match Request::get(URL_BOOKMARKS).send().await {
                        Err(_) => Err("Error fetching data".to_string()),
                        Ok(resp) => {
                            if !resp.ok() {
                                Err(format!(
                                    "Error fetching data: {} ({})",
                                    resp.status(),
                                    resp.status_text()
                                ))
                            } else {
                                resp.json::<Vec<BookmarkResponse>>()
                                    .await
                                    .map_err(|err| err.to_string())
                                    .map(|elements| {
                                        elements
                                            .into_iter()
                                            .map(BookmarkProps::from)
                                            .collect::<Vec<BookmarkProps>>()
                                    })
                                    .map(|bookmarks| BookmarksProps {
                                        bookmarks: Rc::new(bookmarks),
                                    })
                            }
                        }
                    }))
                });
            }

            || {}
        });
    }

    match bookmarks.as_ref() {
        Some(Ok(bookmarks)) => html! {
            <ContextProvider<BookmarksProps> context={(*bookmarks).clone()}>
                { props.children.clone() }
            </ContextProvider<BookmarksProps>>
        },
        Some(Err(err)) => html! {
            <div>{err}</div>
        },
        None => html! {
            <div>{"No data"}</div>
        },
    }
}
