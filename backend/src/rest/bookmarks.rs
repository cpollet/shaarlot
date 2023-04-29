use crate::database::bookmarks::{Filter, Pagination, SearchCriteria, SortOrder};
use crate::sessions::session::UserInfo;
use crate::{database, AppState};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, Response, StatusCode};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use chrono::Utc;
use entity::bookmark::Model as BookmarkModel;
use entity::tag::Model as TagModel;
use qrcode_generator::QrCodeEcc;
use rest_api::bookmarks::create::{CreateBookmarkRequest, CreateBookmarkResult};
use rest_api::bookmarks::delete::DeleteBookmarkResult;
use rest_api::bookmarks::get_many::{GetBookmarksResponse, GetBookmarksResult};
use rest_api::bookmarks::get_one::{GetBookmarkResponse, GetBookmarkResult};
use rest_api::bookmarks::update::{UpdateBookmarkRequest, UpdateBookmarkResult};
use rest_api::bookmarks::{Access, GetBookmarksStatsResponse, GetBookmarksStatsResult};
use sea_orm::{DbErr, TransactionTrait};
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use urlencoding::decode;

// todo review query param serialization and struct shared with API
#[derive(Deserialize)]
pub struct GetBookmarksQueryParams {
    order: Option<String>,
    page: Option<u64>,
    count: Option<u64>,
    tags: Option<String>,
    search: Option<String>,
    filter: Option<String>,
}

fn into_response(
    bookmark: BookmarkModel,
    tags: Vec<TagModel>,
    user: Option<&UserInfo>,
) -> GetBookmarkResponse {
    GetBookmarkResponse {
        id: bookmark.id,
        url: bookmark.url,
        title: bookmark.title,
        description: bookmark.description,
        tags: tags.into_iter().map(|e| e.name).collect::<Vec<String>>(),
        creation_date: bookmark.creation_date.with_timezone(&Utc),
        update_date: bookmark.update_date.map(|d| d.with_timezone(&Utc)),
        user_id: bookmark.user_id,
        access: if user.map(|u| bookmark.user_id == u.id).unwrap_or_default() {
            Access::Write
        } else {
            Access::Read
        },
        private: bookmark.private,
    }
}

pub async fn get_bookmarks(
    Query(query): Query<GetBookmarksQueryParams>,
    Extension(user_info): Extension<Option<UserInfo>>,
    State(state): State<AppState>,
) -> Result<GetBookmarksResult, GetBookmarksResult> {
    let criteria = SearchCriteria {
        user_id: user_info.as_ref().map(|u| u.id),
        tags: query
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
            .unwrap_or_default(),
        search: query
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
            .unwrap_or_default(),
        filter: query
            .filter
            // todo: no manual deserialize
            .map(|v| Filter::try_from(v.as_str()))
            .unwrap_or(Ok(Filter::All))
            .map_err(|_| {
                GetBookmarksResult::InvalidParameter(
                    "Unsupported value provided for the 'sort' query parameter".to_string(),
                )
            })?,
    };

    let page = Pagination {
        page: query.page.unwrap_or_default(),
        size: query.count.unwrap_or(20).min(100),
    };

    let order = query
        .order
        // todo: no manual deserialize
        .map(|v| SortOrder::try_from(v.as_str()))
        .unwrap_or(Ok(SortOrder::CreationDateDesc))
        .map_err(|_| {
            GetBookmarksResult::InvalidParameter(
                "Unsupported value provided for the 'sort' query parameter".to_string(),
            )
        })?;

    let bookmarks = database::bookmarks::Query::find(&state.database, &criteria, &page, order)
        .await
        .map_err(|_| GetBookmarksResult::ServerError)?
        .into_iter()
        .map(|m| into_response(m.0, m.1, user_info.as_ref()))
        .collect::<Vec<GetBookmarkResponse>>();

    let bookmarks_count = database::bookmarks::Query::count(&state.database, &criteria)
        .await
        .map_err(|_| GetBookmarksResult::ServerError)?;

    Ok(GetBookmarksResult::Success(GetBookmarksResponse {
        bookmarks,
        pages_count: (bookmarks_count as f64 / page.size as f64).ceil() as u64,
    }))
}

