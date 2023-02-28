use crate::{database, AppState};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use axum::extract::State;
use axum::Json;
use common::PasswordRules;
use rest_api::users::{CreateUserRequest, CreateUserResponse, CreateUserResult};
use secrecy::ExposeSecret;

pub async fn create_user(
    State(state): State<AppState>,
    Json(user): Json<CreateUserRequest>,
) -> Result<CreateUserResult, CreateUserResult> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

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

    let hashed = argon2
        .hash_password(user.password.expose_secret().into(), &salt)
        .map_err(|_| CreateUserResult::ServerError)?
        .to_string();

    // todo send email address validation email

    database::users::Mutation::create(&state.database, user.email, user.username, hashed)
        .await
        .map_err(|_| CreateUserResult::ServerError)
        .map(|u| {
            CreateUserResult::Success(CreateUserResponse {
                id: u.id,
                username: u.username,
            })
        })
}
