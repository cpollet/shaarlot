use crate::application::import_bookmarks::{ImportBookmarkCommand, ImportBookmarkError, Source};
use crate::presentation::rest::UserInfo;
use crate::AppState;
use axum::extract::State;
use axum::{Extension, Json};
use rest_api::import_shaarli_api::{ShaarliImportApiRequest, ShaarliImportApiResult};
use secrecy::{ExposeSecret, Secret};

pub async fn shaarli_import_api(
    State(state): State<AppState>,
    Extension(user_info): Extension<UserInfo>,
    Json(import_request): Json<ShaarliImportApiRequest>,
) -> Result<ShaarliImportApiResult, ShaarliImportApiResult> {
    if state.demo {
        return Ok(ShaarliImportApiResult::NotImplemented);
    }

    state
        .import_bookmarks
        .execute(ImportBookmarkCommand {
            user_id: user_info.id,
            source: Source::Shaarli {
                url: import_request.url,
                password: Secret::new(import_request.key.expose_secret().0.clone()),
            },
        })
        .await
        .map_err(|e| {
            log::error!("{:?}", e);
            match e {
                ImportBookmarkError::Error(_) => ShaarliImportApiResult::ServerError,
                ImportBookmarkError::ShaarliError(_) => ShaarliImportApiResult::ShaarliError,
            }
        })
        .map(|_| ShaarliImportApiResult::Success)
}
