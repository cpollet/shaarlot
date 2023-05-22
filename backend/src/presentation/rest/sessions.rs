use crate::domain::entities::account::{Account, ClearPassword};
use crate::infrastructure::database::accounts::Query;
use crate::presentation::rest::{UserInfo, SESSION_KEY_USER_INFO};
use crate::AppState;
use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_sessions::extractors::{ReadableSession, WritableSession};
use rest_api::sessions::{CreateSessionRequest, CreateSessionResponse, CreateSessionResult};
use secrecy::{ExposeSecret, Secret};
use rest_api::RestPassword;

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
    let account = state
        .account_repository
        .find_by_username(&user.username)
        .await
        .map_err(|_| CreateSessionResult::ServerError)?;

    let account = match account {
        Some(account) => {
            match account.verify_password(&ClearPassword::from(user.password.expose_secret())) {
                Ok(true) => account,
                _ => return Ok(CreateSessionResult::InvalidCredentials),
            }
        }
        None => {
            // compute a dummy hash to prevent timing attacks
            let hash = PasswordHash::new(DEFAULT_HASH).unwrap();
            let _ = Argon2::default().verify_password(user.password.expose_secret().into(), &hash);
            return Ok(CreateSessionResult::InvalidCredentials);
        }
    };

    if account.email.is_none() {
        return Err(CreateSessionResult::InvalidCredentials);
    }

    session
        .insert(
            SESSION_KEY_USER_INFO,
            UserInfo {
                id: account.id,
                username: account.username,
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
