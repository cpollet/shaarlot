use anyhow::{Context, Error};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{password_hash, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, FixedOffset, Utc};
use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;

pub struct PasswordRecovery {
    pub id: Uuid,
    pub token: Secret<String>,
    pub hashed_token: String,
    pub user_id: i32,
}

pub struct ObfuscatedPasswordRecovery {
    pub id: Uuid,
    pub hashed_token: String,
    pub user_id: i32,
    pub generation_date: DateTime<Utc>,
}

// todo periodic delete when expired

impl PasswordRecovery {
    pub fn new(user_id: i32) -> anyhow::Result<Self> {
        let token = Uuid::new_v4().to_string();
        let salt = SaltString::generate(&mut OsRng);
        let hashed_token = Argon2::default()
            .hash_password(token.as_ref(), &salt)
            .map_err(Error::msg)
            .context("Could not generate password recovery token")?
            .to_string();

        Ok(Self {
            id: Uuid::new_v4(),
            token: Secret::new(token),
            hashed_token,
            user_id,
        })
    }
}

impl ObfuscatedPasswordRecovery {
    pub fn recovery_expired(&self) -> bool {
        let now = DateTime::<FixedOffset>::from(Utc::now());
        let duration = now.signed_duration_since(self.generation_date);

        duration.num_minutes() > 5
    }

    pub fn token_matches(&self, token: Secret<String>) -> anyhow::Result<bool> {
        let token_hash = PasswordHash::new(&self.hashed_token)
            .map_err(Error::msg)
            .context("Could not instantiate hash verifier")?;

        match Argon2::default().verify_password(token.expose_secret().as_ref(), &token_hash) {
            Ok(_) => Ok(true),
            Err(password_hash::Error::Password) => Ok(false),
            Err(e) => Err(e).map_err(Error::msg).context("Could not verify hash"),
        }
    }
}
