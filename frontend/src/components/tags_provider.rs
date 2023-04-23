use super::super::data::{Tag, Tags};
use gloo_net::http::Request;
use rest_api::tags::{GetTagsResult, URL_TAGS};
use std::rc::Rc;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_hooks::use_effect_once;

#[derive(PartialEq, Clone, Default)]
pub enum Order {
    #[default]
    Count,
    Name,
}

impl Order {
    fn query_param(&self) -> &str {
        match self {
            Order::Count => "count",
            Order::Name => "name",
        }
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub children: Children,
    pub order: Option<Order>,
}

#[derive(Clone, PartialEq, Default)]
struct State {
    tags: Option<Rc<Tags>>,
}

// todo move somewhere else
#[function_component(TagsProvider)]
pub fn tags_provider(props: &Props) -> Html {
    let state = use_state(State::default);

    {
        let state = state.clone();
        let props = props.clone();
        use_effect_once(move || {
            if state.tags.is_none() {
                spawn_local(async move {
                    state.set(State {
                        tags: Some(Rc::new(
                            fetch_tags(&props.order.clone().unwrap_or_default()).await,
                        )),
                    });
                });
            }

            || {}
        });
    }

    match &state.tags {
        None => html! {
            <div>{"loading"}</div>
        },
        Some(tags) => html! {
            <ContextProvider<Rc<Tags>> context={(*tags).clone()}>
                { props.children.clone() }
            </ContextProvider<Rc<Tags>>>
        },
    }
}

async fn fetch_tags(order: &Order) -> Tags {
    // todo review query param serialization and struct shared with API
    match GetTagsResult::from(
        Request::get(URL_TAGS)
            .query([("order", order.query_param())])
            .send()
            .await,
    )
    .await
    {
        Some(GetTagsResult::Success(tags)) => tags.into_iter().map(Tag::from).collect::<Vec<Tag>>(),
        _ => {
            // todo handle errors
            vec![]
        }
    }
}
