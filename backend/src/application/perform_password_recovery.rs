use crate::domain::entities::account::RecoverPasswordResult;
use crate::domain::repositories::AccountRepository;
use anyhow::Context;
use secrecy::Secret;
use std::sync::Arc;
use uuid::Uuid;

pub struct PerformPasswordRecoveryCommand {
    pub id: Uuid,
    pub token: Secret<String>,
    pub passwords: (Secret<String>, Secret<String>),
}

pub enum PasswordRecoveryResult {
    Success,
    InvalidPassword,
    InvalidRecovery,
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
    ) -> anyhow::Result<PasswordRecoveryResult> {
        let account = self
            .account_repository
            .find_by_recovery_id(command.id)
            .await
            .context("Could not find account by recovery")?;

        let account = match account {
            Some(account) => account,
            None => {
                log::debug!("account not found for recovery");
                return Ok(PasswordRecoveryResult::InvalidRecovery);
            }
        };

        let result = account
            .recover_password(command.id, command.token, command.passwords)
            .context("Could not update password")?;

        let account = match result {
            RecoverPasswordResult::Success(account) => account,
            RecoverPasswordResult::InvalidRecovery => {
                return Ok(PasswordRecoveryResult::InvalidRecovery)
            }
            RecoverPasswordResult::InvalidPassword => {
                return Ok(PasswordRecoveryResult::InvalidPassword)
            }
        };

        self.account_repository
            .save(account)
            .await
            .context("Could not save updated password")
            .map(|_| PasswordRecoveryResult::Success)
    }
}
