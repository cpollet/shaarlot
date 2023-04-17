use std::rc::Rc;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub placeholder: Option<AttrValue>,
    pub tags: Vec<AttrValue>,
    pub available_tags: Option<Rc<Vec<AttrValue>>>,
    pub onupdate: Callback<Vec<AttrValue>>,
}

#[derive(Clone, PartialEq)]
struct Match {
    prefix: AttrValue,
    matched: AttrValue,
    suffix: AttrValue,
    full: AttrValue,
}

#[derive(Default, Clone, PartialEq)]
struct State {
    tags: Vec<AttrValue>,
    available_tags: Rc<Vec<AttrValue>>,
    matches: Vec<Match>,
    selected_match: Option<usize>,
    selected_tag: Option<usize>,
    string: AttrValue,
    focus: bool,
}

#[function_component(TagInput)]
pub fn tag_input(props: &Props) -> Html {
    let state = use_state(|| State {
        tags: props.tags.clone(),
        available_tags: match &props.available_tags {
            None => Rc::new(vec![]),
            Some(tags) => tags.clone(),
        },
        matches: Vec::default(),
        selected_match: Option::default(),
        selected_tag: Option::default(),
        string: Default::default(),
        focus: false,
    });

    let input_ref = use_node_ref();

    let onkeydown = {
        let state = state.clone();
        let props = props.clone();
        Callback::from(move |e: KeyboardEvent| {
            if (e.key() == "Enter"
                && !e
                    .target_unchecked_into::<HtmlInputElement>()
                    .value()
                    .is_empty())
                || e.key() == " "
                || e.key() == "Spacebar"
            {
                e.prevent_default();
                let value: String = e.target_unchecked_into::<HtmlInputElement>().value();
                let mut new_state = (*state).clone();
                if let Some(selected_match) = state.selected_match {
                    new_state
                        .tags
                        .push(state.matches.get(selected_match).unwrap().full.clone())
                } else if !value.is_empty() {
                    new_state.tags.push(AttrValue::from(value));
                }
                new_state.string = AttrValue::default();
                new_state.matches = Vec::default();
                new_state.selected_match = None;
                props.onupdate.emit(new_state.tags.clone());
                state.set(new_state);
            } else if e.key() == "Backspace" {
                if let Some(selected_tag) = state.selected_tag {
                    e.prevent_default();
                    let mut new_state = (*state).clone();
                    new_state.tags.remove(selected_tag);
                    if !new_state.tags.is_empty() {
                        new_state.selected_tag =
                            Some(selected_tag.checked_sub(1).unwrap_or_default())
                    } else {
                        new_state.selected_tag = None;
                    }
                    props.onupdate.emit(new_state.tags.clone());
                    state.set(new_state);
                } else if state.string.is_empty() {
                    e.prevent_default();
                    let mut new_state = (*state).clone();
                    if let Some(tag) = new_state.tags.pop() {
                        let matches = compute_matches(
                            &state.available_tags,
                            &new_state.tags,
                            &tag,
                            state.selected_match,
                        );
                        new_state.selected_match = matches.0;
                        new_state.matches = matches.1;
                        new_state.string = tag;
                    }
                    props.onupdate.emit(new_state.tags.clone());
                    state.set(new_state);
                }
            } else if e.key() == "Delete" || e.key() == "Del" {
                if let Some(selected_tag) = state.selected_tag {
                    e.prevent_default();
                    let mut new_state = (*state).clone();
                    new_state.tags.remove(selected_tag);
                    if !new_state.tags.is_empty() {
                        new_state.selected_tag = Some(selected_tag.min(new_state.tags.len() - 1))
                    } else {
                        new_state.selected_tag = None;
                    }
                    props.onupdate.emit(new_state.tags.clone());
                    state.set(new_state);
                }
            } else if e.key() == "Up" || e.key() == "ArrowUp" {
                e.prevent_default();
                if !state.matches.is_empty() {
                    let mut new_state = (*state).clone();
                    new_state.selected_match = Some(match state.selected_match {
                        Some(i) if i > 0 => i - 1,
                        Some(_) | None => state.matches.len() - 1,
                    });
                    state.set(new_state);
                }
            } else if e.key() == "Down" || e.key() == "ArrowDown" {
                e.prevent_default();
                if !state.matches.is_empty() {
                    let mut new_state = (*state).clone();
                    new_state.selected_match = Some(match state.selected_match {
                        Some(i) if i < state.matches.len() - 1 => i + 1,
                        Some(_) | None => 0,
                    });
                    state.set(new_state);
                }
            } else if e.key() == "Left" || e.key() == "ArrowLeft" {
                if state.string.is_empty() && !state.tags.is_empty() {
                    e.prevent_default();
                    let mut new_state = (*state).clone();
                    new_state.selected_tag = Some(match state.selected_tag {
                        Some(i) if i > 0 => i - 1,
                        Some(_) | None => state.tags.len() - 1,
                    });
                    state.set(new_state);
                }
            } else if e.key() == "Right" || e.key() == "ArrowRight" {
                if state.string.is_empty() && !state.tags.is_empty() {
                    e.prevent_default();
                    let mut new_state = (*state).clone();
                    new_state.selected_tag = Some(match state.selected_tag {
                        Some(i) if i < state.tags.len() - 1 => i + 1,
                        Some(_) | None => 0,
                    });
                    state.set(new_state);
                }
            } else if e.key() == "Esc" || e.key() == "Escape" {
                e.prevent_default();
                let mut new_state = (*state).clone();
                new_state.selected_tag = None;
                new_state.selected_match = None;
                if state.selected_match.is_none() {
                    new_state.matches = Vec::default();
                }
                state.set(new_state);
            }
        })
    };

    let oninput = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            let mut new_state = (*state).clone();

            if value.is_empty() {
                new_state.matches = Vec::default();
            } else {
                let matches = compute_matches(
                    &state.available_tags,
                    &state.tags,
                    &value,
                    state.selected_match,
                );
                new_state.selected_match = matches.0;
                new_state.matches = matches.1;
                new_state.selected_tag = None;
            }
            new_state.string = AttrValue::from(value);
            state.set(new_state);
        })
    };

    let onfocus = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            new_state.focus = true;
            state.set(new_state);
        })
    };

    let onblur = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            new_state.focus = false;
            state.set(new_state);
        })
    };

    let delete = {
        let state = state.clone();
        let props = props.clone();
        let input_ref = input_ref.clone();
        Callback::from(move |tag: AttrValue| {
            let mut new_state = (*state).clone();
            new_state.tags.retain(|e| e != &tag);

            new_state.selected_match = None;
            new_state.matches = compute_matches(
                &state.available_tags,
                &new_state.tags,
                state.string.as_str(),
                state.selected_match,
            )
            .1;
            new_state.selected_tag = None;

            let _ = input_ref.cast::<HtmlInputElement>().unwrap().focus();
            props.onupdate.emit(new_state.tags.clone());
            state.set(new_state);
        })
    };

    let click = {
        let state = state.clone();
        let input_ref = input_ref.clone();
        Callback::from(move |tag_index: usize| {
            let mut new_state = (*state).clone();

            new_state.selected_tag = Some(tag_index);

            let _ = input_ref.cast::<HtmlInputElement>().unwrap().focus();
            state.set(new_state);
        })
    };

    let add = {
        let state = state.clone();
        let props = props.clone();
        let input_ref = input_ref.clone();
        Callback::from(move |tag: AttrValue| {
            let mut new_state = (*state).clone();
            new_state.tags.push(tag);
            new_state.string = AttrValue::default();
            new_state.matches = Vec::default();
            new_state.selected_match = None;
            let _ = input_ref.cast::<HtmlInputElement>().unwrap().focus();
            props.onupdate.emit(new_state.tags.clone());
            state.set(new_state);
        })
    };

    fn compute_matches(
        tags: &[AttrValue],
        used_tags: &[AttrValue],
        pattern: &str,
        current_selection: Option<usize>,
    ) -> (Option<usize>, Vec<Match>) {
        let matches = tags
            .iter()
            .map(|e| (e.find(pattern), e))
            .filter(|e| e.0.is_some())
            .map(|e| (e.0.unwrap(), e.1))
            .filter(|(_, e)| !used_tags.contains(e))
            .map(|e| Match {
                prefix: AttrValue::from((e.1[..e.0]).to_owned()),
                matched: AttrValue::from((e.1[e.0..e.0 + pattern.bytes().len()]).to_owned()),
                suffix: AttrValue::from((e.1[e.0 + pattern.bytes().len()..]).to_owned()),
                full: e.1.clone(),
            })
            .collect::<Vec<Match>>();

        if matches.is_empty() {
            (None, matches)
        } else if let Some(current_selection) = current_selection {
            (Some(current_selection.min(matches.len() - 1)), matches)
        } else {
            (None, matches)
        }
    }

    html! {
        <div class={classes!("input-tag", state.focus.then_some("input-tag--focus"))}>
            <div class="input-tag__tags">
                {
                    state.tags.iter().enumerate().map(|(i,tag)| html! {
                        <span
                            class={
                                if state.selected_tag.eq(&Some(i)) {
                                    "input-tag__tag--selected"
                                } else {
                                    "input-tag__tag"
                            } }
                        >
                           <span
                                class="material-icons-outlined input-tag__delete"
                                onclick={
                                    let delete = delete.clone();
                                    let tag = tag.clone();
                                    move |_| delete.emit(tag.clone())
                                }
                            >{"clear"}</span>
                            {"\u{00a0}"}
                            <span
                                onclick={
                                    let click = click.clone();
                                    move |_| click.emit(i)
                                }>
                                {tag}
                            </span>
                        </span>
                    }).collect::<Html>()
                }
            </div>
            <div class="input-tag__input">
                <input
                    placeholder={
                        if state.tags.is_empty() {
                            props.placeholder.clone()
                        } else {
                            None
                        }
                    }
                    type="text"
                    value={state.string.clone()}
                    ref={input_ref}
                    {onfocus} {onblur}
                    {oninput}
                    {onkeydown}
                />
                { if !state.matches.is_empty() { html! {
                <ul class="input-tag__autocomplete">
                    {
                        state.matches.iter().enumerate().map(|(i, tag)| html! {
                            <li
                                class={
                                    if state.selected_match.eq(&Some(i)) {
                                        "input-tag__autocomplete--selected"
                                    } else {
                                        ""
                                    }
                                }
                                onclick={
                                    let add = add.clone();
                                    let tag = tag.clone();
                                    move |_| add.emit(tag.full.clone())
                                }
                            >{tag.prefix.clone()}<span class="input-tag__autocomplete-match">{tag.matched.clone()}</span>{tag.suffix.clone()}</li>
                        }).collect::<Html>()
                    }
                </ul>
                } } else { html! { <></> } } }
            </div>
        </div>
    }
}
