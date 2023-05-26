use crate::application::validate_email::{ValidateEmailCommand, ValidationResult};
use crate::AppState;
use axum::extract::{Path, State};
use rest_api::validate_email::ValidateEmailResult;
use uuid::Uuid;

pub async fn update_email(
    State(state): State<AppState>,
    Path(email_token): Path<Uuid>,
) -> Result<ValidateEmailResult, ValidateEmailResult> {
    if state.demo {
        return Err(ValidateEmailResult::NotImplemented);
    }

    let result = state
        .validate_email
        .execute(ValidateEmailCommand { email_token })
        .await
        .map_err(|_| ValidateEmailResult::ServerError)?;

    match result {
        ValidationResult::Validated => Ok(ValidateEmailResult::Success),
        ValidationResult::InvalidToken => Err(ValidateEmailResult::InvalidToken),
    }
}
