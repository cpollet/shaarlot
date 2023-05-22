use crate::application::create_bookmark::CreateBookmarkCommand;
use crate::application::delete_bookmark::{DeleteBookmarkCommand, DeleteResult};
use crate::application::find_bookmark::FindBookmarkCommand;
use crate::application::get_bookmark_stats::GetBookmarksStatsCommand;
use crate::application::search_bookmarks::SearchBookmarkCommand;
use crate::application::update_bookmark::{UpdateBookmarkCommand, UpdateResult};
use crate::domain::entities::bookmark::{Bookmark, Filter, Pagination, Sort};
use crate::presentation::rest::UserInfo;
use crate::AppState;
use anyhow::Context;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, Response, StatusCode};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use qrcode_generator::QrCodeEcc;
use rest_api::bookmarks::create::{CreateBookmarkRequest, CreateBookmarkResult};
use rest_api::bookmarks::delete::DeleteBookmarkResult;
use rest_api::bookmarks::get_many::{GetBookmarksResponse, GetBookmarksResult};
use rest_api::bookmarks::get_one::{GetBookmarkResponse, GetBookmarkResult};
use rest_api::bookmarks::update::{UpdateBookmarkRequest, UpdateBookmarkResult};
use rest_api::bookmarks::{Access, GetBookmarksStatsResponse, GetBookmarksStatsResult};
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use urlencoding::decode;

// todo review query param serialization and struct shared with API
#[derive(Deserialize)]
pub struct GetBookmarksQueryParams {
    order: Option<String>, // todo rename to sort
    page: Option<u64>,
    count: Option<u64>,
    tags: Option<String>,
    search: Option<String>,
    filter: Option<String>,
}

fn into_response(
    bookmark: Bookmark,
    remote_user: Option<&UserInfo>,
) -> anyhow::Result<GetBookmarkResponse> {
    let access = access(&bookmark, remote_user.map(|u| u.id));

    Ok(GetBookmarkResponse {
        id: bookmark.id.context("No ID found")?,
        url: bookmark.url,
        title: bookmark.title,
        description: bookmark.description,
        tags: bookmark.tags,
        creation_date: bookmark.creation_date,
        update_date: bookmark.update_date,
        user_id: bookmark.user_id,
        access,
        private: bookmark.private,
        pinned: bookmark.pinned,
    })
}

fn access(bookmark: &Bookmark, user_id: Option<i32>) -> Access {
    if user_id.map(|id| bookmark.is_owner(id)).unwrap_or_default() {
        Access::Write
    } else {
        Access::Read
    }
}

impl TryFrom<&str> for Filter {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "" => Ok(Filter::All),
            "private" => Ok(Filter::Private),
            "public" => Ok(Filter::Public),
            _ => Err(format!("{} is not valid", value)),
        }
    }
}

impl TryFrom<&str> for Sort {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "creation_date:asc" => Ok(Sort::CreationDateAsc),
            "creation_date:desc" => Ok(Sort::CreationDateDesc),
            _ => Err(format!("{} is not valid", value)),
        }
    }
}

