use crate::bookmarks::Props as BookmarksContext;
use crate::data::Bookmark;
use gloo_net::http::Request;
use rest_api::{BookmarkResponse, URL_BOOKMARKS};
use std::rc::Rc;
use yew::platform::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Children,
}

#[derive(Clone, PartialEq)]
pub enum Order {
    Asc,
    Desc,
}

impl Order {
    fn query_param(&self) -> &str {
        match self {
            Order::Asc => "asc",
            Order::Desc => "desc",
        }
    }
}

#[derive(Clone)]
pub struct State {
    order: Order,
    context: Option<BookmarksContext>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            order: Order::Desc,
            context: None,
        }
    }
}

#[function_component(BookmarksProvider)]
pub fn bookmarks_provider(props: &Props) -> Html {
    let state = use_state(|| State::default());

    let on_change_order = {
        let state = state.clone();
        Callback::from(move |order: Order| {
            state.set(State {
                order,
                context: None,
            });
        })
    };

    {
        let state = state.clone();
        use_effect(move || {
            if state.context.is_none() {
                let state = state.clone();
                spawn_local(async move {
                    let res = fetch_bookmarks(&state.order, on_change_order).await;
                    // todo implement a 500 page
                    if let Ok(bookmarks) = res {
                        state.set(State {
                            order: state.order.clone(),
                            context: Some(bookmarks),
                        })
                    }
                });
            }

            || {}
        });
    }

    match state.context.as_ref() {
        Some(bookmarks) => html! {
            <ContextProvider<BookmarksContext> context={(*bookmarks).clone()}>
                { props.children.clone() }
            </ContextProvider<BookmarksContext >>
        },
        None => html! {
            <div></div>
        },
    }
}

async fn fetch_bookmarks(
    order: &Order,
    callback: Callback<Order>,
) -> Result<BookmarksContext, String> {
    match Request::get(URL_BOOKMARKS)
        .query([("order", format!("creation_date:{}", order.query_param()))])
        .send()
        .await
    {
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
                            .map(Bookmark::from)
                            .collect::<Vec<Bookmark>>()
                    })
                    .map(|bookmarks| BookmarksContext {
                        bookmarks: Rc::new(bookmarks),
                        order: order.clone(),
                        on_change_order: callback,
                    })
            }
        }
    }
}
