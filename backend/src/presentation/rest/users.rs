use crate::application::create_account::CreateAccountCommand;
use crate::domain::entities::account::{ClearPassword, CreateAccountError};
use crate::infrastructure::database::accounts::{Mutation, Query};
use crate::presentation::rest::{UserInfo, SESSION_KEY_USER_INFO};
use crate::AppState;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::extract::State;
use axum::Json;
use axum_sessions::extractors::ReadableSession;
use common::PasswordRules;
use lettre::message::Mailbox;
use lettre::Address;
use rest_api::users::create::{CreateUserRequest, CreateUserResponse, CreateUserResult};
use rest_api::users::get::{GetUserResponse, GetUserResult};
use rest_api::users::update::{UpdateUserRequest, UpdateUserResponse, UpdateUserResult};
use secrecy::{ExposeSecret};

use std::str::FromStr;
use uuid::Uuid;

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
    session: ReadableSession,
) -> Result<GetUserResult, GetUserResult> {
    let user_info = session
        .get::<UserInfo>(SESSION_KEY_USER_INFO)
        .ok_or(GetUserResult::Forbidden)?;

    Query::find_by_username(&state.database, &user_info.username)
        .await
        .map_err(|_| GetUserResult::Forbidden)? // this is ok, Forbidden would be when we query for another user
        .map(|u| {
            Ok(GetUserResult::Success(GetUserResponse {
                id: u.id,
                username: u.username,
                email: u.email.ok_or(GetUserResult::Forbidden)?,
            }))
        })
        .unwrap_or_else(|| Err(GetUserResult::Forbidden))
}

pub async fn update_current_user(
    State(state): State<AppState>,
    session: ReadableSession,
    Json(user): Json<UpdateUserRequest>,
) -> Result<UpdateUserResult, UpdateUserResult> {
    if state.demo {
        return Ok(UpdateUserResult::NotImplemented);
    }

    if user.new_password.is_some()
        && !PasswordRules::default()
            .validate(
                user.new_password.as_ref().unwrap().expose_secret(),
                user.new_password_verif.as_ref().unwrap().expose_secret(),
            )
            .is_valid()
    {
        return Err(UpdateUserResult::InvalidNewPassword);
    }

    let user_new_mailbox = user
        .email
        .parse::<Mailbox>()
        .map_err(|_| UpdateUserResult::InvalidEmailAddress)?;

    let user_info = session
        .get::<UserInfo>(SESSION_KEY_USER_INFO)
        .ok_or(UpdateUserResult::Forbidden)?;

    let db_user = Query::find_by_username(&state.database, &user_info.username)
        .await
        .map_err(|_| UpdateUserResult::ServerError)?
        .ok_or(UpdateUserResult::Forbidden)?;

    let user_old_mailbox = db_user
        .email
        .clone()
        .map(|e| {
            e.parse::<Mailbox>()
                .expect("Invalid email found in database")
        })
        .ok_or(UpdateUserResult::Forbidden)?;

    let password_hash =
        PasswordHash::new(&db_user.password).map_err(|_| UpdateUserResult::ServerError)?;

    let argon2 = Argon2::default();
    argon2
        .verify_password(user.current_password.expose_secret().into(), &password_hash)
        .map_err(|_| UpdateUserResult::InvalidCurrentPassword)?;

    // todo refactor these (bool, Option<?>) things

    let new_password = user
        .new_password
        .map(|p| {
            let salt = SaltString::generate(&mut OsRng);
            Ok((
                true,
                Some(
                    argon2
                        .hash_password(p.expose_secret().into(), &salt)
                        .map_err(|_| UpdateUserResult::ServerError)?
                        .to_string(),
                ),
            ))
        })
        .transpose()?
        .unwrap_or_default();

    let new_email = if user_new_mailbox != user_old_mailbox {
        (true, Some((user.email, Uuid::new_v4())))
    } else {
        (false, None)
    };

    let db_user = Mutation::update(&state.database, db_user, new_password.1, new_email.1)
        .await
        .map_err(|_| UpdateUserResult::ServerError)?;

    // if new_password.0 {
    //     state.mailer.send_password_updated(user_old_mailbox.clone());
    // }
    //
    // if new_email.0 {
    //     state
    //         .mailer
    //         .send_email_token(db_user.email_token.unwrap(), user_new_mailbox);
    //     state.mailer.send_email_updated(
    //         &db_user.new_email.expect("No new email found in database"),
    //         user_old_mailbox,
    //     );
    // }

    Ok(UpdateUserResult::Success(UpdateUserResponse {
        id: db_user.id,
        username: db_user.username,
        email: db_user
            .email
            .expect("Authenticated user must have an email"),
    }))
}
