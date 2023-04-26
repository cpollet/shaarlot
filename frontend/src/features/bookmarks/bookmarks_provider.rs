use super::super::super::data::Bookmark;
use super::pages::bookmarks::Props as BookmarksContext;
use crate::eventually::Eventually;
use crate::Route;
use gloo_net::http::Request;
use rest_api::bookmarks::get_many::GetBookmarksResult;
use rest_api::bookmarks::{
    GetBookmarksStatsResponse, GetBookmarksStatsResult, URL_BOOKMARKS, URL_BOOKMARKS_STATS,
};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use urlencoding::encode;
use yew::platform::spawn_local;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub children: Children,
    pub params: Option<Params>,
    pub on_change: Option<Callback<(Route, Params)>>,
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Params {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub tags: Option<Vec<AttrValue>>,
    pub order: Option<Order>,
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
pub struct State {
    order: Order,
    page: u64,
    page_size: u64,
    tags: Rc<Vec<AttrValue>>,
    data: Eventually<Data>,
    stats: Eventually<Stats>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            order: Order::default(),
            page: 0,
            page_size: 20,
            tags: Rc::new(Vec::new()),
            data: Eventually::None,
            stats: Eventually::None,
        }
    }
}

impl From<&State> for Params {
    fn from(state: &State) -> Self {
        Self {
            page: match state.page {
                0 => None,
                x => Some(x),
            },
            page_size: match state.page_size {
                20 => None,
                x => Some(x),
            },
            tags: if state.tags.is_empty() {
                None
            } else {
                Some((*state.tags).clone())
            },
            order: match state.order {
                Order::CreationDateDesc => None,
                Order::CreationDateAsc => Some(state.order),
            },
        }
    }
}

impl From<Option<&Params>> for State {
    fn from(value: Option<&Params>) -> Self {
        let mut state = State::default();

        if value.is_none() {
            return state;
        }

        let value = value.unwrap();

        if let Some(page) = value.page {
            state.page = page;
        }
        if let Some(page_size) = value.page_size {
            state.page_size = page_size;
        }
        if let Some(tags) = &value.tags {
            if !tags.is_empty() {
                state.tags = Rc::new(tags.clone());
            }
        }
        if let Some(order) = value.order {
            state.order = order;
        }

        state
    }
}

#[derive(Clone, PartialEq)]
struct Data {
    pages_count: u64,
    bookmarks: Rc<Vec<Bookmark>>,
}

impl Debug for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Data {{ pages_count: {}, bookmarks.len(): {} }}",
            self.pages_count,
            self.bookmarks.len()
        )
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
struct Stats {
    count_total: u64,
    count_private: u64,
}

