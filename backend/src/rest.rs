mod bookmarks;
pub mod error_response;
mod json;
mod sessions;
mod users;

use crate::rest::bookmarks::*;
use crate::rest::error_response::ErrorResponse;
use crate::rest::json::Json;
use crate::rest::sessions::*;
use crate::rest::users::*;
use crate::session::Session;
use crate::AppState;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use axum::routing::{delete, get, post, put};
use axum::Router;
use axum_sessions::async_session::SessionStore;
use axum_sessions::{PersistencePolicy, SessionLayer};
use rest_api::*;
use secrecy::{ExposeSecret, SecretVec};
use webpage::{Webpage, WebpageOptions};

pub struct Configuration<S>
where
    S: SessionStore,
{
    pub cookie_secret: SecretVec<u8>,
    pub session_store: S,
}

pub fn router<S>(configuration: &Configuration<S>, state: AppState) -> Router
where
    S: SessionStore,
{
    Router::new()
        .merge(
            Router::new()
                .route(URL_BOOKMARKS, post(create_bookmark))
                .route(URL_BOOKMARK, delete(delete_bookmark))
                .route(URL_BOOKMARK, put(update_bookmark))
                .route(URL_URLS, get(get_url))
                .layer(from_fn(Session::required))
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
                .layer(from_fn(Session::supported))
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
                .layer(from_fn(Session::required))
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
    Path(url): Path<String>,
) -> Result<Json<UrlResponse>, (StatusCode, Json<ErrorResponse>)> {
    log::info!("Fetching metadata about {}", &url);

    let mut options = WebpageOptions::default();
    options.allow_insecure = true;

    let webpage = Webpage::from_url(&url, options).map_err(|e| {
        log::error!("Error while fetching metadata about {}: {}", &url, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(
                "CANNOT_FETCH_DATA",
                "Cannot fetch remote URL data",
            )),
        )
    })?;

    Ok(Json(UrlResponse {
        url: webpage.http.url,
        title: webpage.html.title,
        description: webpage.html.description,
    }))
}
