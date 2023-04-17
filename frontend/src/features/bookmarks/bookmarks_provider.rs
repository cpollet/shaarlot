use super::data::Bookmark;
use super::pages::bookmarks::Props as BookmarksContext;
use gloo_net::http::Request;
use rest_api::bookmarks::get_many::GetBookmarksResult;
use rest_api::bookmarks::URL_BOOKMARKS;
use std::borrow::Cow;
use std::rc::Rc;
use urlencoding::encode;
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

impl Default for Order {
    fn default() -> Self {
        Order::Desc
    }
}

impl Order {
    fn query_param(&self) -> &str {
        match self {
            Order::Asc => "asc",
            Order::Desc => "desc",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct State {
    order: Order,
    page: u64,
    pages_count: u64,
    page_size: u64,
    tags: Rc<Vec<AttrValue>>,
    bookmarks: Option<Rc<Vec<Bookmark>>>,
    loading: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            order: Order::default(),
            page: 0,
            pages_count: 0,
            page_size: 20,
            tags: Rc::new(Vec::new()),
            bookmarks: None,
            loading: false,
        }
    }
}

#[function_component(BookmarksProvider)]
pub fn bookmarks_provider(props: &Props) -> Html {
    let state = use_state(State::default);

    let on_change_order = {
        let state = state.clone();
        Callback::from(move |order: Order| {
            let mut new_state = (*state).clone();
            new_state.order = order;
            new_state.page = 0;
            new_state.loading = false;
            new_state.bookmarks = None;
            state.set(new_state);
        })
    };

    let on_previous = {
        let state = state.clone();
        Callback::from(move |_| {
            if state.loading {
                return;
            }
            let mut new_state = (*state).clone();
            new_state.page = state.page.checked_sub(1).unwrap_or_default();
            new_state.loading = false;
            new_state.bookmarks = None;
            state.set(new_state);
        })
    };

    let on_next = {
        let state = state.clone();
        Callback::from(move |_| {
            if state.loading {
                return;
            }
            let mut new_state = (*state).clone();
            new_state.page = state
                .pages_count
                .checked_sub(1)
                .unwrap_or_default()
                .min(state.page + 1);
            new_state.loading = false;
            new_state.bookmarks = None;
            state.set(new_state);
        })
    };

    let on_change_page_size = {
        let state = state.clone();
        Callback::from(move |page_size: u64| {
            if state.loading {
                return;
            }
            let mut new_state = (*state).clone();
            new_state.page = u64::default();
            new_state.page_size = page_size;
            new_state.loading = false;
            new_state.bookmarks = None;
            state.set(new_state);
        })
    };

    let on_select_tag_filter = {
        let state = state.clone();
        Callback::from(move |t: AttrValue| {
            if !state.tags.contains(&t) {
                let mut new_tags = (*state.tags).clone();
                new_tags.push(t.clone());

                let mut new_state = (*state).clone();
                new_state.tags = Rc::new(new_tags);
                new_state.bookmarks = None;

                state.set(new_state);
            }
        })
    };

    let on_change_tags = {
        let state = state.clone();
        Callback::from(move |tags| {
            let mut new_state = (*state).clone();
            new_state.tags = Rc::new(tags);
            new_state.bookmarks = None;

            state.set(new_state);
        })
    };

    {
        let state = state.clone();
        use_effect(move || {
            if state.bookmarks.is_none() && !state.loading {
                let state = state.clone();

                {
                    let mut new_state = (*state).clone();
                    new_state.loading = true;
                    state.set(new_state);
                }

                spawn_local(async move {
                    let bookmarks = fetch_bookmarks(&state).await;
                    let mut new_state = (*state).clone();
                    new_state.bookmarks = Some(Rc::new(bookmarks.0));
                    new_state.pages_count = bookmarks.1;
                    new_state.loading = false;
                    state.set(new_state);
                });
            }
        });
    }

    match state.bookmarks.as_ref() {
        Some(bookmarks) => {
            let context = BookmarksContext {
                bookmarks: bookmarks.clone(),
                order: state.order.clone(),
                page: state.page,
                page_count: state.pages_count,
                page_size: state.page_size,
                selected_tags: state.tags.clone(),
                on_change_order,
                on_previous,
                on_next,
                on_change_page_size,
                on_select_tag_filter,
                on_change_tags,
            };
            html! {
                <ContextProvider<BookmarksContext> {context}>
                    { props.children.clone() }
                </ContextProvider<BookmarksContext >>
            }
        }
        None => html! {
            <div></div>
        },
    }
}

async fn fetch_bookmarks(state: &State) -> (Vec<Bookmark>, u64) {
    let params = vec![
        (
            "order",
            format!("creation_date:{}", state.order.query_param()),
        ),
        ("page", format!("{}", state.page)),
        ("count", state.page_size.to_string()),
        (
            "tags",
            state
                .tags
                .iter()
                .map(|tag| encode(tag.as_str()))
                .reduce(|a, b| Cow::Owned(format!("{}+{}", a, b)))
                .unwrap_or_default()
                .to_string(),
        ),
    ];

    match GetBookmarksResult::from(Request::get(URL_BOOKMARKS).query(params).send().await).await {
        Some(GetBookmarksResult::Success(response)) => (
            response
                .bookmarks
                .into_iter()
                .map(Bookmark::from)
                .collect::<Vec<Bookmark>>(),
            response.pages_count,
        ),
        _ => {
            // todo handle errors
            (vec![], 0)
        }
    }
}