#[function_component(BookmarksProvider)]
pub fn bookmarks_provider(props: &Props) -> Html {
    let state = use_state(|| {
        // gloo_console::info!("use_state State");
        State::from(props.params.as_ref())
    });

    {
        let state = state.clone();
        use_effect_with_deps(
            move |params| {
                // gloo_console::info!(format!("props.params updated {:?} (data: {:?})", params, state.data));
                if state.data.is_some() {
                    let mut new_state = State::from(params.as_ref());
                    new_state.stats = state.stats.clone();
                    new_state.data = Eventually::None;
                    state.set(new_state);
                }
            },
            props.params.clone(),
        );
    }

    // gloo_console::info!(format!("render bookmarks_provider with state {:?} and props {:?}", state, props.params));

    if state.stats.is_none() {
        let mut new_state = (*state).clone();
        new_state.stats = Eventually::Loading;
        state.set(new_state);
        let state = state.clone();
        spawn_local(async move {
            // gloo_console::info!("fetch stats");
            let data = fetch_stats().await;

            let mut new_state = (*state).clone();
            new_state.stats = Eventually::Some(Stats {
                count_total: data.count_total,
                count_private: data.count_private,
            });
            state.set(new_state);
        });
    }

    if state.data.is_none() && state.stats.is_some() {
        let mut new_state = (*state).clone();
        new_state.data = Eventually::Loading;
        state.set(new_state);

        let state = state.clone();
        spawn_local(async move {
            // gloo_console::info!("fetch bookmarks");
            let bookmarks_and_pages = fetch_bookmarks(&state).await;
            let mut new_state = (*state).clone();
            new_state.data = Eventually::Some(Data {
                bookmarks: Rc::new(bookmarks_and_pages.0),
                pages_count: bookmarks_and_pages.1,
            });
            state.set(new_state);
        });
    }

    let on_change_order = {
        let state = state.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |order: Order| {
            // gloo_console::info!(format!("change order {:?}", order));
            let mut new_state = (*state).clone();
            new_state.order = order;
            new_state.page = 0;
            new_state.data = Eventually::None;

            trigger_update(&on_change, &state, new_state);
        })
    };

    let on_previous = {
        let state = state.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |_| {
            // gloo_console::info!(format!("change page [previous]"));
            let mut new_state = (*state).clone();
            new_state.page = state.page.checked_sub(1).unwrap_or_default();
            new_state.data = Eventually::None;

            trigger_update(&on_change, &state, new_state);
        })
    };

    let on_next = {
        let state = state.clone();
        let on_change = props.on_change.clone();
        // let bookmarks = bookmarks.clone();
        Callback::from(move |_| {
            // gloo_console::info!(format!("change page [next]"));
            let mut new_state = (*state).clone();
            new_state.page = state
                .data
                .as_ref()
                .map(|data| data.pages_count)
                .unwrap_or_default()
                .checked_sub(1)
                .unwrap_or_default()
                .min(state.page + 1);
            new_state.data = Eventually::None;

            trigger_update(&on_change, &state, new_state);
        })
    };

    let on_change_page_size = {
        let state = state.clone();
        let on_change = props.on_change.clone();
        // let props = props.clone();
        Callback::from(move |page_size: u64| {
            // gloo_console::info!(format!("change page_size {:?}", page_size));
            let mut new_state = (*state).clone();
            new_state.page = u64::default();
            new_state.page_size = page_size;
            new_state.data = Eventually::None;

            trigger_update(&on_change, &state, new_state);
        })
    };

    let on_select_tag_filter = {
        let state = state.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |t: AttrValue| {
            if !state.tags.contains(&t) {
                // gloo_console::info!(format!("select tag {}", &t));
                let mut new_tags = (*state.tags).clone();
                new_tags.push(t);

                let mut new_state = (*state).clone();
                new_state.tags = Rc::new(new_tags);
                new_state.data = Eventually::None;

                trigger_update(&on_change, &state, new_state);
            }
        })
    };

    let on_change_tags = {
        let state = state.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |tags: Vec<AttrValue>| {
            // gloo_console::info!(format!("change tags {}", tags.len()));
            let mut new_state = (*state).clone();
            new_state.tags = Rc::new(tags);
            new_state.data = Eventually::None;

            trigger_update(&on_change, &state, new_state);
        })
    };

    match state.data.as_ref() {
        Eventually::Some(data) => {
            let context = BookmarksContext {
                bookmarks: data.bookmarks.clone(),
                tags: Rc::new(Vec::new()),
                order: state.order,
                page: state.page,
                page_size: state.page_size,
                selected_tags: state.tags.clone(),
                page_count: data.pages_count,
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
        _ => html! {
            <div></div>
        },
    }
}

fn trigger_update(
    callback: &Option<Callback<(Route, Params)>>,
    state: &UseStateHandle<State>,
    new_state: State,
) {
    if let Some(callback) = callback {
        // gloo_console::info!("execute callback");
        callback.emit((Route::Bookmarks, Params::from(&new_state)));
    } else {
        // gloo_console::info!("update local state");
        state.set(new_state);
    }
}

async fn fetch_bookmarks(state: &State) -> (Vec<Bookmark>, u64) {
    // todo review query param serialization and struct shared with API
    let params = vec![
        ("order", state.order.query_param().to_string()),
        ("page", state.page.to_string()),
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
