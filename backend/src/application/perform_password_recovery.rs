use crate::domain::entities::account::{ClearPassword, RecoverPasswordError};
use crate::domain::repositories::AccountRepository;
use anyhow::{Context, Error};
use secrecy::Secret;
use std::sync::Arc;
use uuid::Uuid;

pub struct PerformPasswordRecoveryCommand {
    pub id: Uuid,
    pub token: Secret<String>,
    pub passwords: (ClearPassword, ClearPassword),
}

pub enum PasswordRecoveryError {
    InvalidPassword,
    InvalidRecovery,
    Error(Error),
}

#[derive(Clone)]
pub struct PerformPasswordRecoveryUseCase {
    account_repository: Arc<dyn AccountRepository>,
}

impl PerformPasswordRecoveryUseCase {
    pub fn new(account_repository: Arc<dyn AccountRepository>) -> Self {
        Self { account_repository }
    }

    pub async fn execute(
        &self,
        command: PerformPasswordRecoveryCommand,
    ) -> Result<(), PasswordRecoveryError> {
        let account = self
            .account_repository
            .find_by_recovery_id(command.id)
            .await
            .context("Could not find account by recovery")
            .map_err(PasswordRecoveryError::Error)?;

        let mut account = match account {
            Some(account) => account,
            None => {
                return Err(PasswordRecoveryError::InvalidRecovery);
            }
        };

        account
            .recover_password(command.id, command.token, command.passwords)
            .map_err(|e| match e {
                RecoverPasswordError::InvalidPassword => PasswordRecoveryError::InvalidPassword,
                RecoverPasswordError::InvalidRecovery => PasswordRecoveryError::InvalidRecovery,
                RecoverPasswordError::Error(e) => PasswordRecoveryError::Error(e),
            })?;

        self.account_repository
            .save(account)
            .await
            .context("Could not save updated password")
            .map_err(PasswordRecoveryError::Error)
            .map(|_| ())
    }
}
