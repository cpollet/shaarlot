use std::cmp::Reverse;
use std::ops::Deref;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq, Debug)]
pub struct Props {
    pub text: AttrValue,
    pub terms: Option<Rc<Vec<AttrValue>>>,
}

#[function_component(Highlight)]
pub fn highlight(props: &Props) -> Html {
    if props.terms.is_none() || props.terms.as_ref().unwrap().is_empty() {
        return html! {
            { props.text.clone() }
        };
    }

    let terms = {
        let mut terms = props.terms.as_ref().unwrap().deref().clone();
        terms.sort_by_key(|b| Reverse(b.len()));
        terms
            .into_iter()
            .map(|t| t.to_lowercase())
            .collect::<Vec<String>>()
    };

    let mut zones = vec![Zone::from(&props.text)];
    for term in &terms {
        zones = zones.highlight(term.as_str());
    }

    html! {
        zones
        .into_iter()
        .map(|e| match e {
            Zone::Normal(s) => html! { s },
            Zone::Highlight(s) => html!{ <span class="highlight">{s}</span> }
        })
        .collect::<Html>()
    }
}

#[derive(Debug)]
enum Zone<'a> {
    Highlight(&'a str),
    Normal(&'a str),
}

impl<'a> From<&'a AttrValue> for Zone<'a> {
    fn from(value: &'a AttrValue) -> Self {
        Zone::Normal(value.as_str())
    }
}

trait Highlightable<'a> {
    fn highlight(self, pat: &'a str) -> Vec<Zone<'a>>;
}

impl<'a> Highlightable<'a> for &'a AttrValue {
    fn highlight(self, pat: &'a str) -> Vec<Zone<'a>> {
        Zone::Normal(self.as_str()).highlight(pat)
    }
}

impl<'a> Highlightable<'a> for Zone<'a> {
    fn highlight(self, pat: &'a str) -> Vec<Self> {
        match self {
            Zone::Normal(str) => match str.to_lowercase().find(pat) {
                None => vec![self],
                Some(0) => {
                    let mut vec = vec![Zone::Highlight(&str[0..pat.len()])];
                    vec.append(&mut Zone::Normal(&str[pat.len()..]).highlight(pat));
                    vec
                }
                Some(idx) => {
                    let mut vec = vec![
                        Zone::Normal(&str[0..idx]),
                        Zone::Highlight(&str[idx..idx + pat.len()]),
                    ];
                    vec.append(&mut Zone::Normal(&str[idx + pat.len()..]).highlight(pat));
                    vec
                }
            },
            highlight => vec![highlight],
        }
    }
}

impl<'a, T> Highlightable<'a> for Vec<T>
where
    T: Highlightable<'a>,
{
    fn highlight(self, pat: &'a str) -> Vec<Zone> {
        self.into_iter()
            .flat_map(|h| h.highlight(pat))
            .collect::<Vec<Zone>>()
    }
}
