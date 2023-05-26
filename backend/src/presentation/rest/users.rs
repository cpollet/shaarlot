use crate::application::create_account::CreateAccountCommand;
use crate::domain::entities::account::{ClearPassword, CreateAccountError};

use crate::presentation::rest::UserInfo;
use crate::AppState;

use axum::extract::State;
use axum::{Extension, Json};

use lettre::Address;
use rest_api::users::create::{CreateUserRequest, CreateUserResponse, CreateUserResult};
use rest_api::users::get::{GetUserResponse, GetUserResult};
use rest_api::users::update::{UpdateUserRequest, UpdateUserResponse, UpdateUserResult};
use secrecy::{ExposeSecret, Secret};

use crate::application::update_user::{UpdateAccountCommand, UpdateAccountError};
use rest_api::RestPassword;
use std::str::FromStr;

pub async fn create_user(
    State(state): State<AppState>,
    Json(user): Json<CreateUserRequest>,
) -> Result<CreateUserResult, CreateUserResult> {
    if state.demo {
        return Ok(CreateUserResult::NotImplemented);
    }

    let result = state
        .create_account
        .execute(CreateAccountCommand {
            username: user.username,
            email: Address::from_str(&user.email)
                .map_err(|_| CreateUserResult::InvalidEmailAddress)?,
            passwords: (
                ClearPassword::from(user.password.expose_secret()),
                ClearPassword::from(user.password_verif.expose_secret()),
            ),
        })
        .await;

    match result {
        Ok(account) => Ok(CreateUserResult::Success(CreateUserResponse {
            id: account.id.expect("must have an id"),
            username: account.username,
        })),
        Err(CreateAccountError::InvalidPassword) => Ok(CreateUserResult::InvalidPassword),
        Err(CreateAccountError::Error(e)) => {
            log::error!("{:?}", e);
            Err(CreateUserResult::ServerError)
        }
    }
}

pub async fn get_current_user(
    State(state): State<AppState>,
    Extension(user_info): Extension<UserInfo>,
) -> Result<GetUserResult, GetUserResult> {
    state
        .account_repository
        .find_by_id(user_info.id)
        .await
        .map_err(|e| {
            log::error!("{:?}", e);
            GetUserResult::ServerError
        })?
        .map(|u| {
            Ok(GetUserResult::Success(GetUserResponse {
                id: u.id.expect("must have an id"),
                username: u.username,
                email: u.email.ok_or(GetUserResult::Forbidden)?.to_string(),
            }))
        })
        .unwrap_or_else(|| Err(GetUserResult::Forbidden))
}

pub async fn update_current_user(
    State(state): State<AppState>,
    Extension(user_info): Extension<UserInfo>,
    Json(user): Json<UpdateUserRequest>,
) -> Result<UpdateUserResult, UpdateUserResult> {
    if state.demo {
        return Ok(UpdateUserResult::NotImplemented);
    }

    state
        .update_account
        .execute(UpdateAccountCommand {
            user_id: user_info.id,
            password: ClearPassword::from(user.current_password.expose_secret()),
            new_passwords: user.new_password.map(|p1| {
                (
                    ClearPassword::from(p1.expose_secret()),
                    ClearPassword::from(
                        user.new_password_verif
                            .unwrap_or(Secret::from(RestPassword("".to_string())))
                            .expose_secret(),
                    ),
                )
            }),
            new_email: Address::from_str(&user.email)
                .map_err(|_| UpdateUserResult::InvalidEmailAddress)?,
        })
        .await
        .map(|_| {
            UpdateUserResult::Success(UpdateUserResponse {
                id: 0,
                username: "".to_string(),
                email: "".to_string(),
            })
        })
        .map_err(|e| match e {
            UpdateAccountError::AccountDoesNotExist => UpdateUserResult::Forbidden,
            UpdateAccountError::CurrentPasswordIncorrect => {
                UpdateUserResult::InvalidCurrentPassword
            }
            UpdateAccountError::InvalidPassword => UpdateUserResult::InvalidNewPassword,
            UpdateAccountError::Error(e) => {
                log::error!("{:?}", e);
                UpdateUserResult::ServerError
            }
        })
}
