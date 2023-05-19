use crate::infrastructure::database::accounts::Mutation as UserMutation;
use crate::infrastructure::database::accounts::Query as UserQuery;
use crate::infrastructure::database::password_recoveries::{Mutation, Query};
use crate::AppState;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::extract::State;
use axum::Json;
use chrono::{DateTime, FixedOffset, Utc};
use common::PasswordRules;
use entity::password_recovery::Model;
use rest_api::password_recoveries::create::{
    CreatePasswordRecoveryRequest, CreatePasswordRecoveryResult,
};
use rest_api::password_recoveries::update::{
    UpdatePasswordRecoveryRequest, UpdatePasswordRecoveryResult,
};
use rest_api::{RestPassword, RestToken};
use sea_orm::{DatabaseConnection, DbErr, TransactionTrait};
use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;

// todo delete when expired

pub async fn create_password_recovery(
    State(state): State<AppState>,
    Json(request): Json<CreatePasswordRecoveryRequest>,
) -> Result<CreatePasswordRecoveryResult, CreatePasswordRecoveryResult> {
    if state.demo {
        return Ok(CreatePasswordRecoveryResult::NotImplemented);
    }

    let user = UserQuery::find_by_username(&state.database, &request.username_or_email)
        .await
        .map_err(|e| {
            log::error!("Error fetching by username: {}", e);
            CreatePasswordRecoveryResult::ServerError
        })?;

    let user = match user {
        None => UserQuery::find_by_email(&state.database, &request.username_or_email)
            .await
            .map_err(|e| {
                log::error!("Error fetching by email: {}", e);
                CreatePasswordRecoveryResult::ServerError
            })?,
        s => s,
    };

    let argon2 = Argon2::default();
    let recovery_id = Uuid::new_v4();
    let recovery_token = {
        let token = Uuid::new_v4();
        let salt = SaltString::generate(&mut OsRng);
        let hashed_token = argon2
            .hash_password(token.as_ref(), &salt)
            .map_err(|e| {
                log::error!("Error generating the token: {}", e);
                CreatePasswordRecoveryResult::ServerError
            })?
            .to_string();
        (token, hashed_token)
    };

    if let Some(user) = user {
        let to = user
            .email
            .map(|e| e.parse())
            .ok_or_else(|| {
                log::warn!("No email found for {}", user.username);
                CreatePasswordRecoveryResult::ServerError
            })?
            .map_err(|e| {
                log::error!("Could not parse email of {}: {}", user.username, e);
                CreatePasswordRecoveryResult::ServerError
            })?;

        Mutation::create(
            &state.database,
            &recovery_id.to_string(),
            user.id,
            recovery_token.1,
        )
        .await
        .map_err(|e| {
            log::error!("Could not save recovery in database: {}", e);
            CreatePasswordRecoveryResult::ServerError
        })?;

        state
            .mailer
            .send_password_recovery(recovery_id, recovery_token.0, to);
    }

    Ok(CreatePasswordRecoveryResult::Success)
}

pub async fn update_password_recovery(
    State(state): State<AppState>,
    Json(request): Json<UpdatePasswordRecoveryRequest>,
) -> Result<UpdatePasswordRecoveryResult, UpdatePasswordRecoveryResult> {
    if state.demo {
        return Ok(UpdatePasswordRecoveryResult::NotImplemented);
    }

    if !PasswordRules::default()
        .validate(
            request.password.expose_secret(),
            request.password_verif.expose_secret(),
        )
        .is_valid()
    {
        return Err(UpdatePasswordRecoveryResult::InvalidPassword);
    }

    let password = hash_password(&request.password)?;
    let recovery = find_recovery(&state.database, &request.id, &request.token).await?;

    let user = UserQuery::find_by_id(&state.database, recovery.user_id)
        .await
        .map_err(|e| {
            log::error!("Could not fetch user {}: {}", recovery.user_id, e);
            UpdatePasswordRecoveryResult::ServerError
        })?
        .ok_or(UpdatePasswordRecoveryResult::InvalidToken)?;

    let user_mailbox = user
        .email
        .clone()
        .map(|e| e.parse().expect("Invalid email found in database"))
        .ok_or(UpdatePasswordRecoveryResult::ServerError)?;

    state
        .database
        .transaction::<_, (), DbErr>(|tx| {
            Box::pin(async move {
                UserMutation::update(tx, user, Some(password), None).await?;
                Mutation::delete(tx, &recovery.id).await?;
                Ok(())
            })
        })
        .await
        .map_err(|_| UpdatePasswordRecoveryResult::ServerError)?;

    state.mailer.send_password_updated(user_mailbox);

    Ok(UpdatePasswordRecoveryResult::Success)
}

fn hash_password(password: &Secret<RestPassword>) -> Result<String, UpdatePasswordRecoveryResult> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hashed = argon2
        .hash_password(password.expose_secret().into(), &salt)
        .map_err(|e| {
            log::error!("Could not hash new password: {}", e);
            UpdatePasswordRecoveryResult::ServerError
        })?;
    Ok(hashed.to_string())
}

async fn find_recovery(
    db: &DatabaseConnection,
    id: &str,
    token: &Secret<RestToken>,
) -> Result<Model, UpdatePasswordRecoveryResult> {
    let recovery = Query::find_by_id(db, id)
        .await
        .map_err(|e| {
            log::error!("Unable to fetch recovery {}: {}", id, e);
            UpdatePasswordRecoveryResult::InvalidToken
        })?
        .ok_or({
            log::info!("Recovery {} not found", id);
            UpdatePasswordRecoveryResult::InvalidToken
        })?;

    let now = DateTime::<FixedOffset>::from(Utc::now());
    let duration = now.signed_duration_since(recovery.generation_date);

    if duration.num_minutes() > 5 {
        let _ = Mutation::delete(db, id).await;
        log::info!("Recovery {} expired", id);
        return Err(UpdatePasswordRecoveryResult::InvalidToken);
    }

    let token_hash = PasswordHash::new(&recovery.token).map_err(|e| {
        log::error!("Error while instantiating hash verifier: {}", e);
        UpdatePasswordRecoveryResult::InvalidToken
    })?;

    let token = Uuid::parse_str(&token.expose_secret().0).map_err(|_| {
        log::info!("The provided token is not a valid UUID");
        UpdatePasswordRecoveryResult::InvalidToken
    })?;

    Argon2::default()
        .verify_password(token.as_ref(), &token_hash)
        .map_err(|e| {
            log::info!("Error while checking hash: {}", e);
            UpdatePasswordRecoveryResult::InvalidToken
        })?;

    Ok(recovery)
}
