use std::str::FromStr;
use crate::domain::repositories::BookmarkRepository;
use crate::domain::values::url_details::UrlDetails;
use anyhow::Context;
use reqwest::Client;
use std::sync::Arc;
use url::Url;
use webpage::HTML;

#[derive(Debug)]
pub struct GetUrlDetailsCommand<S>
where
    S: AsRef<str>,
{
    pub user_id: i32,
    pub url: S,
}

pub enum GetUrlDetailsResult {
    Exists(i32),
    UrlDetails {
        url: String,
        title: Option<String>,
        description: Option<String>,
    },
}

#[derive(Clone)]
pub struct GetUrlDetailsUseCase {
    bookmark_repository: Arc<dyn BookmarkRepository>,
    http_client: Client,
}

impl GetUrlDetailsUseCase {
    pub fn new(bookmark_repository: Arc<dyn BookmarkRepository>, http_client: Client) -> Self {
        Self {
            bookmark_repository,
            http_client,
        }
    }

    pub async fn execute<S>(
        &self,
        command: GetUrlDetailsCommand<S>,
    ) -> anyhow::Result<GetUrlDetailsResult>
    where
        S: AsRef<str>,
    {
        let bookmark_id = self
            .bookmark_repository
            .find_id_by_url(command.user_id, command.url.as_ref())
            .await
            .context("Could not get url details")?;

        if let Some(bookmark_id) = bookmark_id {
            return Ok(GetUrlDetailsResult::Exists(bookmark_id));
        }

        let response = self
            .http_client
            .get(Url::from_str(command.url.as_ref()).context("Could not parse URL")?)
            .send()
            .await
            .context("Could not fetch remote url")?;
        let url = response.url().to_string();
        let html = response
            .text()
            .await
            .context("Could not get response body")
            .and_then(|html| HTML::from_string(html, None).context("Could not parse HTML"))?;

        Ok(GetUrlDetailsResult::UrlDetails {
            url,
            title: html.title,
            description: html.description,
        })
    }
}
