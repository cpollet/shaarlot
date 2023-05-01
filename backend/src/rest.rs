mod application;
mod bookmarks;
mod emails;
mod json;
mod password_recoveries;
mod sessions;
mod shaarli_import_api;
mod tags;
mod users;

use crate::rest::application::get_application;
use crate::rest::bookmarks::*;
use crate::rest::emails::update_email;
use crate::rest::password_recoveries::{create_password_recovery, update_password_recovery};
use crate::rest::sessions::*;
use crate::rest::shaarli_import_api::shaarli_import_api;
use crate::rest::tags::get_tags;
use crate::rest::users::*;
use crate::sessions::session::{SessionHint, UserInfo};
use crate::{database, AppState};
use axum::extract::{Path, State};
use axum::middleware::from_fn;
use axum::routing::{delete, get, post, put};
use axum::{Extension, Router};
use axum_sessions::async_session::SessionStore;
use axum_sessions::{PersistencePolicy, SessionLayer};
use rest_api::application::URL_APPLICATION;
use rest_api::bookmarks::{URL_BOOKMARK, URL_BOOKMARKS_STATS};
use rest_api::bookmarks::{URL_BOOKMARKS, URL_BOOKMARK_QRCODE};
use rest_api::import_shaarli_api::URL_SHAARLI_IMPORT_API;
use rest_api::password_recoveries::URL_PASSWORD_RECOVERIES;
use rest_api::sessions::{URL_SESSIONS, URL_SESSIONS_CURRENT};
use rest_api::tags::URL_TAGS;
use rest_api::urls::{GetUrlConflictResponse, GetUrlResponse, GetUrlResult, URL_URLS};
use rest_api::users::{URL_CURRENT_USER, URL_USERS};
use rest_api::validate_email::URL_EMAIL;
use secrecy::{ExposeSecret, SecretVec};
use webpage::{Webpage, WebpageOptions};

pub struct Configuration<S>
where
    S: SessionStore,
{
    pub cookie_secret: SecretVec<u8>,
    pub session_store: S,
}

pub fn api_router<S>(configuration: &Configuration<S>, state: AppState) -> Router
where
    S: SessionStore,
{
    Router::new()
        .merge(
            Router::new()
                .route(URL_APPLICATION, get(get_application))
                .route(URL_PASSWORD_RECOVERIES, post(create_password_recovery))
                .route(URL_PASSWORD_RECOVERIES, put(update_password_recovery)),
        )
        .merge(
            Router::new()
                .route(URL_BOOKMARKS, post(create_bookmark))
                .route(URL_BOOKMARK, delete(delete_bookmark))
                .route(URL_BOOKMARK, put(update_bookmark))
                .route(URL_URLS, get(get_url))
                .route(URL_SHAARLI_IMPORT_API, post(shaarli_import_api))
                .layer(from_fn(SessionHint::required))
                .layer(
                    SessionLayer::new(
                        configuration.session_store.clone(),
                        configuration.cookie_secret.expose_secret().as_slice(),
                    )
                    .with_persistence_policy(PersistencePolicy::ExistingOnly),
                ),
        )
        .merge(
            Router::new()
                .route(URL_BOOKMARKS, get(get_bookmarks))
                .route(URL_BOOKMARK, get(get_bookmark))
                .route(URL_BOOKMARK_QRCODE, get(get_bookmark_qrcode))
                .route(URL_USERS, post(create_user))
                .route(URL_EMAIL, put(update_email))
                .route(URL_TAGS, get(get_tags))
                .route(URL_BOOKMARKS_STATS, get(get_bookmarks_stats))
                .layer(from_fn(SessionHint::supported))
                .layer(
                    SessionLayer::new(
                        configuration.session_store.clone(),
                        configuration.cookie_secret.expose_secret().as_slice(),
                    )
                    .with_persistence_policy(PersistencePolicy::ExistingOnly),
                ),
        )
        .merge(
            Router::new()
                .route(URL_SESSIONS, post(create_session))
                .route(URL_SESSIONS_CURRENT, delete(delete_current_session))
                .layer(
                    SessionLayer::new(
                        configuration.session_store.clone(),
                        configuration.cookie_secret.expose_secret().as_slice(),
                    )
                    .with_persistence_policy(PersistencePolicy::Always),
                ),
        )
        .merge(
            Router::new()
                .route(URL_SESSIONS_CURRENT, get(get_current_session))
                .route(URL_CURRENT_USER, get(get_current_user))
                .route(URL_CURRENT_USER, post(update_current_user))
                .layer(from_fn(SessionHint::required))
                .layer(
                    SessionLayer::new(
                        configuration.session_store.clone(),
                        configuration.cookie_secret.expose_secret().as_slice(),
                    )
                    .with_persistence_policy(PersistencePolicy::Always),
                ),
        )
        .with_state(state)
}

async fn get_url(
    Extension(user_info): Extension<UserInfo>,
    Path(url): Path<String>,
    State(state): State<AppState>,
) -> Result<GetUrlResult, GetUrlResult> {
    if let Some(id) = database::bookmarks::Query::find_by_url(&state.database, user_info.id, &url)
        .await
        .map_err(|_| GetUrlResult::ServerError)?
    {
        return Ok(GetUrlResult::Conflict(GetUrlConflictResponse { id }));
    }

    log::info!("Fetching metadata about {}", &url);

    let options = WebpageOptions {
        allow_insecure: true,
        ..WebpageOptions::default()
    };

    // todo use reqwest to fetch HTML instead of curl
    let webpage = Webpage::from_url(&url, options).map_err(|e| {
        log::error!("Error while fetching metadata about {}: {}", &url, e);
        GetUrlResult::ServerError
    })?;

    Ok(GetUrlResult::Success(GetUrlResponse {
        url: webpage.http.url,
        title: webpage.html.title,
        description: webpage.html.description,
    }))
}
