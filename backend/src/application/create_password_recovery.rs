use crate::domain::entities::password_recovery::{ClearPasswordRecovery, PasswordRecovery};
use crate::domain::repositories::AccountRepository;
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

        // fixme: avoid expect()
        let password_recovery = ClearPasswordRecovery::new(
            account
                .as_ref()
                .map(|a| a.id.expect("must be present"))
                .unwrap_or_default(),
        )
        .context("Could not create password recovery")?;

        if let Some(mut account) = account {
            let id = password_recovery.id();
            let token = password_recovery.token.clone();

            account.add_password_recovery(PasswordRecovery::Clear(password_recovery));

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
        }

        Ok(())
    }
}
