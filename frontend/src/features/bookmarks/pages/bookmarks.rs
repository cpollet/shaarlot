use super::super::bookmark::Bookmark;
use super::super::data::Bookmark as BookmarkData;
use crate::features::bookmarks::bookmarks_provider::Order;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub bookmarks: Rc<Vec<BookmarkData>>,
    pub order: Order,
    pub on_change_order: Callback<Order>,
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
                {"sort:"}
                <a {onclick} class="material-icons-outlined md-18" href="#" >{state.order.icon()}</a>
            </div>
            <ul class="bookmarks">
            {
                props.bookmarks.as_slice().iter().map(|b| html! {
                    <Bookmark key={b.id} bookmark={Rc::new(b.clone())} />
                }).collect::<Html>()
            }
            </ul>
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
