use anyhow::{Context, Error};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{password_hash, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, FixedOffset, Utc};
use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;

pub trait Expire {
    fn is_expired(&self) -> bool;
}

pub trait Verify {
    fn token_matches(&self, token: Secret<String>) -> anyhow::Result<bool>;
}

// todo periodic delete when expired
#[derive(Debug)]
pub enum PasswordRecovery {
    Clear(ClearPasswordRecovery),
    Hashed(HashedPasswordRecovery),
}

impl PasswordRecovery {
    pub fn create(user_id: i32) -> anyhow::Result<ClearPasswordRecovery> {
        ClearPasswordRecovery::new(user_id).context("Could not create new password recovery")
    }

    pub fn id(&self) -> Uuid {
        match self {
            PasswordRecovery::Clear(r) => r.id(),
            PasswordRecovery::Hashed(r) => r.id(),
        }
    }
}

impl Expire for PasswordRecovery {
    fn is_expired(&self) -> bool {
        match self {
            PasswordRecovery::Clear(r) => r.is_expired(),
            PasswordRecovery::Hashed(r) => r.is_expired(),
        }
    }
}

impl Verify for PasswordRecovery {
    fn token_matches(&self, token: Secret<String>) -> anyhow::Result<bool> {
        match self {
            PasswordRecovery::Clear(r) => r.token_matches(token),
            PasswordRecovery::Hashed(r) => r.token_matches(token),
        }
    }
}

#[derive(Debug)]
pub struct ClearPasswordRecovery {
    // todo remove pub
    pub id: Uuid,
    // todo create new type
    pub token: Secret<String>,
    pub hashed_token: String,
    pub user_id: i32,
}

impl ClearPasswordRecovery {
    pub fn new(user_id: i32) -> anyhow::Result<Self> {
        let token = Uuid::new_v4().to_string();
        let salt = SaltString::generate(&mut OsRng);
        // todo move hash-related things into a common stuff
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

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn token(&self) -> &Secret<String> {
        &self.token
    }
}

impl Expire for ClearPasswordRecovery {
    fn is_expired(&self) -> bool {
        false
    }
}

impl Verify for ClearPasswordRecovery {
    fn token_matches(&self, token: Secret<String>) -> anyhow::Result<bool> {
        Ok(self.token.expose_secret().eq(token.expose_secret()))
    }
}

#[derive(Debug)]
pub struct HashedPasswordRecovery {
    // todo remove pub
    pub id: Uuid,
    pub hashed_token: String,
    pub user_id: i32,
    pub generation_date: DateTime<Utc>,
}

impl HashedPasswordRecovery {
    pub fn id(&self) -> Uuid {
        self.id
    }
}

impl Expire for HashedPasswordRecovery {
    fn is_expired(&self) -> bool {
        let now = DateTime::<FixedOffset>::from(Utc::now());
        let duration = now.signed_duration_since(self.generation_date);

        duration.num_minutes() > 5
    }
}

impl Verify for HashedPasswordRecovery {
    // todo move hash-related things into a common stuff
    fn token_matches(&self, token: Secret<String>) -> anyhow::Result<bool> {
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
