use super::data::Bookmark;
use super::pages::bookmarks::Props as BookmarksContext;
use crate::features::bookmarks::bookmarks_query::search;
use crate::Route;
use gloo_net::http::Request;
use rest_api::bookmarks::get_many::GetBookmarksResult;
use rest_api::bookmarks::{
    GetBookmarksStatsResponse, GetBookmarksStatsResult, URL_BOOKMARKS, URL_BOOKMARKS_STATS,
};
use std::borrow::Cow;
use std::rc::Rc;
use urlencoding::encode;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub children: Children,
    pub params: Option<Params>,
    pub on_change: Option<Callback<(Route, Params)>>,
}

#[derive(Clone, PartialEq, Default)]
pub struct Params {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub tags: Option<Vec<AttrValue>>,
    pub order: Option<Order>,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Order {
    #[default]
    CreationDateDesc,
    CreationDateAsc,
}

impl Order {
    fn query_param(&self) -> &str {
        match self {
            Order::CreationDateAsc => "creation_date:asc",
            Order::CreationDateDesc => "creation_date:desc",
        }
    }
}

#[derive(Clone, PartialEq, Default)]
struct State {
    params: StateParams,
    pages_count: u64,
    bookmarks: Option<Rc<Vec<Bookmark>>>,
    loading: bool,
    stats: Option<GetBookmarksStatsResponse>,
}

#[derive(Clone, PartialEq)]
struct StateParams {
    order: Order,
    page: u64,
    page_size: u64,
    tags: Rc<Vec<AttrValue>>,
}

impl Default for StateParams {
    fn default() -> Self {
        Self {
            order: Order::default(),
            page: 0,
            page_size: 20,
            tags: Rc::new(Vec::new()),
        }
    }
}

impl StateParams {
    fn is_default(&self) -> bool {
        self.order == Order::default()
            && self.page == 0
            && self.page_size == 20
            && self.tags.is_empty()
    }
}

impl From<&StateParams> for Params {
    fn from(params: &StateParams) -> Self {
        Self {
            page: match params.page {
                0 => None,
                x => Some(x),
            },
            page_size: match params.page_size {
                20 => None,
                x => Some(x),
            },
            tags: if params.tags.is_empty() {
                None
            } else {
                Some((*params.tags).clone())
            },
            order: match params.order {
                Order::CreationDateDesc => None,
                Order::CreationDateAsc => Some(params.order),
            },
        }
    }
}

impl From<Option<&Params>> for StateParams {
    fn from(value: Option<&Params>) -> Self {
        let mut state_params = StateParams::default();

        if value.is_none() {
            return state_params;
        }

        let value = value.unwrap();

        if let Some(page) = value.page {
            state_params.page = page;
        }
        if let Some(page_size) = value.page_size {
            state_params.page_size = page_size;
        }
        if let Some(tags) = &value.tags {
            if !tags.is_empty() {
                state_params.tags = Rc::new(tags.clone());
            }
        }
        if let Some(order) = value.order {
            state_params.order = order;
        }

        state_params
    }
}

#[function_component(BookmarksProvider)]
pub fn bookmarks_provider(props: &Props) -> Html {
    let state = use_state(|| State {
        params: StateParams::from(props.params.as_ref()),
        ..Default::default()
    });

    let on_change_order = {
        let state = state.clone();
        Callback::from(move |order: Order| {
            let mut new_state = (*state).clone();
            new_state.params.order = order;
            new_state.params.page = 0;
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
            new_state.params.page = state.params.page.checked_sub(1).unwrap_or_default();
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
            new_state.params.page = state
                .pages_count
                .checked_sub(1)
                .unwrap_or_default()
                .min(state.params.page + 1);
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
            new_state.params.page = u64::default();
            new_state.params.page_size = page_size;
            new_state.loading = false;
            new_state.bookmarks = None;
            state.set(new_state);
        })
    };

    let on_select_tag_filter = {
        let state = state.clone();
        Callback::from(move |t: AttrValue| {
            if !state.params.tags.contains(&t) {
                let mut new_tags = (*state.params.tags).clone();
                new_tags.push(t);

                let mut new_state = (*state).clone();
                new_state.params.tags = Rc::new(new_tags);
                new_state.bookmarks = None;

                state.set(new_state);
            }
        })
    };

    let on_change_tags = {
        let state = state.clone();
        Callback::from(move |tags| {
            let mut new_state = (*state).clone();
            new_state.params.tags = Rc::new(tags);
            new_state.loading = false;
            new_state.bookmarks = None;

            state.set(new_state);
        })
    };

    {
        let state = state.clone();
        let props = props.clone();
        let navigator = use_navigator().unwrap();
        let location = use_location().unwrap();
        use_effect(move || {
            if state.stats.is_none() {
                let state = state.clone();
                spawn_local(async move {
                    let mut new_state = (*state).clone();
                    new_state.stats = Some(fetch_stats().await);
                    state.set(new_state);
                });
            }
            if state.bookmarks.is_none() && !state.loading {
                {
                    let mut new_state = (*state).clone();
                    new_state.loading = true;
                    state.set(new_state);
                }

                // this is not beautiful but it works
                if state.params.is_default() {
                    if location.path() != &Route::Bookmarks.to_path() {
                        navigator.push(&Route::Bookmarks);
                    }
                } else if let Some(callback) = props.on_change {
                    callback.emit((Route::BookmarksSearch, Params::from(&state.params)));
                } else {
                    search(
                        &navigator,
                        Route::BookmarksSearch,
                        Params::from(&state.params),
                        &location,
                        None,
                    );
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
                tags: Rc::new(Vec::new()),
                order: state.params.order,
                page: state.params.page,
                page_size: state.params.page_size,
                selected_tags: state.params.tags.clone(),
                page_count: state.pages_count,
                on_change_order,
                on_previous,
                on_next,
                on_change_page_size,
                on_select_tag_filter,
                on_change_tags,
                links: state
                    .stats
                    .as_ref()
                    .map(|s| s.count_total)
                    .unwrap_or_default(),
                private_links: state
                    .stats
                    .as_ref()
                    .map(|s| s.count_private)
                    .unwrap_or_default(),
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
        ("order", state.params.order.query_param().to_string()),
        ("page", state.params.page.to_string()),
        ("count", state.params.page_size.to_string()),
        (
            "tags",
            state
                .params
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

async fn fetch_stats() -> GetBookmarksStatsResponse {
    match GetBookmarksStatsResult::from(Request::get(URL_BOOKMARKS_STATS).send().await).await {
        Some(GetBookmarksStatsResult::Success(response)) => response,
        _ => {
            // todo handle errors
            GetBookmarksStatsResponse {
                count_total: 0,
                count_private: 0,
            }
        }
    }
}
