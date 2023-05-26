
use crate::domain::repositories::AccountRepository;
use crate::infrastructure::mailer::Mailer;
use anyhow::{Context, Error};
use secrecy::{ExposeSecret};
use std::sync::Arc;
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use uuid::Uuid;

pub struct CreatePasswordRecoveryCommand {
    pub username_or_email: String,
}

#[derive(Clone)]
pub struct CreatePasswordRecoveryUseCase {
    account_repository: Arc<dyn AccountRepository>,
    mailer: Arc<Mailer>,
}

impl CreatePasswordRecoveryUseCase {
    pub fn new(account_repository: Arc<dyn AccountRepository>, mailer: Arc<Mailer>) -> Self {
        Self {
            account_repository,
            mailer,
        }
    }

    pub async fn execute(&self, command: CreatePasswordRecoveryCommand) -> anyhow::Result<()> {
        let account = self
            .account_repository
            .find_by_email(&command.username_or_email)
            .await
            .context("Could not retrieve account by email")?;

        let account = match account {
            None => self
                .account_repository
                .find_by_username(&command.username_or_email)
                .await
                .context("Could not retrieve account by username")?,
            Some(account) => Some(account),
        };

        if let Some(mut account) = account {
            let (id, token) = match account.create_password_recovery()? {
                None => return Ok(()),
                Some((id, token)) => (id, token),
            };

            let account = self
                .account_repository
                .save(account)
                .await
                .context("Could not create password recovery")?;

            let email = account
                .email()
                .map_err(Error::msg)
                .context("Could not find email address")?;

            self.mailer
                .send_password_recovery(id, token.expose_secret(), email.clone());
        } else {
            let token = Uuid::new_v4().to_string();
            let salt = SaltString::generate(&mut OsRng);
            // todo move hash-related things into a common stuff (sessions and accounts as well)
            Argon2::default()
                .hash_password(token.as_ref(), &salt)
                .map_err(Error::msg)
                .context("Could not generate password recovery token")?;
        }

        Ok(())
    }
}
