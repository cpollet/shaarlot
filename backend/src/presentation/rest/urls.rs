use crate::infrastructure::database;
use crate::presentation::rest::UserInfo;
use crate::AppState;
use axum::extract::{Path, State};
use axum::Extension;
use rest_api::urls::{GetUrlConflictResponse, GetUrlResponse, GetUrlResult};
use webpage::HTML;

pub async fn get_url(
    Extension(user_info): Extension<UserInfo>,
    Path(url): Path<String>,
    State(state): State<AppState>,
) -> Result<GetUrlResult, GetUrlResult> {
    if let Some(id) = database::bookmarks::Query::find_by_url(&state.database, user_info.id, &url)
        .await
        .map_err(|_| GetUrlResult::ServerError)?
    {
        return Ok(GetUrlResult::Conflict(GetUrlConflictResponse { id }));
    }

    let url =
        crate::url::clean(url, &state.ignored_query_params).ok_or(GetUrlResult::InvalidUrl)?;
    log::info!("Fetching metadata about {}", &url);

    let response = state.http_client.get(&url).send().await.map_err(|e| {
        log::error!("{:?}", e);
        GetUrlResult::Success(GetUrlResponse {
            url,
            title: None,
            description: None,
        })
    })?;

    let url = response.url().to_string();

    let html = response.text().await.map_err(|e| {
        log::error!("{:?}", e);
        GetUrlResult::Success(GetUrlResponse {
            url: url.clone(),
            title: None,
            description: None,
        })
    })?;

    let html = HTML::from_string(html, None).map_err(|e| {
        log::error!("{:?}", e);
        GetUrlResult::Success(GetUrlResponse {
            url: url.clone(),
            title: None,
            description: None,
        })
    })?;

    Ok(GetUrlResult::Success(GetUrlResponse {
        url,
        title: html.title,
        description: html.description,
    }))
}
