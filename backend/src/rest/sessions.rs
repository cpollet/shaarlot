use crate::sessions::session::{UserInfo, SESSION_KEY_USER_INFO};
use crate::{database, AppState};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_sessions::extractors::{ReadableSession, WritableSession};
use database::accounts::Query;
use rest_api::sessions::{CreateSessionRequest, CreateSessionResponse, CreateSessionResult};
use secrecy::ExposeSecret;

const DEFAULT_HASH: &str = "$argon2id$v=19$m=4096,t=3,p=1$baDtBn+xiGM5bIMWdtwslA$df2X6ViJYdLDvARhcgkcmo6QfQAXrbjdrOYxKWWrdF8";

pub async fn get_current_session(
    session: ReadableSession,
) -> Result<CreateSessionResult, CreateSessionResult> {
    session
        .get::<UserInfo>(SESSION_KEY_USER_INFO)
        .ok_or(CreateSessionResult::InvalidCredentials) // fixme this is weird...
        .map(|s| {
            CreateSessionResult::Success(CreateSessionResponse {
                username: s.username,
            })
        })
}

pub async fn create_session(
    mut session: WritableSession,
    State(state): State<AppState>,
    Json(user): Json<CreateSessionRequest>,
) -> Result<CreateSessionResult, CreateSessionResult> {
    let db_user = Query::find_by_username(&state.database, &user.username)
        .await
        .map_err(|_| CreateSessionResult::ServerError)?
        .ok_or({
            // compute a dummy hash to prevent timing attacks
            let hash = PasswordHash::new(DEFAULT_HASH).unwrap();
            let _ = Argon2::default().verify_password(user.password.expose_secret().into(), &hash);
            CreateSessionResult::InvalidCredentials
        })?;

    let password_hash = PasswordHash::new(&db_user.password)
        .map_err(|_| CreateSessionResult::InvalidCredentials)?;

    Argon2::default()
        .verify_password(user.password.expose_secret().into(), &password_hash)
        .map_err(|_| CreateSessionResult::InvalidCredentials)?;

    if db_user.email.is_none() {
        return Err(CreateSessionResult::InvalidCredentials);
    }

    session
        .insert(
            SESSION_KEY_USER_INFO,
            UserInfo {
                id: db_user.id,
                username: db_user.username,
            },
        )
        .map_err(|_| CreateSessionResult::ServerError)?;

    Ok(CreateSessionResult::Success(CreateSessionResponse {
        username: user.username,
    }))
}

pub async fn delete_current_session(mut session: WritableSession) -> StatusCode {
    session.destroy();
    StatusCode::NO_CONTENT
}
