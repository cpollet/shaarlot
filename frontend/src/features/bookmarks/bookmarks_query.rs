use crate::features::bookmarks::bookmarks_provider::{BookmarksProvider, Order, Params};
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

// todo review query param serialization and struct shared with API (less relevant as this is
//  front-end only)
#[derive(Serialize, Deserialize, Default, PartialEq, Clone)]
struct QueryParams {
    page: Option<u64>,
    page_size: Option<u64>,
    #[serde(with = "serialize_tags", default)]
    tags: Option<Vec<AttrValue>>,
    order: Option<QueryOrder>,
}

impl From<Params> for QueryParams {
    fn from(value: Params) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
            tags: value.tags,
            order: value.order.map(QueryOrder::from),
        }
    }
}

impl From<&QueryParams> for Params {
    fn from(value: &QueryParams) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
            tags: value.tags.clone(),
            order: value.order.as_ref().map(Order::from),
        }
    }
}

mod serialize_tags {
    use serde::de::{Error, Visitor};
    use serde::{Deserializer, Serializer};
    use std::borrow::Cow;
    use std::fmt::Formatter;
    use urlencoding::{decode, encode};
    use yew::AttrValue;

    pub fn serialize<S: Serializer>(
        vec: &Option<Vec<AttrValue>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match vec {
            None => serializer.serialize_none(),
            Some(vec) => serializer.serialize_str(
                &vec.iter()
                    .map(|tag| encode(tag.as_str()))
                    .reduce(|a, b| Cow::Owned(format!("{}+{}", a, b)))
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
                let tags = v.split('+').map(decode).collect::<Vec<_>>();

                if tags.iter().any(|i| i.is_err()) {
                    return Err(Error::custom("not a valid UTF-8 string"));
                }

                let tags = tags
                    .into_iter()
                    .map(|i| i.expect("error should have been returned already"))
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

pub fn search(
    nav: &Navigator,
    route: Route,
    params: Params,
    curr_location: &Location,
    curr_params: Option<&Params>,
) {
    if curr_location.path() != route.to_path()
        || curr_params.is_none()
        || curr_params != Some(&params)
    {
        let _ = nav.push_with_query(&route, &QueryParams::from(params));
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
            search(&navigator, p.0, p.1, &location, params.as_ref());
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
