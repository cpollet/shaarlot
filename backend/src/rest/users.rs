use crate::{database, AppState};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use axum::extract::State;
use axum::Json;
use common::PasswordRules;
use lettre::message::Mailbox;
use rest_api::users::{CreateUserRequest, CreateUserResponse, CreateUserResult};
use secrecy::ExposeSecret;
use uuid::Uuid;

pub async fn create_user(
    State(state): State<AppState>,
    Json(user): Json<CreateUserRequest>,
) -> Result<CreateUserResult, CreateUserResult> {
    // todo better validation (username, email)
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

    let argon2 = Argon2::default();

    let salt = SaltString::generate(&mut OsRng);
    let hashed = argon2
        .hash_password(user.password.expose_secret().into(), &salt)
        .map_err(|_| CreateUserResult::ServerError)?
        .to_string();

    let email_token = Uuid::new_v4().to_string();

    let result = database::users::Mutation::create(
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
