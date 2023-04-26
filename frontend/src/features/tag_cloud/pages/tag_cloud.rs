use super::super::super::super::data::Tags;
use crate::Route;
use serde::Serialize;
use std::rc::Rc;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    tags: Rc<Tags>,
}

#[derive(Serialize)]
pub struct Tag {
    pub tags: String,
}

#[function_component(TagCloud)]
pub fn tag_cloud(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();

    let max_count = props.tags.iter().map(|t| t.count).max().unwrap_or_default() as f32;
    let min_count = props.tags.iter().map(|t| t.count).min().unwrap_or_default() as f32;

    let font_step = (19 - 1) as f32 / (max_count - min_count);

    html! {
        <div class="centered-box">
            <h1 class="centered-box__title">{"Tag cloud"}</h1>
            <ul class="tag-cloud">
                {
                    props.tags.iter().map(|t| {
                        let size = (1f32 + (t.count as f32 - min_count) * font_step).round() as i32;
                        html! {
                            <li>
                                <a
                                    href={format!("{}?tags={}",Route::Bookmarks.to_path(), &t.name)}
                                    // href={format!("/bookmarks/~search?tags={}", &t.name)}
                                    data-weight={size.to_string()}
                                    onclick={
                                        let navigator = navigator.clone();
                                        let tag = Tag { tags: t.name.to_string() };
                                        Callback::from(move |e: MouseEvent| {
                                            e.prevent_default();
                                            let _ = navigator.push_with_query(
                                                // &Route::BookmarksSearch,
                                                &Route::Bookmarks,
                                                &tag
                                            );
                                        })
                                    }
                                >
                                    {t.name.clone()}
                                    <span>{t.count}</span>
                                </a>
                            </li>
                        }
                    }).collect::<Html>()
                }
            </ul>
        </div>
    }
}

#[function_component(TagCloudHOC)]
pub fn tag_cloud_hoc() -> Html {
    let tags = use_context::<Rc<Tags>>().expect("no ctx found");

    html! {
        <TagCloud {tags} />
    }
}
