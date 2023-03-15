use crate::database::accounts::{Mutation, Query};
use crate::sessions::session::{UserInfo, SESSION_KEY_USER_INFO};
use crate::AppState;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::extract::State;
use axum::Json;
use axum_sessions::extractors::ReadableSession;
use common::PasswordRules;
use lettre::message::Mailbox;
use rest_api::users::create::{CreateUserRequest, CreateUserResponse, CreateUserResult};
use rest_api::users::get::{GetUserResponse, GetUserResult};
use rest_api::users::update::{UpdateUserRequest, UpdateUserResponse, UpdateUserResult};
use secrecy::ExposeSecret;
use uuid::Uuid;

pub async fn create_user(
    State(state): State<AppState>,
    Json(user): Json<CreateUserRequest>,
) -> Result<CreateUserResult, CreateUserResult> {
    if !PasswordRules::default()
        .validate(
            user.password.expose_secret(),
            user.password_verif.expose_secret(),
        )
        .is_valid()
    {
        return Err(CreateUserResult::InvalidPassword);
    }

    let user_mailbox = user
        .email
        .parse::<Mailbox>()
        .map_err(|_| CreateUserResult::InvalidEmailAddress)?;

    let salt = SaltString::generate(&mut OsRng);
    let hashed = Argon2::default()
        .hash_password(user.password.expose_secret().into(), &salt)
        .map_err(|_| CreateUserResult::ServerError)?
        .to_string();

    let email_token = Uuid::new_v4().to_string();

    let result = Mutation::create(
        &state.database,
        user.email,
        user.username,
        hashed,
        email_token.clone(),
    )
    .await
    .map_err(|_| CreateUserResult::ServerError)
    .map(|u| {
        CreateUserResult::Success(CreateUserResponse {
            id: u.id,
            username: u.username,
        })
    })?;

    state.mailer.send_email_token(email_token, user_mailbox);

    Ok(result)
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

    if new_password.0 {
        state.mailer.send_password_updated(user_old_mailbox.clone());
    }

    if new_email.0 {
        state
            .mailer
            .send_email_token(db_user.email_token.unwrap(), user_new_mailbox);
        state.mailer.send_email_updated(
            &db_user.new_email.expect("No new email found in database"),
            user_old_mailbox,
        );
    }

    Ok(UpdateUserResult::Success(UpdateUserResponse {
        id: db_user.id,
        username: db_user.username,
        email: db_user
            .email
            .expect("Authenticated user must have an email"),
    }))
}
