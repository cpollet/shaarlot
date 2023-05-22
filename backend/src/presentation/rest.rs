mod application;
mod bookmarks;
mod emails;
mod json;
mod password_recoveries;
mod sessions;
mod shaarli_import_api;
mod tags;
mod urls;
mod users;

use crate::domain::entities::account::ClearPassword;
use crate::presentation::rest::application::get_application;
use crate::presentation::rest::bookmarks::{
    create_bookmark, delete_bookmark, get_bookmark, get_bookmark_qrcode, get_bookmarks,
    get_bookmarks_stats, update_bookmark,
};
use crate::presentation::rest::emails::update_email;
use crate::presentation::rest::json::Json;
use crate::presentation::rest::password_recoveries::{
    create_password_recovery, update_password_recovery,
};
use crate::presentation::rest::sessions::{
    create_session, delete_current_session, get_current_session,
};
use crate::presentation::rest::shaarli_import_api::shaarli_import_api;
use crate::presentation::rest::tags::get_tags;
use crate::presentation::rest::urls::get_url;
use crate::presentation::rest::users::{create_user, get_current_user, update_current_user};
use crate::AppState;
use axum::http::{Request, StatusCode};
use axum::middleware::from_fn;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::Router;
use axum_sessions::async_session::SessionStore;
use axum_sessions::{PersistencePolicy, SessionHandle, SessionLayer};
use rest_api::application::URL_APPLICATION;
use rest_api::bookmarks::{URL_BOOKMARK, URL_BOOKMARKS_STATS};
use rest_api::bookmarks::{URL_BOOKMARKS, URL_BOOKMARK_QRCODE};
use rest_api::error_response::ErrorResponse;
use rest_api::import_shaarli_api::URL_SHAARLI_IMPORT_API;
use rest_api::password_recoveries::URL_PASSWORD_RECOVERIES;
use rest_api::sessions::{URL_SESSIONS, URL_SESSIONS_CURRENT};
use rest_api::tags::URL_TAGS;
use rest_api::urls::URL_URLS;
use rest_api::users::{URL_CURRENT_USER, URL_USERS};
use rest_api::validate_email::URL_EMAIL;
use rest_api::RestPassword;
use secrecy::{ExposeSecret, SecretVec};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub struct Configuration<S>
where
    S: SessionStore,
{
    pub cookie_secret: SecretVec<u8>,
    pub session_store: S,
}

pub const SESSION_KEY_USER_INFO: &str = "USER_INFO";

#[derive(Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
}

impl Display for UserInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[id:{}, username:{}]", self.id, self.username)
    }
}

pub struct SessionHint;

impl SessionHint {
    pub async fn required<B>(
        mut request: Request<B>,
        next: axum::middleware::Next<B>,
    ) -> impl IntoResponse {
        if let Some(session_handle) = request.extensions().get::<SessionHandle>() {
            let user_info = {
                session_handle
                    .read()
                    .await
                    .get::<UserInfo>(SESSION_KEY_USER_INFO)
            };
            if let Some(user_info) = user_info {
                log::info!("Requested session configured with user {}", user_info);
                request.extensions_mut().insert(user_info);
                return next.run(request).await;
            }
        } else {
            log::warn!("No session found");
        }

        (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "FORBIDDEN",
                "You're not allowed to access this resource",
            )),
        )
            .into_response()
    }

    pub async fn supported<B>(
        mut request: Request<B>,
        next: axum::middleware::Next<B>,
    ) -> Response {
        if let Some(session_handle) = request.extensions().get::<SessionHandle>() {
            let user_info = {
                session_handle
                    .read()
                    .await
                    .get::<UserInfo>(SESSION_KEY_USER_INFO)
            };
            if let Some(user_info) = user_info {
                log::info!("Supported session configured with user {}", user_info);
                request.extensions_mut().insert(Some(user_info));
            } else {
                request.extensions_mut().insert::<Option<UserInfo>>(None);
            }
        }
        next.run(request).await
    }
}

impl From<&RestPassword> for ClearPassword {
    fn from(value: &RestPassword) -> Self {
        ClearPassword::from(value.0.clone())
    }
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
