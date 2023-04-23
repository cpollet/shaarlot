use crate::database::tags;
use crate::database::tags::SortOrder;
use crate::sessions::session::UserInfo;
use crate::AppState;
use axum::extract::{Query, State};
use axum::Extension;
use rest_api::tags::{GetTagsResult, Tag};
use serde::Deserialize;

// todo review query param serialization and struct shared with API
#[derive(Deserialize)]
pub struct GetTagsQueryParams {
    order: Option<String>,
}

pub async fn get_tags(
    Query(query): Query<GetTagsQueryParams>,
    Extension(user_info): Extension<Option<UserInfo>>,
    State(state): State<AppState>,
) -> Result<GetTagsResult, GetTagsResult> {
    let order = query
        .order
        // todo review query param serialization and struct shared with API
        .map(|v| SortOrder::try_from(v.as_str()))
        .unwrap_or(Ok(SortOrder::Count))
        .map_err(|_| {
            GetTagsResult::InvalidParameter(
                "Unsupported value provided for the 'sort' query parameter".to_string(),
            )
        })?;

    let tags =
        tags::Query::find_by_user_id_order_by(&state.database, user_info.map(|u| u.id), order)
            .await
            .map_err(|e| {
                tracing::error!("{}", e);
                GetTagsResult::ServerError
            })?
            .into_iter()
            .map(|t| Tag {
                name: t.name.to_lowercase(),
                count: t.count as i32,
            })
            .collect::<Vec<Tag>>();

    Ok(GetTagsResult::Success(tags))
}
