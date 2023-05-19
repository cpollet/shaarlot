use crate::infrastructure::database::accounts::{Mutation, Query};
use crate::AppState;
use axum::extract::{Path, State};
use chrono::{DateTime, FixedOffset, Utc};
use rest_api::validate_email::ValidateEmailResult;
use uuid::Uuid;

pub async fn update_email(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> Result<ValidateEmailResult, ValidateEmailResult> {
    if state.demo {
        return Ok(ValidateEmailResult::NotImplemented);
    }

    let generation_date = Query::find_by_email_token(&state.database, uuid.to_string().as_str())
        .await
        .map_err(|_| ValidateEmailResult::ServerError)?
        .and_then(|m| match m.email_token_generation_date {
            Some(t) => Some((m.id, t)),
            None => None,
        })
        .ok_or(ValidateEmailResult::InvalidToken)?;

    let now = DateTime::<FixedOffset>::from(Utc::now());
    let duration = now.signed_duration_since(generation_date.1);

    if duration.num_minutes() > 60 {
        return Err(ValidateEmailResult::InvalidToken);
    }

    match Mutation::remove_email_token(&state.database, generation_date.0).await {
        Ok(Some(_)) => Ok(ValidateEmailResult::Success),
        _ => Err(ValidateEmailResult::ServerError),
    }
}
