use super::super::super::super::data::Bookmark as BookmarkData;
use super::super::super::super::data::Tags;
use super::super::bookmark::Bookmark;
use super::super::bookmarks_provider::{Filter, Order};
use crate::components::nav::Nav;
use crate::components::page_size::PageSize;
use crate::components::tag_input::TagInput;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub bookmarks: Rc<Vec<BookmarkData>>,
    pub tags: Rc<Tags>,
    pub order: Order,
    pub page: u64,
    pub page_count: u64,
    pub page_size: u64,
    pub selected_tags: Rc<Vec<AttrValue>>,
    pub filter: Option<Filter>,
    pub on_change_order: Callback<Order>,
    pub on_previous: Callback<()>,
    pub on_next: Callback<()>,
    pub on_change_page_size: Callback<u64>,
    pub on_select_tag_filter: Callback<AttrValue>,
    pub on_change_tags: Callback<Vec<AttrValue>>,
    pub on_change_filter: Callback<Filter>,
    pub links: u64,
    pub private_links: u64,
}

impl Order {
    fn invert(&self) -> Self {
        match self {
            Order::CreationDateAsc => Order::CreationDateDesc,
            Order::CreationDateDesc => Order::CreationDateAsc,
        }
    }
    fn icon(&self) -> &str {
        match self {
            Order::CreationDateAsc => "expand_less",
            Order::CreationDateDesc => "expand_more",
        }
    }
}

#[function_component(Bookmarks)]
pub fn bookmarks(props: &Props) -> Html {
    // gloo_console::info!("render bookmarks");

    let onclick = {
        let props = props.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            props.on_change_order.emit(props.order.invert())
        })
    };

    html! {
        <div>
            <div class="bookmarks-header">
                <div class="bookmarks__stats">
                    <ul>
                        <li>{props.links}{" links"}</li>
                        <li>{props.private_links}{" private links"}</li>
                    </ul>
                </div>
                <div class="bookmarks__search">
                    // todo do not auto refresh after a new tag is added by typing
                    <TagInput
                        placeholder="filter by tag"
                        tags={(*props.selected_tags).clone()}
                        available_tags={Rc::new(props.tags.iter().map(|t| t.name.clone()).collect::<Vec<AttrValue>>())}
                        onupdate={props.on_change_tags.clone()}
                    />
                </div>
                <div class="bookmarks__stats"></div>
            </div>
            <div class="bookmarks-header">
                <div class="bookmarks__filter">
                    <span
                        class={classes!(
                            "material-icons-outlined", "md-14",
                            props
                                .filter
                                .map(|f| f == Filter::Private)
                                .unwrap_or_default()
                                .then_some("bookmarks__filter-item--selected")
                                .unwrap_or("bookmarks__filter-item")
                        )}
                        onclick={
                            let props = props.clone();
                            move |_:MouseEvent| props.on_change_filter.emit(Filter::Private)
                        }
                    >
                        {"lock"}
                    </span>
                    <span
                        class={classes!(
                            "material-icons-outlined", "md-14", "bookmarks__filter-item",
                            props
                                .filter
                                .map(|f| f == Filter::Public)
                                .unwrap_or_default()
                                .then_some("bookmarks__filter-item--selected")
                                .unwrap_or("bookmarks__filter-item")
                        )}
                        onclick={
                            let props = props.clone();
                            move |_:MouseEvent| props.on_change_filter.emit(Filter::Public)
                        }
                    >
                        {"public"}
                    </span>
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
                    <span {onclick} class="bookmarks__sort-toggle material-icons-outlined md-18">
                        {props.order.icon()}
                    </span>
                </div>
            </div>
            <ul class="bookmarks">
            {
                props.bookmarks.as_slice().iter().map(|b| html! {
                    <Bookmark
                        key={b.id}
                        bookmark={Rc::new(b.clone())}
                        on_select_tag_filter={props.on_select_tag_filter.clone()}
                    />
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
    let tags = use_context::<Rc<Tags>>().expect("no ctx found");
    html! {
        <Bookmarks tags={tags} ..bookmarks />
    }
}
