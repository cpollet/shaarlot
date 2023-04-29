use crate::features::bookmarks::bookmarks_provider::{BookmarksProvider, Filter, Order, Params};
use crate::Route;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub children: ChildrenWithProps<BookmarksProvider>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
enum QueryOrder {
    #[serde(rename(serialize = "creation_date:asc", deserialize = "creation_date:asc"))]
    CreationDateDesc,
    #[serde(rename(serialize = "creation_date:desc", deserialize = "creation_date:desc"))]
    CreationDateAsc,
}

impl From<Order> for QueryOrder {
    fn from(value: Order) -> Self {
        match value {
            Order::CreationDateDesc => QueryOrder::CreationDateDesc,
            Order::CreationDateAsc => QueryOrder::CreationDateAsc,
        }
    }
}

impl From<&QueryOrder> for Order {
    fn from(value: &QueryOrder) -> Self {
        match value {
            QueryOrder::CreationDateDesc => Order::CreationDateDesc,
            QueryOrder::CreationDateAsc => Order::CreationDateAsc,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
enum QueryFilter {
    #[serde(rename(serialize = "private", deserialize = "private"))]
    Private,
    #[serde(rename(serialize = "public", deserialize = "public"))]
    Public,
}

impl From<Filter> for QueryFilter {
    fn from(value: Filter) -> Self {
        match value {
            Filter::Private => QueryFilter::Private,
            Filter::Public => QueryFilter::Public,
        }
    }
}

impl From<&QueryFilter> for Filter {
    fn from(value: &QueryFilter) -> Self {
        match value {
            QueryFilter::Private => Filter::Private,
            QueryFilter::Public => Filter::Public,
        }
    }
}

// todo review query param serialization and struct shared with API (less relevant as this is
//  front-end only)
#[derive(Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct QueryParams {
    page: Option<u64>,
    page_size: Option<u64>,
    #[serde(with = "serialize_tags", default)]
    // todo should it be String?
    tags: Option<Vec<AttrValue>>,
    #[serde(with = "serialize_tags", default)]
    // todo should it be String?
    terms: Option<Vec<AttrValue>>,
    order: Option<QueryOrder>,
    filter: Option<QueryFilter>,
}

impl From<Params> for QueryParams {
    fn from(value: Params) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
            tags: value.tags,
            terms: value.search_terms,
            order: value.order.map(QueryOrder::from),
            filter: value.filter.map(QueryFilter::from),
        }
    }
}

impl From<&QueryParams> for Params {
    fn from(value: &QueryParams) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
            tags: value.tags.clone(),
            search_terms: value.terms.clone(),
            order: value.order.as_ref().map(Order::from),
            filter: value.filter.as_ref().map(Filter::from),
        }
    }
}

mod serialize_tags {
    use serde::de::{Error, Visitor};
    use serde::{Deserializer, Serializer};
    use std::fmt::Formatter;
    use yew::AttrValue;

    pub fn serialize<S: Serializer>(
        vec: &Option<Vec<AttrValue>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match vec {
            None => serializer.serialize_none(),
            Some(vec) => serializer.serialize_str(
                &vec.iter()
                    .map(|tag| tag.to_string())
                    .reduce(|a, b| format!("{} {}", a, b))
                    .unwrap_or_default(),
            ),
        }
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<AttrValue>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TagsVisitor {}

        impl TagsVisitor {
            fn new() -> Self {
                Self {}
            }
        }

        impl<'de> Visitor<'de> for TagsVisitor {
            type Value = Option<Vec<AttrValue>>;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("an URL-encoded string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let tags = v.split(' ').collect::<Vec<_>>();

                let tags = tags
                    .into_iter()
                    // .map(|i| i.expect("error should have been returned already"))
                    .map(|t| AttrValue::from(t.to_string()))
                    .collect::<Vec<AttrValue>>();

                if tags.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(tags))
                }
            }
        }
        deserializer.deserialize_str(TagsVisitor::new())
    }
}

#[function_component(BookmarksQuery)]
pub fn bookmarks_query(props: &Props) -> Html {
    let query_params = use_location()
        .unwrap()
        .query::<QueryParams>()
        .ok()
        .unwrap_or_default();

    let on_update = {
        let navigator = use_navigator().unwrap();
        let location = use_location().unwrap();
        let params = Some(Params::from(&query_params));
        Callback::from(move |p: (Route, Params)| {
            let route = p.0;
            let new_params = p.1;
            let curr_params = params.as_ref();

            if location.path() != route.to_path()
                || curr_params.is_none()
                || curr_params != Some(&new_params)
            {
                let _ = navigator.push_with_query(&route, &QueryParams::from(new_params));
            }
        })
    };

    html! {
        <>
            {
                props.children.iter().map(|mut child| {
                    let mut props = Rc::make_mut(&mut child.props);
                    props.params = Some(Params::from(&query_params.clone()));
                    props.on_change = Some(on_update.clone());
                    child
                }).collect::<Html>()
            }
        </>
    }
}
