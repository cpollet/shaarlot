use url::Url;

pub fn clean<U>(url: U, ignored_query_params: &[&str]) -> Option<String>
where
    U: Into<String>,
{
    let mut url = url.into();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        url = format!("http://{}", url);
    }

    match Url::parse(&url) {
        Err(_) => None,
        Ok(mut url) => {
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
            Some(url)
        }
    }
}
