use crate::rest::error_response::ErrorResponse;
use crate::{database, AppState};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_sessions::extractors::{ReadableSession, WritableSession};
use rest_api::CreateSessionRequest;
use secrecy::ExposeSecret;

const DEFAULT_HASH: &str ="$argon2id$v=19$m=4096,t=3,p=1$baDtBn+xiGM5bIMWdtwslA$df2X6ViJYdLDvARhcgkcmo6QfQAXrbjdrOYxKWWrdF8";

const SESSION_KEY_USERID: &str = "USERID";

pub async fn get_current_session(session: ReadableSession) -> Json<Option<i32>> {
    Json(session.get(SESSION_KEY_USERID))
}

pub async fn create_session(
    mut session: WritableSession,
    State(state): State<AppState>,
    Json(user): Json<CreateSessionRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let argon2 = Argon2::default();

    let db_user = database::users::Query::find_by_username(&state.database, &user.username)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "CANNOT_READ_FROM_DATABASE",
                    &format!("Cannot read user '{}' from database", user.username),
                )),
            )
        })?
        .ok_or({
            // compute a dummy hash to prevent timing attacks
            let hash = PasswordHash::new(DEFAULT_HASH).unwrap();
            let _ = argon2.verify_password(user.password.expose_secret().into(), &hash);
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("NOT_FOUND", "User not found")),
            )
        })?;

    let password_hash = PasswordHash::new(&db_user.password).map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "User not found")),
        )
    })?;

    argon2
        .verify_password(user.password.expose_secret().into(), &password_hash)
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("NOT_FOUND", "User not found")),
            )
        })?;

    session
        .insert(SESSION_KEY_USERID, db_user.id)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "CANNOT_CREATE_SESSION",
                    "Cannot create session",
                )),
            )
        })?;

    Ok(StatusCode::CREATED)
}

pub async fn delete_current_session(mut session: WritableSession) -> StatusCode {
    session.destroy();
    StatusCode::NO_CONTENT
}
