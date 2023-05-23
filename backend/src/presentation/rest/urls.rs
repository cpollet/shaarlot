use crate::application::get_url_details::{GetUrlDetailsCommand, GetUrlDetailsResult};

use crate::presentation::rest::UserInfo;
use crate::AppState;

use axum::extract::{Path, State};
use axum::Extension;
use rest_api::urls::{GetUrlConflictResponse, GetUrlResponse, GetUrlResult};


pub async fn get_url(
    Extension(user_info): Extension<UserInfo>,
    Path(url): Path<String>,
    State(state): State<AppState>,
) -> Result<GetUrlResult, GetUrlResult> {
    let url =
        crate::url::clean(url, &state.ignored_query_params).ok_or(GetUrlResult::InvalidUrl)?;

    let url_details = state
        .get_url_details
        .execute(GetUrlDetailsCommand {
            user_id: user_info.id,
            url: &url,
        })
        .await
        .map_err(|e| {
            log::error!("{:?}", e);
            GetUrlResult::Success(GetUrlResponse {
                url,
                title: None,
                description: None,
            })
        })?;

    match url_details {
        GetUrlDetailsResult::Exists(id) => {
            Ok(GetUrlResult::Conflict(GetUrlConflictResponse { id }))
        }
        GetUrlDetailsResult::UrlDetails {
            url,
            title,
            description,
        } => Ok(GetUrlResult::Success(GetUrlResponse {
            url,
            title,
            description,
        })),
    }
}
