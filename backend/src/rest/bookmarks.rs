use crate::database::bookmarks::SortOrder;
use crate::{database, AppState};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, Response, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use qrcode_generator::QrCodeEcc;
use rest_api::bookmarks::create::{
    CreateBookmarkRequest, CreateBookmarkResponse, CreateBookmarkResult,
};
use rest_api::bookmarks::delete::DeleteBookmarkResult;
use rest_api::bookmarks::get_many::{GetBookmarksResponse, GetBookmarksResult};
use rest_api::bookmarks::get_one::{GetBookmarkResponse, GetBookmarkResult};
use rest_api::bookmarks::update::{
    UpdateBookmarkRequest, UpdateBookmarkResponse, UpdateBookmarkResult,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Deserialize)]
pub struct GetBookmarksQueryParams {
    order: Option<String>,
}

pub async fn get_bookmarks(
    Query(query): Query<GetBookmarksQueryParams>,
    State(state): State<AppState>,
) -> Result<GetBookmarksResult, GetBookmarksResult> {
    let order = query
        .order
        .map(|v| SortOrder::try_from(v.as_str()))
        .unwrap_or(Ok(SortOrder::CreationDateDesc))
        .map_err(|_| {
            GetBookmarksResult::InvalidParameter(
                "Unsupported value provided for the 'sort' query parameter".to_string(),
            )
        })?;

    let bookmarks = database::bookmarks::Query::find_all_order_by(&state.database, order)
        .await
        .map_err(|_| GetBookmarksResult::ServerError)?
        .into_iter()
        .map(|m| GetBookmarkResponse {
            id: m.id,
            url: m.url,
            title: m.title,
            description: m.description,
            tags: vec![],
            creation_date: m.creation_date.with_timezone(&Utc),
            update_date: m.update_date.map(|d| d.with_timezone(&Utc)),
        })
        .collect::<GetBookmarksResponse>();

    Ok(GetBookmarksResult::Success(bookmarks))
}

pub async fn get_bookmark(
    State(state): State<AppState>,
    Path(bookmark_id): Path<i32>,
) -> Result<GetBookmarkResult, GetBookmarkResult> {
    let bookmark = database::bookmarks::Query::find_by_id(&state.database, bookmark_id)
        .await
        .map_err(|_| GetBookmarkResult::ServerError)?
        .map(|m| GetBookmarkResponse {
            id: m.id,
            url: m.url,
            title: m.title,
            description: m.description,
            tags: vec![],
            creation_date: m.creation_date.with_timezone(&Utc),
            update_date: m.update_date.map(|d| d.with_timezone(&Utc)),
        })
        .ok_or(GetBookmarkResult::NotFound(
            bookmark_id,
            format!("Bookmark '{}' not found", bookmark_id),
        ))?;

    Ok(GetBookmarkResult::Success(bookmark))
}

pub async fn get_bookmark_qrcode(
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

pub async fn create_bookmark(
    State(state): State<AppState>,
    Json(bookmark): Json<CreateBookmarkRequest>,
) -> Result<CreateBookmarkResult, CreateBookmarkResult> {
    let bookmark = database::bookmarks::Mutation::create_bookmark(
        &state.database,
        bookmark.url,
        bookmark.title,
        bookmark.description,
    )
    .await
    .map_err(|_| CreateBookmarkResult::ServerError)
    .map(|m| CreateBookmarkResponse {
        id: m.id,
        url: m.url,
        title: m.title,
        description: m.description,
        tags: vec![],
        creation_date: m.creation_date.with_timezone(&Utc),
        update_date: m.update_date.map(|d| d.with_timezone(&Utc)),
    })?;

    Ok(CreateBookmarkResult::Success(bookmark))
}

pub async fn update_bookmark(
    State(state): State<AppState>,
    Path(bookmark_id): Path<i32>,
    Json(bookmark): Json<UpdateBookmarkRequest>,
) -> Result<UpdateBookmarkResult, UpdateBookmarkResult> {
    let bookmark = database::bookmarks::Mutation::update_bookmark(
        &state.database,
        bookmark_id,
        bookmark.url,
        bookmark.title,
        bookmark.description,
    )
    .await
    .map_err(|_| UpdateBookmarkResult::ServerError)?
    .map(|m| UpdateBookmarkResponse {
        id: bookmark_id,
        url: m.url,
        title: m.title,
        description: m.description,
        tags: vec![],
        creation_date: m.creation_date.with_timezone(&Utc),
        update_date: m.update_date.map(|d| d.with_timezone(&Utc)),
    })
    .ok_or(UpdateBookmarkResult::NotFound(
        bookmark_id,
        format!("Bookmark '{}' not found", bookmark_id),
    ))?;

    Ok(UpdateBookmarkResult::Success(bookmark))
}

pub async fn delete_bookmark(
    State(state): State<AppState>,
    Path(bookmark_id): Path<i32>,
) -> Result<DeleteBookmarkResult, DeleteBookmarkResult> {
    database::bookmarks::Mutation::delete_bookmark(&state.database, bookmark_id)
        .await
        .map_err(|_| DeleteBookmarkResult::ServerError)?
        .map(|_| DeleteBookmarkResult::Success)
        .ok_or(DeleteBookmarkResult::NotFound(
            bookmark_id,
            format!("Bookmark '{}' not found", bookmark_id),
        ))
}
