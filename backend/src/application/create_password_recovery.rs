use crate::domain::entities::password_recovery::PasswordRecovery;
use crate::domain::repositories::{AccountRepository, PasswordRecoveryRepository};
use crate::infrastructure::mailer::Mailer;
use anyhow::{Context, Error};
use secrecy::ExposeSecret;
use std::sync::Arc;

pub struct CreatePasswordRecoveryCommand {
    pub username_or_email: String,
}

#[derive(Clone)]
pub struct CreatePasswordRecoveryUseCase {
    account_repository: Arc<dyn AccountRepository>,
    password_recovery_repository: Arc<dyn PasswordRecoveryRepository>,
    mailer: Arc<Mailer>,
}

impl CreatePasswordRecoveryUseCase {
    pub fn new(
        account_repository: Arc<dyn AccountRepository>,
        password_recovery_repository: Arc<dyn PasswordRecoveryRepository>,
        mailer: Arc<Mailer>,
    ) -> Self {
        Self {
            account_repository,
            password_recovery_repository,
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

        let password_recovery =
            PasswordRecovery::new(account.as_ref().map(|a| a.id).unwrap_or_default())
                .context("Could not create password recovery")?;

        let mailbox = account
            .map(|a| a.mailbox())
            .transpose()
            .map_err(Error::msg)
            .context("Could not find email address")?;

        if let Some(mailbox) = mailbox {
            let token = password_recovery.token.clone();

            let password_recovery = self
                .password_recovery_repository
                .save(password_recovery)
                .await
                .context("Could not save password recovers")?;

            self.mailer.send_password_recovery(
                password_recovery.id,
                token.expose_secret(),
                mailbox,
            );
        }

        Ok(())
    }
}
