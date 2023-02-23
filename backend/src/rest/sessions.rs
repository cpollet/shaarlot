use crate::session::{UserInfo, SESSION_KEY_USER_INFO};
use crate::{database, AppState};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_sessions::extractors::{ReadableSession, WritableSession};
use rest_api::authentication::sessions::{
    CreateSessionRequest, CreateSessionResponseCode, SessionResponse,
};
use secrecy::ExposeSecret;

const DEFAULT_HASH: &str ="$argon2id$v=19$m=4096,t=3,p=1$baDtBn+xiGM5bIMWdtwslA$df2X6ViJYdLDvARhcgkcmo6QfQAXrbjdrOYxKWWrdF8";

pub async fn get_current_session(
    session: ReadableSession,
) -> Result<Json<SessionResponse>, StatusCode> {
    session
        .get::<UserInfo>(SESSION_KEY_USER_INFO)
        .ok_or(StatusCode::NOT_FOUND)
        .map(|s| {
            Json(SessionResponse {
                username: s.username,
            })
        })
}

pub async fn create_session(
    mut session: WritableSession,
    State(state): State<AppState>,
    Json(user): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<SessionResponse>), CreateSessionResponseCode> {
    let argon2 = Argon2::default();

    let db_user = database::users::Query::find_by_username(&state.database, &user.username)
        .await
        .map_err(|_| CreateSessionResponseCode::ServerError)?
        .ok_or({
            // compute a dummy hash to prevent timing attacks
            let hash = PasswordHash::new(DEFAULT_HASH).unwrap();
            let _ = argon2.verify_password(user.password.expose_secret().into(), &hash);
            CreateSessionResponseCode::InvalidCredentials
        })?;

    let password_hash = PasswordHash::new(&db_user.password)
        .map_err(|_| CreateSessionResponseCode::InvalidCredentials)?;

    argon2
        .verify_password(user.password.expose_secret().into(), &password_hash)
        .map_err(|_| CreateSessionResponseCode::InvalidCredentials)?;

    session
        .insert(
            SESSION_KEY_USER_INFO,
            UserInfo {
                id: db_user.id,
                username: db_user.username,
            },
        )
        .map_err(|_| CreateSessionResponseCode::ServerError)?;

    Ok((
        StatusCode::from(CreateSessionResponseCode::Success),
        Json(SessionResponse {
            username: user.username,
        }),
    ))
}

pub async fn delete_current_session(mut session: WritableSession) -> StatusCode {
    session.destroy();
    StatusCode::NO_CONTENT
}
