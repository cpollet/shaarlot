use crate::bookmarks::Props as BookmarksContext;
use crate::data::Bookmark;
use gloo_net::http::Request;
use rest_api::bookmarks::get_many::GetBookmarksResult;
use rest_api::bookmarks::URL_BOOKMARKS;
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
            // todo check why it calls the endpoint twice!
            if state.context.is_none() {
                let state = state.clone();
                spawn_local(async move {
                    let bookmarks = fetch_bookmarks(&state.order, on_change_order).await;
                    state.set(State {
                        order: state.order.clone(),
                        context: Some(bookmarks),
                    })
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

async fn fetch_bookmarks(order: &Order, callback: Callback<Order>) -> BookmarksContext {
    let bookmarks = match GetBookmarksResult::from(
        Request::get(URL_BOOKMARKS)
            .query([("order", format!("creation_date:{}", order.query_param()))])
            .send()
            .await,
    )
    .await
    {
        Some(GetBookmarksResult::Success(bookmarks)) => bookmarks
            .into_iter()
            .map(Bookmark::from)
            .collect::<Vec<Bookmark>>(),
        _ => {
            // todo handle errors
            vec![]
        }
    };

    BookmarksContext {
        bookmarks: Rc::new(bookmarks),
        order: order.clone(),
        on_change_order: callback,
    }
}
