use crate::domain::entities::account::{Account, ClearPassword, CreateAccountError};
use crate::domain::repositories::AccountRepository;
use crate::infrastructure::mailer::Mailer;
use anyhow::Context;

use lettre::Address;
use std::sync::Arc;

pub struct CreateAccountCommand {
    pub username: String,
    pub email: Address,
    pub passwords: (ClearPassword, ClearPassword),
}

#[derive(Clone)]
pub struct CreateAccountUseCase {
    repository: Arc<dyn AccountRepository>,
    mailer: Arc<Mailer>,
}

impl CreateAccountUseCase {
    pub fn new(repository: Arc<dyn AccountRepository>, mailer: Arc<Mailer>) -> Self {
        Self { repository, mailer }
    }

    pub async fn execute(
        &self,
        command: CreateAccountCommand,
    ) -> Result<Account, CreateAccountError> {
        let account = Account::new(command.username, command.email, command.passwords)?;

        let account = self
            .repository
            .save(account)
            .await
            .context("Could not save account")
            .map_err(CreateAccountError::Error)?;

        let next_email = account
            .next_email()
            .expect("new accounts must must have a next email");

        self.mailer
            .send_email_token(next_email.token(), next_email.email().clone());

        Ok(account)
    }
}
