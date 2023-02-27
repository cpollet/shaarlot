use crate::{database, AppState};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use common::PasswordRules;
use rest_api::authentication::{CreateUserRequest, CreateUserResponseCode, UserResponse};
use secrecy::ExposeSecret;

pub async fn create_user(
    State(state): State<AppState>,
    Json(user): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), CreateUserResponseCode> {
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
        return Err(CreateUserResponseCode::InvalidPassword);
    }

    let hashed = argon2
        .hash_password(user.password.expose_secret().into(), &salt)
        .map_err(|_| CreateUserResponseCode::ServerError)?
        .to_string();

    // todo send email address validation email

    database::users::Mutation::create(&state.database, user.email, user.username, hashed)
        .await
        .map_err(|_| CreateUserResponseCode::ServerError)
        .map(|u| {
            (
                StatusCode::from(CreateUserResponseCode::Success),
                Json(UserResponse {
                    id: u.id,
                    username: u.username,
                }),
            )
        })
}