pub async fn get_bookmarks(
    Query(query): Query<GetBookmarksQueryParams>,
    Extension(user_info): Extension<Option<UserInfo>>,
    State(state): State<AppState>,
) -> Result<GetBookmarksResult, GetBookmarksResult> {
    let tags = query
        .tags
        // todo: no manual deserialize
        .map(|tags| {
            tags.split('+')
                .map(decode)
                .map(|t| t.unwrap_or_default())
                .filter(|t| !t.is_empty())
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    let search = query
        .search
        // todo: no manual deserialize
        .map(|tags| {
            tags.split('+')
                .map(decode)
                .map(|t| t.unwrap_or_default())
                .filter(|t| !t.is_empty())
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    let filter = query
        .filter
        // todo: no manual deserialize
        .map(|v| Filter::try_from(v.as_str()))
        .unwrap_or(Ok(Filter::All))
        .map_err(|_| {
            GetBookmarksResult::InvalidParameter(
                "Unsupported value provided for the 'filter' query parameter".to_string(),
            )
        })?;
    let pagination = Pagination {
        page: query.page.unwrap_or_default(),
        size: query.count.unwrap_or(20).min(100),
    };
    let sort = query
        .order
        // todo: no manual deserialize
        .map(|v| Sort::try_from(v.as_str()))
        .unwrap_or(Ok(Sort::CreationDateDesc))
        .map_err(|_| {
            GetBookmarksResult::InvalidParameter(
                "Unsupported value provided for the 'sort' query parameter".to_string(),
            )
        })?;

    let search_result = state
        .search_bookmarks
        .execute(SearchBookmarkCommand {
            user_id: user_info.as_ref().map(|u| u.id),
            tags,
            search,
            filter,
            pagination: pagination.clone(),
            sort,
        })
        .await
        .map_err(|_| GetBookmarksResult::ServerError)?;

    let mut bookmarks = Vec::with_capacity(search_result.bookmarks.len());
    for bookmark in search_result.bookmarks {
        bookmarks.push(
            into_response(bookmark, user_info.as_ref())
                .map_err(|_| GetBookmarksResult::ServerError)?,
        );
    }

    Ok(GetBookmarksResult::Success(GetBookmarksResponse {
        bookmarks,
        pages_count: (search_result.total_count as f64 / pagination.size as f64).ceil() as u64,
    }))
}

pub async fn get_bookmark(
    State(state): State<AppState>,
    Extension(user_info): Extension<Option<UserInfo>>,
    Path(bookmark_id): Path<i32>,
) -> Result<GetBookmarkResult, GetBookmarkResult> {
    let bookmark = state
        .find_bookmark
        .execute(FindBookmarkCommand {
            user_id: user_info.as_ref().map(|u| u.id),
            bookmark_id,
        })
        .await
        .map_err(|_| GetBookmarkResult::ServerError)?
        .ok_or(GetBookmarkResult::NotFound(
            bookmark_id,
            format!("Bookmark '{}' not found", bookmark_id),
        ))?;

    into_response(bookmark, user_info.as_ref())
        .map_err(|_| GetBookmarkResult::ServerError)
        .map(GetBookmarkResult::Success)
}

pub async fn get_bookmark_qrcode(
    State(state): State<AppState>,
    Extension(user_info): Extension<Option<UserInfo>>,
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

    let bookmark = state
        .find_bookmark
        .execute(FindBookmarkCommand {
            user_id: user_info.as_ref().map(|u| u.id),
            bookmark_id,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let bytes =
        qrcode_generator::to_png_to_vec(bookmark.url.as_bytes(), QrCodeEcc::Low, size as usize)
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
    Extension(user_info): Extension<UserInfo>,
    Json(bookmark): Json<CreateBookmarkRequest>,
) -> Result<CreateBookmarkResult, CreateBookmarkResult> {
    state
        .create_bookmark
        .execute(CreateBookmarkCommand {
            user_id: user_info.id,
            url: bookmark.url.clone(),
            title: bookmark.title.clone(),
            description: bookmark.description.clone(),
            tags: bookmark.tags.clone().unwrap_or_default(),
            private: bookmark.private.unwrap_or_default(),
        })
        .await
        .and_then(|b| into_response(b, Some(user_info).as_ref()))
        .map_err(|_| CreateBookmarkResult::ServerError)
        .map(CreateBookmarkResult::Success)
}

pub async fn update_bookmark(
    State(state): State<AppState>,
    Path(bookmark_id): Path<i32>,
    Extension(user_info): Extension<UserInfo>,
    Json(bookmark): Json<UpdateBookmarkRequest>,
) -> Result<UpdateBookmarkResult, UpdateBookmarkResult> {
    let result = state
        .update_bookmark
        .execute(UpdateBookmarkCommand {
            id: bookmark_id,
            user_id: user_info.id,
            url: bookmark.url,
            title: bookmark.title,
            description: bookmark.description,
            tags: bookmark.tags,
            private: bookmark.private,
            pinned: bookmark.pinned,
        })
        .await
        .map_err(|_| UpdateBookmarkResult::ServerError)?
        .ok_or(UpdateBookmarkResult::NotFound(
            bookmark_id,
            format!("Bookmark '{}' not found", bookmark_id),
        ))?;

    let bookmark = match result {
        UpdateResult::Updated(bookmark) => bookmark,
        UpdateResult::Forbidden => return Err(UpdateBookmarkResult::Forbidden),
    };

    into_response(bookmark, Some(user_info).as_ref())
        .map_err(|_| UpdateBookmarkResult::ServerError)
        .map(UpdateBookmarkResult::Success)
}

pub async fn delete_bookmark(
    State(state): State<AppState>,
    Path(bookmark_id): Path<i32>,
    Extension(user_info): Extension<UserInfo>,
) -> Result<DeleteBookmarkResult, DeleteBookmarkResult> {
    let result = state
        .delete_bookmark
        .execute(DeleteBookmarkCommand {
            user_id: user_info.id,
            bookmark_id,
        })
        .await
        .map_err(|_| DeleteBookmarkResult::ServerError)?
        .ok_or(DeleteBookmarkResult::NotFound(
            bookmark_id,
            format!("Bookmark '{}' not found", bookmark_id),
        ))?;

    match result {
        DeleteResult::Deleted => Ok(DeleteBookmarkResult::Success),
        DeleteResult::Forbidden => Err(DeleteBookmarkResult::Forbidden),
    }
}

pub async fn get_bookmarks_stats(
    Extension(user_info): Extension<Option<UserInfo>>,
    State(state): State<AppState>,
) -> Result<GetBookmarksStatsResult, GetBookmarksStatsResult> {
    state
        .get_bookmarks_stats
        .execute(GetBookmarksStatsCommand {
            user_id: user_info.map(|u| u.id),
        })
        .await
        .map_err(|_| GetBookmarksStatsResult::ServerError)
        .map(|s| {
            GetBookmarksStatsResult::Success(GetBookmarksStatsResponse {
                count_total: s.total,
                count_private: s.private,
            })
        })
}
