use crate::database::tags::Query;
use crate::sessions::session::UserInfo;
use crate::AppState;
use axum::extract::State;
use axum::Extension;
use rest_api::tags::{GetTagsResult, Tag};

pub async fn get_tags(
    Extension(user_info): Extension<Option<UserInfo>>,
    State(state): State<AppState>,
) -> Result<GetTagsResult, GetTagsResult> {
    let tags = Query::find_by_user_id(&state.database, user_info.map(|u| u.id))
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
