use crate::domain::entities::password_recovery::{
    ClearPasswordRecovery, Expire, PasswordRecovery, Verify,
};
use anyhow::{Context, Error};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{password_hash, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, FixedOffset, Utc};
use common::PasswordRules;

use lettre::Address;
use secrecy::{ExposeSecret, Secret};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::mem;

use uuid::Uuid;

#[derive(Debug)]
pub struct Account {
    // todo remove pub
    // todo how to remove Option<i32>?
    pub id: Option<i32>,
    pub username: String,
    pub password: HashedPassword,
    pub new_password: Option<HashedPassword>,
    pub creation_date: DateTime<Utc>,
    pub email: Option<Address>,
    pub next_email: Option<NextEmail>,
    pub password_recoveries: HashMap<Uuid, PasswordRecovery>,
    /// Tells whether the current instance already passed a password verification
    pub password_verified: bool,
    pub events: HashSet<Event>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Event {
    AccountCreated, // todo handle
    EmailUpdated,
    PasswordRecoveryStarted,  // todo handle
    PasswordRecoveryFinished, // todo handle
    PasswordUpdated,
}

impl Account {
    pub fn new(
        username: String,
        email: Address,
        passwords: (ClearPassword, ClearPassword),
    ) -> anyhow::Result<Account, CreateAccountError> {
        if !Self::validate_password(&passwords) {
            return Err(CreateAccountError::InvalidPassword);
        }

        Ok(Self {
            id: None,
            username,
            email: None,
            next_email: Some(NextEmail::create(email)),
            password: Self::hash_password(passwords.0)
                .context("Could not hash password")
                .map_err(CreateAccountError::Error)?,
            new_password: None,
            creation_date: Utc::now(),
            password_recoveries: Default::default(),
            password_verified: false,
            events: HashSet::from([Event::AccountCreated]),
        })
    }

    fn validate_password(passwords: &(ClearPassword, ClearPassword)) -> bool {
        PasswordRules::default()
            .validate(
                passwords.0.expose_secret_as_str(),
                passwords.1.expose_secret_as_str(),
            )
            .is_valid()
    }

    pub fn update_email(
        &mut self,
        password: &ClearPassword,
        email: Address,
    ) -> Result<(), UpdateEmailError> {
        if !self
            .verify_password(password)
            .context("Could not verify password for email update")
            .map_err(UpdateEmailError::Error)?
        {
            return Err(UpdateEmailError::CurrentPasswordIncorrect);
        }

        if self.email().expect("email must exist") == &email {
            return Ok(());
        }

        self.next_email = Some(NextEmail::create(email));
        self.events.insert(Event::EmailUpdated);

        Ok(())
    }

    // todo &mut self
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

    pub fn email(&self) -> Result<&Address, EmailError> {
        self.email.as_ref().ok_or(EmailError::NoEmail)
    }

    pub fn next_email(&self) -> Option<&NextEmail> {
        self.next_email.as_ref()
    }

    // todo move hash-related things into a common stuff
    pub fn verify_password(&mut self, password: &ClearPassword) -> anyhow::Result<bool> {
        if self.password_verified {
            return Ok(true);
        }
        let password_hash = PasswordHash::new(self.password.expose_secret_as_str())
            .map_err(Error::msg)
            .context("Could not instantiate hash verifier")?;

        match Argon2::default().verify_password(password.expose_secret_as_bytes(), &password_hash) {
            Ok(_) => {
                self.password_verified = true;
                Ok(true)
            }
            Err(password_hash::Error::Password) => Ok(false),
            Err(e) => Err(e).map_err(Error::msg).context("Could not verify hash"),
        }
    }

    pub fn update_password(
        &mut self,
        password: &ClearPassword,
        passwords: (ClearPassword, ClearPassword),
    ) -> Result<(), UpdatePasswordError> {
        if !self
            .verify_password(password)
            .context("Could not verify password for password update")
            .map_err(UpdatePasswordError::Error)?
        {
            return Err(UpdatePasswordError::CurrentPasswordIncorrect);
        }

        self.change_password(passwords).map_err(|e| match e {
            ChangePasswordError::InvalidPassword => UpdatePasswordError::InvalidPassword,
            ChangePasswordError::Error(e) => UpdatePasswordError::Error(e),
        })
    }

    fn change_password(
        &mut self,
        passwords: (ClearPassword, ClearPassword),
    ) -> Result<(), ChangePasswordError> {
        if !Self::validate_password(&passwords) {
            return Err(ChangePasswordError::InvalidPassword);
        }

        self.new_password = Some(
            Self::hash_password(passwords.0)
                .context("Could not change password")
                .map_err(ChangePasswordError::Error)?,
        );
        self.events.insert(Event::PasswordUpdated);
        Ok(())
    }

