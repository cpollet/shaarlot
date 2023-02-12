mod json;

use crate::rest::json::Json;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use axum::Router;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct BookmarkResponse {
    id: u32,
    url: String,
    description: String,
    tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct CreateBookmarkRequest {
    url: String,
    description: String,
}

#[derive(Deserialize)]
pub struct UpdateBookmarkRequest {
    url: String,
    description: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/bookmarks", get(get_bookmarks))
        .route("/bookmarks", post(create_bookmark))
        .route("/bookmarks/:id", get(get_bookmark))
        .route("/bookmarks/:id", delete(delete_bookmark))
        .route("/bookmarks/:id", put(update_bookmark))
}

async fn get_bookmarks() -> Json<Vec<BookmarkResponse>> {
    Json(vec![
        BookmarkResponse {
            id: 1,
            url: "https://github.com".to_owned(),
            description: "GitHub.com".to_owned(),
            tags: vec!["dev".to_owned()],
        },
        BookmarkResponse {
            id: 2,
            url: "https://google.com".to_owned(),
            description: "Google".to_owned(),
            tags: vec![],
        },
    ])
}

async fn get_bookmark(Path(bookmark_id): Path<u32>) -> Json<BookmarkResponse> {
    Json(BookmarkResponse {
        id: bookmark_id,
        url: "https://youtube.com".to_owned(),
        description: "YouTube".to_owned(),
        tags: vec![],
    })
}

async fn create_bookmark(
    Json(bookmark): Json<CreateBookmarkRequest>,
) -> (StatusCode, Json<BookmarkResponse>) {
    (
        StatusCode::CREATED,
        Json(BookmarkResponse {
            id: 3,
            url: bookmark.url,
            description: bookmark.description,
            tags: vec![],
        }),
    )
}

async fn update_bookmark(
    Path(id): Path<u32>,
    Json(bookmark): Json<UpdateBookmarkRequest>,
) -> Json<BookmarkResponse> {
    Json(BookmarkResponse {
        id,
        url: bookmark.url,
        description: bookmark.description,
        tags: vec![],
    })
}

async fn delete_bookmark(Path(id): Path<u32>) -> StatusCode {
    StatusCode::NO_CONTENT
}
