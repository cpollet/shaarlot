mod error_response;
mod json;

use crate::database::bookmarks::SortOrder;
use crate::rest::error_response::ErrorResponse;
use crate::rest::json::Json;
use crate::{database, AppState};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;
use chrono::Utc;
use qrcode_generator::QrCodeEcc;
use rest_api::BookmarkResponse;
use rest_api::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use log::log;
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::{AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenUrl};
use oauth2::reqwest::{async_http_client, http_client};
use sea_orm::strum::Display;
use uuid::Uuid;
use webpage::{Webpage, WebpageOptions};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(URL_BOOKMARKS, get(get_bookmarks))
        .route(URL_BOOKMARKS, post(create_bookmark))
        .route(URL_BOOKMARK, get(get_bookmark))
        .route(URL_BOOKMARK, delete(delete_bookmark))
        .route(URL_BOOKMARK, put(update_bookmark))
        .route(URL_BOOKMARK_QRCODE, get(get_bookmark_qrcode))
        .route(URL_URLS, get(get_url))
        .route("/api/login", post(post_login))
        .route("/api/oauth/callback", get(oauth_callback))
        .with_state(state)
}

#[derive(Deserialize)]
struct GetBookmarksQueryParams {
    order: Option<String>,
}

async fn get_bookmarks(
    Query(query): Query<GetBookmarksQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<Vec<BookmarkResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let order = query
        .order
        .map(|v| SortOrder::try_from(v.as_str()))
        .unwrap_or(Ok(SortOrder::CreationDateDesc))
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "INVALID_PARAMETER",
                    "Unsupported value provided for the 'sort' query parameter",
                )),
            )
        })?;

    Ok(Json(
        database::bookmarks::Query::find_all_order_by(&state.database, order)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(
                        "CANNOT_READ_FROM_DATABASE",
                        "Cannot read bookmarks from database",
                    )),
                )
            })?
            .into_iter()
            .map(|m| BookmarkResponse {
                id: m.id,
                url: m.url,
                title: m.title,
                description: m.description,
                tags: vec![],
                creation_date: m.creation_date.with_timezone(&Utc),
                update_date: m.update_date.map(|d|d.with_timezone(&Utc)),
            })
            .collect(),
    ))
}

async fn get_bookmark(
    State(state): State<AppState>,
    Path(bookmark_id): Path<i32>,
) -> Result<Json<BookmarkResponse>, (StatusCode, Json<ErrorResponse>)> {
    database::bookmarks::Query::find_by_id(&state.database, bookmark_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    ErrorResponse::new(
                        "CANNOT_READ_FROM_DATABASE",
                        &format!("Cannot read bookmark '{}' from database", bookmark_id),
                    )
                    .with_data("id", &format!("{}", bookmark_id)),
                ),
            )
        })?
        .map(|m| {
            Json(BookmarkResponse {
                id: m.id,
                url: m.url,
                title: m.title,
                description: m.description,
                tags: vec![],
                creation_date: m.creation_date.with_timezone(&Utc),
                update_date: m.update_date.map(|d|d.with_timezone(&Utc)),
            })
        })
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(
                ErrorResponse::new(
                    "NOT_FOUND",
                    &format!("Bookmark '{}' not found", bookmark_id),
                )
                .with_data("id", &format!("{}", bookmark_id)),
            ),
        ))
}

