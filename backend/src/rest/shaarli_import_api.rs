use crate::database::{bookmarks, bookmarks_tags, tags};
use crate::rest::json::Json;
use crate::sessions::session::UserInfo;
use crate::AppState;
use axum::extract::State;
use axum::Extension;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use reqwest::{header, Client};
use rest_api::import_shaarli_api::{ShaarliImportApiRequest, ShaarliImportApiResult};
use sea_orm::{DbErr, TransactionTrait};
use secrecy::ExposeSecret;
use serde::Deserialize;
use sha2::Sha512;
use std::collections::BTreeMap;
use std::time::SystemTime;

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

pub async fn shaarli_import_api(
    State(state): State<AppState>,
    Extension(user_info): Extension<UserInfo>,
    Json(bookmark): Json<ShaarliImportApiRequest>,
) -> Result<ShaarliImportApiResult, ShaarliImportApiResult> {
    let key: Hmac<Sha512> = Hmac::new_from_slice(bookmark.key.expose_secret().0.as_bytes())
        .map_err(|_| ShaarliImportApiResult::ServerError)?;

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut claims = BTreeMap::new();
    claims.insert("iat", timestamp.to_string());
    let token_str = claims
        .sign_with_key(&key)
        .map_err(|_| ShaarliImportApiResult::ServerError)?;

    // tracing::info!("{}", &token_str);

    let mut auth_value = header::HeaderValue::from_str(&format!("Bearer {}", token_str))
        .map_err(|_| ShaarliImportApiResult::ServerError)?;
    auth_value.set_sensitive(true);

    let mut headers = header::HeaderMap::new();
    headers.insert(header::AUTHORIZATION, auth_value);

    let client = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|_| ShaarliImportApiResult::ServerError)?;

    let bookmarks = client
        .get(format!("{}/api/v1/links?limit=all", bookmark.url))
        .send()
        .await
        .map_err(|_| ShaarliImportApiResult::ShaarliError)?
        .json::<Vec<ShaarliBookmark>>()
        .await
        .map_err(|_| ShaarliImportApiResult::ShaarliError)?;

    state
        .database
        .transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                for bookmark in bookmarks {
                    let bookmark_id = bookmarks::Mutation::import_bookmark(
                        txn,
                        bookmark.url.clone(),
                        (!bookmark.title.is_empty()).then_some(bookmark.title),
                        (!bookmark.description.is_empty()).then_some(bookmark.description),
                        bookmark.created,
                        (bookmark.updated != bookmark.created).then_some(bookmark.updated),
                        user_info.id,
                        bookmark.private,
                    )
                    .await?
                    .id;

                    let unique_tags = {
                        let mut tags = bookmark.tags;
                        tags.sort();
                        tags.dedup();
                        tags
                    };
                    for tag in unique_tags {
                        let tag_id = tags::Mutation::create_tag(txn, tag.to_lowercase())
                            .await?
                            .id;

                        bookmarks_tags::Mutation::create_link(txn, bookmark_id, tag_id).await?;
                    }
                }

                Ok(())
            })
        })
        .await
        .map_err(|_| ShaarliImportApiResult::ServerError)?;

    Ok(ShaarliImportApiResult::Success)
}
