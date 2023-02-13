mod error_response;
mod json;

use crate::database;
use crate::rest::error_response::ErrorResponse;
use crate::rest::json::Json;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use axum::{Extension, Router};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct BookmarkResponse {
    id: i32,
    url: String,
    title: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct CreateBookmarkRequest {
    url: String,
    title: Option<String>,
    description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateBookmarkRequest {
    url: String,
    title: Option<String>,
    description: Option<String>,
}

pub fn router(database: DatabaseConnection) -> Router {
    Router::new()
        .route("/bookmarks", get(get_bookmarks))
        .route("/bookmarks", post(create_bookmark))
        .route("/bookmarks/:id", get(get_bookmark))
        .route("/bookmarks/:id", delete(delete_bookmark))
        .route("/bookmarks/:id", put(update_bookmark))
        .layer(Extension(database))
}

async fn get_bookmarks(
    Extension(database): Extension<DatabaseConnection>,
) -> Result<Json<Vec<BookmarkResponse>>, (StatusCode, Json<ErrorResponse>)> {
    Ok(Json(
        database::bookmarks::Query::find_all(&database)
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
    Extension(database): Extension<DatabaseConnection>,
    Path(bookmark_id): Path<i32>,
) -> Result<Json<BookmarkResponse>, (StatusCode, Json<ErrorResponse>)> {
    database::bookmarks::Query::find_by_id(&database, bookmark_id)
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

async fn create_bookmark(
    Extension(database): Extension<DatabaseConnection>,
    Json(bookmark): Json<CreateBookmarkRequest>,
) -> Result<(StatusCode, Json<BookmarkResponse>), (StatusCode, Json<ErrorResponse>)> {
    database::bookmarks::Mutation::create_bookmark(
        &database,
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
    Extension(database): Extension<DatabaseConnection>,
    Path(bookmark_id): Path<i32>,
    Json(bookmark): Json<UpdateBookmarkRequest>,
) -> Result<Json<BookmarkResponse>, (StatusCode, Json<ErrorResponse>)> {
    database::bookmarks::Mutation::update_bookmark(
        &database,
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
    Extension(database): Extension<DatabaseConnection>,
    Path(bookmark_id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    database::bookmarks::Mutation::delete_bookmark(&database, bookmark_id)
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
