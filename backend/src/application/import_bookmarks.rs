use crate::domain::entities::bookmark::Bookmark;
use crate::domain::repositories::BookmarkRepository;

use anyhow::{Context, Error};
use chrono::{DateTime, Utc};
use hmac::digest::KeyInit;
use hmac::Hmac;
use jwt::SignWithKey;
use reqwest::{header, Client};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sha2::Sha512;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Debug)]
pub struct ImportBookmarkCommand {
    pub user_id: i32,
    pub source: Source,
}

#[derive(Debug)]
pub enum Source {
    Shaarli {
        url: String,
        password: Secret<String>,
    },
}

#[derive(Clone)]
pub struct ImportBookmarkUseCase {
    bookmark_repository: Arc<dyn BookmarkRepository>,
    http_client: Client,
}

#[derive(Debug)]
pub enum ImportBookmarkError {
    Error(Error),
    ShaarliError(Error),
}

impl ImportBookmarkUseCase {
    pub fn new(bookmark_repository: Arc<dyn BookmarkRepository>, http_client: Client) -> Self {
        Self {
            bookmark_repository,
            http_client,
        }
    }
    pub async fn execute(&self, command: ImportBookmarkCommand) -> Result<(), ImportBookmarkError> {
        let bookmarks = match command.source {
            Source::Shaarli { url, password } => {
                let key: Hmac<Sha512> = Hmac::new_from_slice(password.expose_secret().as_bytes())
                    .context("Could not initialize hmac-sah512")
                    .map_err(ImportBookmarkError::Error)?;

                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let mut claims = BTreeMap::new();
                claims.insert("iat", timestamp.to_string());
                let token_str = claims
                    .sign_with_key(&key)
                    .context("Could not create token")
                    .map_err(ImportBookmarkError::Error)?;

                let mut auth_value =
                    header::HeaderValue::from_str(&format!("Bearer {}", token_str))
                        .context("Could not create header")
                        .map_err(ImportBookmarkError::Error)?;
                auth_value.set_sensitive(true);

                let bookmarks = self
                    .http_client
                    .get(format!("{}/api/v1/links?limit=all", url))
                    .header(header::AUTHORIZATION, auth_value)
                    .send()
                    .await
                    .context("Could not fetch bookmarks")
                    .map_err(ImportBookmarkError::ShaarliError)?
                    .json::<Vec<ShaarliBookmark>>()
                    .await
                    .context("Could not deserialize JSON")
                    .map_err(ImportBookmarkError::ShaarliError)?;

                bookmarks
                    .into_iter()
                    .map(|b| Bookmark {
                        id: None,
                        user_id: command.user_id,
                        url: b.url,
                        title: (!b.title.is_empty()).then_some(b.title),
                        description: (!b.description.is_empty()).then_some(b.description),
                        tags: {
                            let mut tags = b.tags;
                            tags.sort();
                            tags.dedup();
                            tags
                        },
                        creation_date: b.created,
                        update_date: (b.updated != b.created).then_some(b.updated),
                        private: b.private,
                        pinned: false,
                    })
                    .collect::<Vec<Bookmark>>()
            }
        };

        self.bookmark_repository
            .import(bookmarks)
            .await
            .context("Could not import bookmarks")
            .map_err(ImportBookmarkError::Error)
            .map(|_| ())
    }
}

#[derive(Deserialize, Debug)]
struct ShaarliBookmark {
    url: String,
    title: String,
    description: String,
    tags: Vec<String>,
    private: bool,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
}
