mod error_response;
mod json;

use crate::rest::error_response::ErrorResponse;
use crate::rest::json::Json;
use crate::{database, AppState};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;
use qrcode_generator::QrCodeEcc;
use rest_api::*;
use std::collections::HashMap;
use std::str::FromStr;
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
        .with_state(state)
}

async fn get_bookmarks(
    State(state): State<AppState>,
) -> Result<Json<Vec<BookmarkResponse>>, (StatusCode, Json<ErrorResponse>)> {
    Ok(Json(
        database::bookmarks::Query::find_all(&state.database)
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
    let webpage = Webpage::from_url(&url, WebpageOptions::default()).map_err(|_| {
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