async fn get_bookmark_qrcode(
    State(state): State<AppState>,
    Path(bookmark_id): Path<i32>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    let size = params
        .get("size")
        .map_or(Ok(256), |s| u32::from_str(s))
        .map_err(|e| {
            log::info!("{}", e);
            StatusCode::BAD_REQUEST
        })?;

    let model = database::bookmarks::Query::find_by_id(&state.database, bookmark_id)
        .await
        .map_err(|e| {
            log::error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let bytes =
        qrcode_generator::to_png_to_vec(model.url.as_bytes(), QrCodeEcc::Low, size as usize)
            .map_err(|e| {
                log::error!("{}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/x-png")
        .body(Body::from(bytes))
        .unwrap())
}

async fn create_bookmark(
    State(state): State<AppState>,
    Json(bookmark): Json<CreateBookmarkRequest>,
) -> Result<(StatusCode, Json<BookmarkResponse>), (StatusCode, Json<ErrorResponse>)> {
    database::bookmarks::Mutation::create_bookmark(
        &state.database,
        bookmark.url,
        bookmark.title,
        bookmark.description,
    )
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(
                "CANNOT_WRITE_TO_DATABASE",
                "Cannot save bookmark to database",
            )),
        )
    })
    .map(|m| {
        (
            StatusCode::CREATED,
            Json(BookmarkResponse {
                id: m.id,
                url: m.url,
                title: m.title,
                description: m.description,
                tags: vec![],
                creation_date: m.creation_date.with_timezone(&Utc),
                update_date: m.update_date.map(|d|d.with_timezone(&Utc)),
            }),
        )
    })
}

async fn update_bookmark(
    State(state): State<AppState>,
    Path(bookmark_id): Path<i32>,
    Json(bookmark): Json<UpdateBookmarkRequest>,
) -> Result<Json<BookmarkResponse>, (StatusCode, Json<ErrorResponse>)> {
    database::bookmarks::Mutation::update_bookmark(
        &state.database,
        bookmark_id,
        bookmark.url,
        bookmark.title,
        bookmark.description,
    )
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(
                "CANNOT_WRITE_TO_DATABASE",
                "Cannot save bookmark to database",
            )),
        )
    })?
    .map(|m| {
        Json(BookmarkResponse {
            id: bookmark_id,
            url: m.url,
            title: m.title,
            description: m.description,
            tags: vec![],
            creation_date: m.creation_date.with_timezone(&Utc),
            update_date: m.update_date.map(|d|d.with_timezone(&Utc)),
        })
    })
    .ok_or((
        StatusCode::NOT_FOUND,
        Json(
            ErrorResponse::new(
                "NOT_FOUND",
                &format!("Bookmark '{}' not found", bookmark_id),
            )
            .with_data("id", &format!("{}", bookmark_id)),
        ),
    ))
}

async fn delete_bookmark(
    State(state): State<AppState>,
    Path(bookmark_id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    database::bookmarks::Mutation::delete_bookmark(&state.database, bookmark_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "CANNOT_WRITE_TO_DATABASE",
                    "Cannot save bookmark to database",
                )),
            )
        })?
        .map(|_| StatusCode::NO_CONTENT)
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(
                ErrorResponse::new(
                    "NOT_FOUND",
                    &format!("Bookmark '{}' not found", bookmark_id),
                )
                .with_data("id", &format!("{}", bookmark_id)),
            ),
        ))
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

async fn post_login(State(state): State<AppState>) -> (StatusCode, Json<LoginResponse>) {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = state.oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .add_scope(Scope::new("user:email".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    log::info!("csrf: {}", csrf_token.secret());

    state.cache.insert(csrf_token.secret().to_string(), pkce_verifier.secret().to_string()).await;

    (StatusCode::CREATED, Json(LoginResponse{ url: auth_url.to_string() }))
}

#[derive(Deserialize)]
struct OAuthCallbackParams {
    code: String,
    state: String,
}

async fn oauth_callback(
    State(state):State<AppState>,
    Query(query): Query<OAuthCallbackParams>,
) -> Result<Json<BasicTokenResponse>, (StatusCode, Json<ErrorResponse>)>{
    let pkce_verifier = state.cache.get(&query.state)
        .map(|s| PkceCodeVerifier::new(s))
        .ok_or((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("INVALID_PARAMETERS", "Invalid parameters"))))?;

    let token = state.oauth_client.exchange_code(AuthorizationCode::new(query.code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client).await.unwrap();


    Ok(Json(token))
}
