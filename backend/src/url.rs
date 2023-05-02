use url::Url;

// todo implement https://github.com/corbindavenport/link-cleaner/blob/main/js/shared.js#L4?
// todo implement https://github.com/Cimbali/CleanLinks/blob/master/addon/data/rules.json?

// todo better error reporting
pub fn clean<U>(url: U, ignored_query_params: &[&str]) -> Result<String, ()>
where
    U: Into<String>,
{
    let mut url = url.into();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        url = format!("http://{}", url);
    }

    let mut url = Url::parse(&url).map_err(|_| ())?;

    let filtered_query_params = url
        .query_pairs()
        .filter(|(name, _)| !ignored_query_params.contains(&name.as_ref()))
        .map(|(name, value)| (name.into_owned(), value.into_owned()))
        .collect::<Vec<(String, String)>>();

    url.query_pairs_mut()
        .clear()
        .extend_pairs(&filtered_query_params);
    let mut url = url.to_string();
    if url.ends_with('?') {
        url.remove(url.len() - 1);
    }
    Ok(url)
}
