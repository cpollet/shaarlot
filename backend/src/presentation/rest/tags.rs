use crate::application::get_tags::GetTagsCommand;
use crate::domain::values::tag::{CountedTag, Sort};

use crate::presentation::rest::UserInfo;
use crate::AppState;

use axum::extract::{Query, State};
use axum::Extension;
use rest_api::tags::{GetTagsResult, Tag};
use serde::Deserialize;

// todo review query param serialization and struct shared with API
#[derive(Deserialize)]
pub struct GetTagsQueryParams {
    order: Option<String>, // todo rename to sort
}

impl TryFrom<&str> for Sort {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "name" => Ok(Sort::NameAsc),
            "count" => Ok(Sort::CountAsc),
            _ => Err(format!("{} is not valid", value)),
        }
    }
}

impl From<CountedTag> for Tag {
    fn from(value: CountedTag) -> Self {
        Self {
            name: value.name,
            count: value.count,
        }
    }
}

pub async fn get_tags(
    Query(query): Query<GetTagsQueryParams>,
    Extension(user_info): Extension<Option<UserInfo>>,
    State(state): State<AppState>,
) -> Result<GetTagsResult, GetTagsResult> {
    let sort = query
        .order
        .map(|v| Sort::try_from(v.as_str()))
        .unwrap_or(Ok(Sort::CountAsc))
        .map_err(|_| {
            GetTagsResult::InvalidParameter(
                "Unsupported value provided for the 'sort' query parameter".to_string(),
            )
        })?;

    let tags = state
        .get_tags
        .execute(GetTagsCommand {
            user_id: user_info.map(|u| u.id),
            sort,
        })
        .await
        .map_err(|e| {
            log::error!("{:?}", e);
            GetTagsResult::ServerError
        })?
        .into_iter()
        .map(Tag::from)
        .collect::<Vec<Tag>>();

    Ok(GetTagsResult::Success(tags))
}
