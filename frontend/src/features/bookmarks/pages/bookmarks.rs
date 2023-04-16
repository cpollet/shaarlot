use super::super::bookmark::Bookmark;
use super::super::data::Bookmark as BookmarkData;
use crate::components::nav::Nav;
use crate::components::page_size::PageSize;
use crate::features::bookmarks::bookmarks_provider::Order;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub bookmarks: Rc<Vec<BookmarkData>>,
    pub order: Order,
    pub page: u64,
    pub page_count: u64,
    pub page_size: u64,
    pub on_change_order: Callback<Order>,
    pub on_previous: Callback<()>,
    pub on_next: Callback<()>,
    pub on_change_page_size: Callback<u64>,
}

struct State {
    order: Order,
}

impl State {
    fn from(props: &Props) -> Self {
        Self {
            order: props.order.clone(),
        }
    }
}

impl Order {
    fn invert(&self) -> Self {
        match self {
            Order::Asc => Order::Desc,
            Order::Desc => Order::Asc,
        }
    }
    fn icon(&self) -> &str {
        match self {
            Order::Asc => "expand_less",
            Order::Desc => "expand_more",
        }
    }
}

#[function_component(Bookmarks)]
pub fn bookmarks(props: &Props) -> Html {
    let state = use_state(|| State::from(props));

    let onclick = {
        let state = state.clone();
        let props = props.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            props.on_change_order.emit(state.order.invert())
        })
    };

    html! {
        <div>
            <div class="bookmarks-header">
                <div class="bookmarks__filter">
                </div>
                <Nav
                    page={props.page + 1}
                    page_count={props.page_count}
                    on_previous={props.on_previous.clone()}
                    on_next={props.on_next.clone()}
                />
                <PageSize
                    page_size={props.page_size}
                    on_change_page_size={props.on_change_page_size.clone()}
                />
                <div class="bookmarks__sort">
                    {"sort:"}
                    <a {onclick} class="material-icons-outlined md-18" href="#">
                        {state.order.icon()}
                    </a>
                </div>
            </div>
            <ul class="bookmarks">
            {
                props.bookmarks.as_slice().iter().map(|b| html! {
                    <Bookmark key={b.id} bookmark={Rc::new(b.clone())} />
                }).collect::<Html>()
            }
            </ul>
            <div class="bookmarks-footer">
                    <div class="bookmarks__filter">
                    </div>
                    <Nav
                        page={props.page + 1}
                        page_count={props.page_count}
                        on_previous={props.on_previous.clone()}
                        on_next={props.on_next.clone()}
                    />
                    <PageSize
                        page_size={props.page_size}
                        on_change_page_size={props.on_change_page_size.clone()}
                    />
                    <div class="bookmarks__sort">
                    </div>
                </div>
        </div>
    }
}

#[function_component(BookmarksHOC)]
pub fn bookmarks_hoc() -> Html {
    let bookmarks = use_context::<Props>().expect("no ctx found");
    html! {
        <Bookmarks ..bookmarks />
    }
}
