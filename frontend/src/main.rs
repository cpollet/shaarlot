use gloo_net::http::Request;
use rest_api::{BookmarkResponse, URL_BOOKMARKS};
use yew::platform::spawn_local;
use yew::prelude::*;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BookmarksList/>
    }
}

#[function_component(BookmarksList)]
fn bookmarks_list() -> Html {
    let data = use_state(|| None);

    {
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    data.set(Some (match Request::get(URL_BOOKMARKS).send().await {
                        Err(_) => Err(format!("Error fetching data")),
                        Ok(resp) => {
                            if !resp.ok() {
                                Err(format!(
                                "Error fetching data {} ({})",
                                resp.status(),
                                resp.status_text()
                            ))
                            } else {
                                resp.json::<Vec<BookmarkResponse>>()
                                    .await
                                    .map_err(|err|err.to_string())
                            }
                        }
                    }))
                });
            }

            || {}
        });
    }

    match data.as_ref() {
        None => {
            html! {
                <div>{"No server response"}</div>
            }
        }
        Some(Ok(data)) => {
            html! {
                <ul>
                {
                    data.into_iter().map(|b| html! {
                        <BookmarkItem
                            url={AttrValue::from(b.url.clone())}
                            title={b.title.as_ref().map(|v| AttrValue::from(v.clone()))}
                            description={b.description.as_ref().map(|v| AttrValue::from(v.clone()))}
                        />
                    }).collect::<Html>()
                }
                </ul>
            }
        }
        Some(Err(err)) => {
            html! {
                <div>{"Error requesting data from server: "}{err}</div>
            }
        }
    }
}

#[derive(Properties, PartialEq)]
struct BookmarkItemProperties {
    url: AttrValue,
    title: Option<AttrValue>,
    description: Option<AttrValue>,
}

#[function_component(BookmarkItem)]
fn bookmark_item(props: &BookmarkItemProperties) -> Html {
    html! {
        <li>
            <a href={props.url.clone()}>
                {props.title.clone().unwrap_or_else(|| props.url.clone())}
            </a>
        </li>
    }
}
