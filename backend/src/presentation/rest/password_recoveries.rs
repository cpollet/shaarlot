use crate::application::create_password_recovery::CreatePasswordRecoveryCommand;
use crate::application::perform_password_recovery::{
    PasswordRecoveryResult, PerformPasswordRecoveryCommand,
};
use crate::AppState;
use axum::extract::State;
use axum::Json;
use rest_api::password_recoveries::create::{
    CreatePasswordRecoveryRequest, CreatePasswordRecoveryResult,
};
use rest_api::password_recoveries::update::{
    UpdatePasswordRecoveryRequest, UpdatePasswordRecoveryResult,
};
use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;

pub async fn create_password_recovery(
    State(state): State<AppState>,
    Json(request): Json<CreatePasswordRecoveryRequest>,
) -> Result<CreatePasswordRecoveryResult, CreatePasswordRecoveryResult> {
    if state.demo {
        return Ok(CreatePasswordRecoveryResult::NotImplemented);
    }

    state
        .create_password_recovery
        .execute(CreatePasswordRecoveryCommand {
            username_or_email: request.username_or_email,
        })
        .await
        .map_err(|e| {
            log::error!("{:?}", e);
            CreatePasswordRecoveryResult::ServerError
        })
        .map(|_| CreatePasswordRecoveryResult::Success)
}

pub async fn update_password_recovery(
    State(state): State<AppState>,
    Json(request): Json<UpdatePasswordRecoveryRequest>,
) -> Result<UpdatePasswordRecoveryResult, UpdatePasswordRecoveryResult> {
    if state.demo {
        return Ok(UpdatePasswordRecoveryResult::NotImplemented);
    }

    let result = state
        .perform_password_recovery
        .execute(PerformPasswordRecoveryCommand {
            id: Uuid::parse_str(request.id.as_str())
                .map_err(|_| UpdatePasswordRecoveryResult::InvalidToken)?,
            token: Secret::new(request.token.expose_secret().0.clone()),
            passwords: (
                Secret::new(request.password.expose_secret().0.clone()),
                Secret::new(request.password_verif.expose_secret().0.clone()),
            ),
        })
        .await
        .map_err(|e| {
            log::error!("{:?}", e);
            UpdatePasswordRecoveryResult::ServerError
        })?;

    match result {
        PasswordRecoveryResult::InvalidPassword => {
            Err(UpdatePasswordRecoveryResult::InvalidPassword)
        }
        PasswordRecoveryResult::InvalidRecovery => Err(UpdatePasswordRecoveryResult::InvalidToken),
        PasswordRecoveryResult::Success => Ok(UpdatePasswordRecoveryResult::Success),
    }
}