pub async fn get_bookmark(
    State(state): State<AppState>,
    Extension(user_info): Extension<Option<UserInfo>>,
    Path(bookmark_id): Path<i32>,
) -> Result<GetBookmarkResult, GetBookmarkResult> {
    let bookmark = database::bookmarks::Query::find_visible_by_id(
        &state.database,
        bookmark_id,
        user_info.as_ref().map(|u| u.id),
    )
    .await
    .map_err(|_| GetBookmarkResult::ServerError)?
    .map(|m| into_response(m.0, m.1, user_info.as_ref()))
    .ok_or(GetBookmarkResult::NotFound(
        bookmark_id,
        format!("Bookmark '{}' not found", bookmark_id),
    ))?;

    Ok(GetBookmarkResult::Success(bookmark))
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

    let model = database::bookmarks::Query::find_visible_by_id(
        &state.database,
        bookmark_id,
        user_info.as_ref().map(|u| u.id),
    )
    .await
    .map_err(|e| {
        log::error!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?
    .0;

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
    Extension(user_info): Extension<UserInfo>,
    Json(bookmark): Json<CreateBookmarkRequest>,
) -> Result<CreateBookmarkResult, CreateBookmarkResult> {
    let bookmark_id = state
        .database
        .transaction::<_, i32, DbErr>(|txn| {
            Box::pin(async move {
                let tags = {
                    let mut tags = Vec::new();
                    for tag in bookmark.tags {
                        tags.push(
                            database::tags::Mutation::create_tag(txn, tag.to_lowercase()).await?,
                        )
                    }
                    tags
                };

                let bookmark = database::bookmarks::Mutation::create_bookmark(
                    txn,
                    bookmark.url,
                    bookmark.title,
                    bookmark.description,
                    user_info.id,
                    bookmark.private,
                )
                .await?;

                for tag in tags {
                    database::bookmarks_tags::Mutation::create_link(txn, bookmark.id, tag.id)
                        .await?;
                }

                Ok(bookmark.id)
            })
        })
        .await
        .map_err(|_| CreateBookmarkResult::ServerError)?;

    let bookmark = database::bookmarks::Query::find_by_id(&state.database, bookmark_id)
        .await
        .map_err(|_| CreateBookmarkResult::ServerError)?
        .map(|m| into_response(m.0, m.1, Some(user_info).as_ref()))
        .ok_or(CreateBookmarkResult::ServerError)?;

    Ok(CreateBookmarkResult::Success(bookmark))
}

pub async fn update_bookmark(
    State(state): State<AppState>,
    Path(bookmark_id): Path<i32>,
    Extension(user_info): Extension<UserInfo>,
    Json(bookmark): Json<UpdateBookmarkRequest>,
) -> Result<UpdateBookmarkResult, UpdateBookmarkResult> {
    if database::bookmarks::Query::find_by_id(&state.database, bookmark_id)
        .await
        .map_err(|_| UpdateBookmarkResult::ServerError)?
        .ok_or(UpdateBookmarkResult::NotFound(
            bookmark_id,
            format!("Bookmark '{}' not found", bookmark_id),
        ))?
        .0
        .user_id
        != user_info.id
    {
        return Err(UpdateBookmarkResult::Forbidden);
    }

    state
        .database
        .transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                let tags = {
                    let mut tags = Vec::new();
                    for tag in bookmark.tags {
                        tags.push(
                            database::tags::Mutation::create_tag(txn, tag.to_lowercase()).await?,
                        )
                    }
                    tags
                };

                database::bookmarks_tags::Mutation::delete_all_links(txn, bookmark_id).await?;

                for tag in tags {
                    database::bookmarks_tags::Mutation::create_link(txn, bookmark_id, tag.id)
                        .await?;
                }

                database::tags::Mutation::delete_orphans(txn).await?;

                database::bookmarks::Mutation::update_bookmark(
                    txn,
                    bookmark_id,
                    bookmark.url,
                    bookmark.title,
                    bookmark.description,
                    bookmark.private,
                )
                .await?;

                Ok(())
            })
        })
        .await
        .map_err(|_| UpdateBookmarkResult::ServerError)?;

    let bookmark = database::bookmarks::Query::find_by_id(&state.database, bookmark_id)
        .await
        .map_err(|_| UpdateBookmarkResult::ServerError)?
        .map(|m| into_response(m.0, m.1, Some(user_info).as_ref()))
        .ok_or(UpdateBookmarkResult::NotFound(
            bookmark_id,
            format!("Bookmark '{}' not found", bookmark_id),
        ))?;

    Ok(UpdateBookmarkResult::Success(bookmark))
}

pub async fn delete_bookmark(
    State(state): State<AppState>,
    Path(bookmark_id): Path<i32>,
    Extension(user_info): Extension<UserInfo>,
) -> Result<DeleteBookmarkResult, DeleteBookmarkResult> {
    if database::bookmarks::Query::find_by_id(&state.database, bookmark_id)
        .await
        .map_err(|_| DeleteBookmarkResult::ServerError)?
        .ok_or(DeleteBookmarkResult::NotFound(
            bookmark_id,
            format!("Bookmark '{}' not found", bookmark_id),
        ))?
        .0
        .user_id
        != user_info.id
    {
        return Err(DeleteBookmarkResult::Forbidden);
    };

    state
        .database
        .transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                database::bookmarks_tags::Mutation::delete_all_links(txn, bookmark_id).await?;
                database::tags::Mutation::delete_orphans(txn).await?;
                database::bookmarks::Mutation::delete_bookmark(txn, bookmark_id).await?;
                Ok(())
            })
        })
        .await
        .map_err(|_| DeleteBookmarkResult::ServerError)?;

    Ok(DeleteBookmarkResult::Success)
}

pub async fn get_bookmarks_stats(
    Extension(user_info): Extension<Option<UserInfo>>,
    State(state): State<AppState>,
) -> Result<GetBookmarksStatsResult, GetBookmarksStatsResult> {
    let user_id = user_info.map(|u| u.id);

    let visible = database::bookmarks::Query::count(
        &state.database,
        &SearchCriteria {
            user_id,
            ..SearchCriteria::default()
        },
    )
    .await
    .map_err(|_| GetBookmarksStatsResult::ServerError)?;

    let private = database::bookmarks::Query::count(
        &state.database,
        &SearchCriteria {
            user_id,
            filter: Filter::Private,
            ..SearchCriteria::default()
        },
    )
    .await
    .map_err(|_| GetBookmarksStatsResult::ServerError)?;

    Ok(GetBookmarksStatsResult::Success(
        GetBookmarksStatsResponse {
            count_total: visible as u64,
            count_private: private as u64,
        },
    ))
}
