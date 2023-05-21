use crate::domain::entities::password_recovery::{Expire, PasswordRecovery, Verify};
use anyhow::{Context, Error};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use chrono::{DateTime, FixedOffset, Utc};
use common::PasswordRules;
use lettre::message::Mailbox;
use secrecy::{ExposeSecret, Secret};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::mem;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug)]
pub struct Account {
    // todo remove pub
    pub id: i32,
    pub username: String,
    pub password: Password,
    pub creation_date: DateTime<Utc>,
    pub email: Option<String>,
    pub next_email: Option<NextEmail>,
    pub password_recoveries: HashMap<Uuid, PasswordRecovery>,
}

impl Account {
    pub fn validate_email(mut self) -> Result<Self, ValidateEmailError> {
        match self.next_email {
            None => Ok(self),
            Some(next_email) if next_email.token_expired() => Err(ValidateEmailError::InvalidToken),
            Some(next_email) => {
                self.email = Some(next_email.email);
                self.next_email = None;
                Ok(self)
            }
        }
    }

    pub fn mailbox(&self) -> Result<Mailbox, EmailError> {
        self.email
            .as_ref()
            .map(|e| Mailbox::from_str(e.as_str()))
            .ok_or(EmailError::NoEmail)?
            .map_err(|_| EmailError::InvalidEmail)
    }

    pub fn change_password(
        // todo should take &mut self
        self,
        passwords: (Secret<String>, Secret<String>),
    ) -> anyhow::Result<ChangePasswordResult> {
        if !PasswordRules::default()
            .validate(
                passwords.0.expose_secret().as_str(),
                passwords.1.expose_secret().as_str(),
            )
            .is_valid()
        {
            return Ok(ChangePasswordResult::InvalidPassword);
        }

        Ok(ChangePasswordResult::Success(Self {
            id: self.id,
            username: self.username,
            password: Password::Change(Self::hash_password(passwords.0)?),
            creation_date: self.creation_date,
            email: self.email,
            next_email: self.next_email,
            password_recoveries: self.password_recoveries,
        }))
    }

    fn hash_password(password: Secret<String>) -> anyhow::Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let hashed = Argon2::default()
            .hash_password(password.expose_secret().as_bytes(), &salt)
            .map_err(Error::msg)
            .context("Could not hash password")?;
        Ok(hashed.to_string())
    }

    pub fn add_password_recovery(&mut self, password_recovery: PasswordRecovery) {
        self.remove_expired_recoveries();
        self.password_recoveries
            .insert(password_recovery.id(), password_recovery);
    }

    pub fn recover_password(
        // todo should take &mut self
        mut self,
        recovery_id: Uuid,
        token: Secret<String>,
        passwords: (Secret<String>, Secret<String>),
    ) -> anyhow::Result<RecoverPasswordResult> {
        self.remove_expired_recoveries();

        // todo delete invalid ones anyways

        let recovery = match self.password_recoveries.get(&recovery_id) {
            None => return Ok(RecoverPasswordResult::InvalidRecovery),
            Some(recovery) => recovery,
        };

        if !recovery
            .token_matches(token)
            .context("Could not verify token")?
        {
            return Ok(RecoverPasswordResult::InvalidRecovery);
        }

        self.password_recoveries.remove(&recovery.id());

        match self
            .change_password(passwords)
            .context("Could not change password")?
        {
            ChangePasswordResult::Success(a) => Ok(RecoverPasswordResult::Success(a)),
            ChangePasswordResult::InvalidPassword => Ok(RecoverPasswordResult::InvalidRecovery),
        }
    }

    fn remove_expired_recoveries(&mut self) {
        self.password_recoveries.retain(|_k, v| !v.is_expired());
    }

    pub fn take_password_recoveries(&mut self) -> Vec<PasswordRecovery> {
        mem::take(&mut self.password_recoveries)
            .into_values()
            .collect::<Vec<PasswordRecovery>>()
    }
}

#[derive(Debug)]
pub enum Password {
    Keep,
    // todo should be PasswordHash
    Change(String),
}

#[derive(Debug)]
pub struct NextEmail {
    email: String,
    token: Uuid,
    token_generation_date: DateTime<Utc>,
}

impl NextEmail {
    pub fn new(email: String, token: Uuid, token_generation_date: DateTime<Utc>) -> Self {
        Self {
            email,
            token,
            token_generation_date,
        }
    }

    fn token_expired(&self) -> bool {
        let now = DateTime::<FixedOffset>::from(Utc::now());
        let duration = now.signed_duration_since(self.token_generation_date);

        duration.num_minutes() > 60
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn token(&self) -> Uuid {
        self.token
    }

    pub fn token_generation_date(&self) -> &DateTime<Utc> {
        &self.token_generation_date
    }
}

pub enum ValidateEmailError {
    InvalidToken,
}

#[derive(Debug)]
pub enum EmailError {
    NoEmail,
    InvalidEmail,
}

impl Display for EmailError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailError::NoEmail => write!(f, "No email address found"),
            EmailError::InvalidEmail => write!(f, "Invalid email address"),
        }
    }
}

#[derive(Debug)]
pub enum ChangePasswordResult {
    Success(Account),
    InvalidPassword,
}

#[derive(Debug)]
pub enum RecoverPasswordResult {
    Success(Account),
    InvalidRecovery,
    InvalidPassword,
}
