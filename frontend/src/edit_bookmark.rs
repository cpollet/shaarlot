use crate::data::Bookmark;
use crate::Route;
use gloo_net::http::Request;
use rest_api::bookmarks::{UpdateBookmarkRequest, URL_BOOKMARK};
use std::rc::Rc;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub bookmark: Rc<Bookmark>,
}

#[derive(Clone, PartialEq)]
struct State {
    focused: bool,
    bookmark: Bookmark,
}

#[function_component(EditBookmark)]
pub fn edit_bookmark(props: &Props) -> Html {
    let state = use_state(|| State {
        focused: false,
        bookmark: (*props.bookmark).clone(),
    });
    let navigator = use_navigator().unwrap();

    let url_input_ref = use_node_ref();

    {
        let url_input_ref = url_input_ref.clone();
        let state = state.clone();
        use_effect(move || {
            if !state.focused {
                let _ = url_input_ref.cast::<HtmlInputElement>().unwrap().focus();
                let mut new_state = (*state).clone();
                new_state.focused = true;
                state.set(new_state);
            }
        });
    }

    let onsubmit = {
        let state = state.clone();
        let navigator = navigator.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let state = state.clone();
            let navigator = navigator.clone();
            spawn_local(async move {
                // TODO finish this
                let _todo =
                    Request::put(&URL_BOOKMARK.replace(":id", &state.bookmark.id.to_string()))
                        .json(&UpdateBookmarkRequest::from(&state.bookmark))
                        .expect("could not set json")
                        .send()
                        .await;
                navigator.push(&Route::Bookmarks);
            })
        })
    };
    let onclick_cancel = {
        let navigator = navigator.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            navigator.push(&Route::Bookmarks);
        })
    };

    let oninput_url = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let mut new_state = (*state).clone();
            let input: HtmlInputElement = e.target_unchecked_into();
            new_state.bookmark.url = AttrValue::from(input.value());
            state.set(new_state);
        })
    };
    let oninput_title = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let mut new_state = (*state).clone();
            let value = e
                .target_unchecked_into::<HtmlInputElement>()
                .value()
                .to_string();
            if value.is_empty() {
                new_state.bookmark.title = None
            } else {
                new_state.bookmark.title = Some(AttrValue::from(value))
            }
            state.set(new_state);
        })
    };
    let oninput_description = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let mut new_state = (*state).clone();
            let value = e
                .target_unchecked_into::<HtmlInputElement>()
                .value()
                .to_string();
            if value.is_empty() {
                new_state.bookmark.description = None
            } else {
                new_state.bookmark.description = Some(AttrValue::from(value))
            }
            state.set(new_state);
        })
    };

    html! {
        <div class="centered-box">
            <h1 class="centered-box__title">{"Edit bookmark"}</h1>
            <form {onsubmit}>
                <p>
                    <input
                        ref={url_input_ref}
                        type="text"
                        placeholder="url"
                        value={state.bookmark.url.clone()}
                        oninput={oninput_url}
                    />
                </p>
                <p>
                    <input
                        type="text"
                        placeholder="title"
                        value={state.bookmark.title.clone()}
                        oninput={oninput_title}
                    />
                </p>
                <p>
                    <textarea
                        placeholder="description"
                        value={state.bookmark.description.clone()}
                        oninput={oninput_description}
                    />
                </p>
                <p class="centered-box__buttons">
                    <button type="button" onclick={onclick_cancel} class="button--safe">{"Cancel"}</button>
                    {" "}
                    <button type="submit" class="button--action">{"Update"}</button>
                </p>
            </form>
        </div>
    }
}

#[function_component(EditBookmarkHOC)]
pub fn edit_bookmark_hoc() -> Html {
    let bookmark = use_context::<Rc<Bookmark>>().expect("no ctx found");
    html! {
        <EditBookmark bookmark={bookmark.clone()} />
    }
}
