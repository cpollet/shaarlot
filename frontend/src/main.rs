use std::rc::Rc;
use gloo_net::http::Request;
use yew::html::IntoPropValue;
use rest_api::{BookmarkResponse, URL_BOOKMARKS};
use yew::platform::spawn_local;
use yew::prelude::*;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BookmarksProvider/>
    }
}

#[derive(Properties, Clone, PartialEq)]
struct BookmarkProps {
    url: AttrValue,
    title: Option<AttrValue>,
    description: Option<AttrValue>,
}

impl From<BookmarkResponse> for BookmarkProps {
    fn from(value: BookmarkResponse) -> Self {
        BookmarkProps {
            url: AttrValue::from(value.url),
            title: value.title.map(|v|AttrValue::from(v)),
            description: value.description.map(|v|AttrValue::from(v)),
        }
    }
}

#[derive(Properties, Clone, PartialEq)]
struct BookmarksProps {
    bookmarks: Rc<Vec<BookmarkProps>>,
}

impl IntoPropValue<Rc<Vec<BookmarkProps>>> for BookmarksProps {
    fn into_prop_value(self) -> Rc<Vec<BookmarkProps>> {
        self.bookmarks.clone()
    }
}

#[function_component(BookmarksProvider)]
fn bookmarks_provider() -> Html {
    let bookmarks = use_state(|| None);

    {
        let bookmarks = bookmarks.clone();
        use_effect(move || {
            if bookmarks.is_none() {
                spawn_local(async move {
                    bookmarks.set(Some (match Request::get(URL_BOOKMARKS).send().await {
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
                                        .map_err(|err| err.to_string())
                                        .map(|elements|
                                            elements
                                                .into_iter()
                                                .map(BookmarkProps::from)
                                                .collect::<Vec<BookmarkProps>>()
                                        )
                                        .map(|bookmarks| BookmarksProps { bookmarks: Rc::new(bookmarks) })
                            }
                        }
                    }))
                });
            }

            || {}
        });
    }

    match bookmarks.as_ref() {
        Some(Ok(bookmarks)) => html! {
            <Bookmarks bookmarks={(*bookmarks).clone()} />
        },
        Some(Err(err)) => html! {
            <div>{err}</div>
        },
        None => html! {
            <div>{"No data"}</div>
        }
    }
}

#[function_component(Bookmarks)]
fn bookmarks(props: &BookmarksProps) -> Html {
    html! {
        <ul>
        {
            props.bookmarks.as_slice().into_iter().map(|b| html! {
                <Bookmark ..b.clone() />
            }).collect::<Html>()
        }
        </ul>
    }
}

#[function_component(Bookmark)]
fn bookmark(props: &BookmarkProps) -> Html {
    html! {
        <li>
            <a href={props.url.clone()}>
                {props.title.clone().unwrap_or_else(|| props.url.clone())}
            </a>
        </li>
    }
}
