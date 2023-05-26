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

    // if user.new_password.is_some()
    //     && !PasswordRules::default()
    //         .validate(
    //             user.new_password.as_ref().unwrap().expose_secret(),
    //             user.new_password_verif.as_ref().unwrap().expose_secret(),
    //         )
    //         .is_valid()
    // {
    //     return Err(UpdateUserResult::InvalidNewPassword);
    // }
    //
    // let user_new_mailbox = user
    //     .email
    //     .parse::<Mailbox>()
    //     .map_err(|_| UpdateUserResult::InvalidEmailAddress)?;
    //
    // let user_info = session
    //     .get::<UserInfo>(SESSION_KEY_USER_INFO)
    //     .ok_or(UpdateUserResult::Forbidden)?;
    //
    // let db_user = Query::find_by_username(&state.database, &user_info.username)
    //     .await
    //     .map_err(|_| UpdateUserResult::ServerError)?
    //     .ok_or(UpdateUserResult::Forbidden)?;
    //
    // let user_old_mailbox = db_user
    //     .email
    //     .clone()
    //     .map(|e| {
    //         e.parse::<Mailbox>()
    //             .expect("Invalid email found in database")
    //     })
    //     .ok_or(UpdateUserResult::Forbidden)?;
    //
    // let password_hash =
    //     PasswordHash::new(&db_user.password).map_err(|_| UpdateUserResult::ServerError)?;
    //
    // let argon2 = Argon2::default();
    // argon2
    //     .verify_password(user.current_password.expose_secret().into(), &password_hash)
    //     .map_err(|_| UpdateUserResult::InvalidCurrentPassword)?;
    //
    // // todo refactor these (bool, Option<?>) things
    //
    // let new_password = user
    //     .new_password
    //     .map(|p| {
    //         let salt = SaltString::generate(&mut OsRng);
    //         Ok((
    //             true,
    //             Some(
    //                 argon2
    //                     .hash_password(p.expose_secret().into(), &salt)
    //                     .map_err(|_| UpdateUserResult::ServerError)?
    //                     .to_string(),
    //             ),
    //         ))
    //     })
    //     .transpose()?
    //     .unwrap_or_default();
    //
    // let new_email = if user_new_mailbox != user_old_mailbox {
    //     (true, Some((user.email, Uuid::new_v4())))
    // } else {
    //     (false, None)
    // };
    //
    // let db_user = Mutation::update(&state.database, db_user, new_password.1, new_email.1)
    //     .await
    //     .map_err(|_| UpdateUserResult::ServerError)?;
    //
    // // if new_password.0 {
    // //     state.mailer.send_password_updated(user_old_mailbox.clone());
    // // }
    // //
    // // if new_email.0 {
    // //     state
    // //         .mailer
    // //         .send_email_token(db_user.email_token.unwrap(), user_new_mailbox);
    // //     state.mailer.send_email_updated(
    // //         &db_user.new_email.expect("No new email found in database"),
    // //         user_old_mailbox,
    // //     );
    // // }
    //
    // Ok(UpdateUserResult::Success(UpdateUserResponse {
    //     id: db_user.id,
    //     username: db_user.username,
    //     email: db_user
    //         .email
    //         .expect("Authenticated user must have an email"),
    // }))
}
