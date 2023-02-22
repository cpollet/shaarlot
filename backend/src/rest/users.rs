use crate::rest::error_response::ErrorResponse;
use crate::{database, AppState};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use rest_api::{CreateUserRequest, UserResponse};
use secrecy::ExposeSecret;

pub async fn create_user(
    State(state): State<AppState>,
    Json(user): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, Json<ErrorResponse>)> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hashed = argon2
        .hash_password(user.password.expose_secret().into(), &salt)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("INTERNAL_ERROR", "Internal error")),
            )
        })?
        .to_string();

    database::users::Mutation::create(&state.database, user.username, hashed)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "CANNOT_WRITE_TO_DATABASE",
                    "Cannot save user to database",
                )),
            )
        })
        .map(|u| {
            (
                StatusCode::CREATED,
                Json(UserResponse {
                    id: u.id,
                    username: u.username,
                }),
            )
        })
}