    // todo move hash-related things into a common stuff
    fn hash_password(password: ClearPassword) -> anyhow::Result<HashedPassword> {
        let salt = SaltString::generate(&mut OsRng);
        let hashed = Argon2::default()
            .hash_password(password.expose_secret_as_bytes(), &salt)
            .map_err(Error::msg)
            .context("Could not hash password")?;
        Ok(HashedPassword(Secret::new(hashed.to_string())))
    }

    pub fn create_password_recovery(&mut self) -> anyhow::Result<Option<(Uuid, Secret<String>)>> {
        self.remove_expired_recoveries();

        let id = match self.id {
            None => return Ok(None),
            Some(id) => id,
        };

        let password_recovery =
            ClearPasswordRecovery::new(id).context("Could not create password recovery")?;

        let id = password_recovery.id;
        let token = password_recovery.token.clone();

        self.password_recoveries.insert(
            password_recovery.id(),
            PasswordRecovery::Clear(password_recovery),
        );

        Ok(Some((id, token)))
    }

    pub fn recover_password(
        &mut self,
        recovery_id: Uuid,
        token: Secret<String>,
        passwords: (ClearPassword, ClearPassword),
    ) -> Result<(), RecoverPasswordError> {
        self.remove_expired_recoveries();

        // todo delete invalid ones anyways

        let recovery = match self.password_recoveries.get(&recovery_id) {
            None => return Err(RecoverPasswordError::InvalidRecovery),
            Some(recovery) => recovery,
        };

        if !recovery
            .token_matches(token)
            .context("Could not verify token")
            .map_err(RecoverPasswordError::Error)?
        {
            return Err(RecoverPasswordError::InvalidRecovery);
        }

        self.password_recoveries.remove(&recovery.id());

        self.change_password(passwords).map_err(|e| match e {
            ChangePasswordError::InvalidPassword => RecoverPasswordError::InvalidPassword,
            ChangePasswordError::Error(e) => RecoverPasswordError::Error(e),
        })
    }

    fn remove_expired_recoveries(&mut self) {
        self.password_recoveries.retain(|_k, v| !v.is_expired());
    }

    pub fn take_password_recoveries(&mut self) -> Vec<PasswordRecovery> {
        mem::take(&mut self.password_recoveries)
            .into_values()
            .collect::<Vec<PasswordRecovery>>()
    }

    pub fn events(&self) -> HashSet<Event> {
        self.events.clone()
    }
}

#[derive(Debug)]
pub struct HashedPassword(Secret<String>);

impl HashedPassword {
    pub fn expose_secret_as_str(&self) -> &str {
        self.0.expose_secret().as_str()
    }
}

impl From<String> for HashedPassword {
    fn from(value: String) -> Self {
        Self(Secret::new(value))
    }
}

#[derive(Debug)]
pub struct ClearPassword(Secret<String>);

impl ClearPassword {
    pub fn expose_secret_as_bytes(&self) -> &[u8] {
        self.0.expose_secret().as_bytes()
    }

    pub fn expose_secret_as_str(&self) -> &str {
        self.0.expose_secret().as_str()
    }
}

impl From<String> for ClearPassword {
    fn from(value: String) -> Self {
        Self(Secret::new(value))
    }
}

#[derive(Debug)]
pub struct NextEmail {
    email: Address,
    // todo move behind a secret
    token: Uuid,
    token_generation_date: DateTime<Utc>,
}

impl NextEmail {
    pub fn new(email: Address, token: Uuid, token_generation_date: DateTime<Utc>) -> Self {
        Self {
            email,
            token,
            token_generation_date,
        }
    }

    pub fn create(email: Address) -> Self {
        Self {
            email,
            token: Uuid::new_v4(),
            token_generation_date: Utc::now(),
        }
    }

    fn token_expired(&self) -> bool {
        let now = DateTime::<FixedOffset>::from(Utc::now());
        let duration = now.signed_duration_since(self.token_generation_date);

        duration.num_minutes() > 60
    }

    pub fn email(&self) -> &Address {
        &self.email
    }

    pub fn token(&self) -> Uuid {
        self.token
    }

    pub fn token_generation_date(&self) -> &DateTime<Utc> {
        &self.token_generation_date
    }
}

#[derive(Debug)]
pub enum CreateAccountError {
    InvalidPassword,
    Error(Error),
}

#[derive(Debug)]
pub enum UpdatePasswordError {
    CurrentPasswordIncorrect,
    InvalidPassword,
    Error(Error),
}

pub enum UpdateEmailError {
    CurrentPasswordIncorrect,
    Error(Error),
}

#[derive(Debug)]
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
pub enum ChangePasswordError {
    InvalidPassword,
    Error(Error),
}

#[derive(Debug)]
pub enum RecoverPasswordError {
    InvalidPassword,
    InvalidRecovery,
    Error(Error),
}
