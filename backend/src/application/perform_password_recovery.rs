use crate::domain::entities::account::ChangePasswordResult;
use crate::domain::repositories::{AccountRepository, PasswordRecoveryRepository};
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
    password_recovery_repository: Arc<dyn PasswordRecoveryRepository>,
}

impl PerformPasswordRecoveryUseCase {
    pub fn new(
        account_repository: Arc<dyn AccountRepository>,
        password_recovery_repository: Arc<dyn PasswordRecoveryRepository>,
    ) -> Self {
        Self {
            account_repository,
            password_recovery_repository,
        }
    }

    pub async fn execute(
        &self,
        command: PerformPasswordRecoveryCommand,
    ) -> anyhow::Result<PasswordRecoveryResult> {
        let recovery = self
            .password_recovery_repository
            .find_by_id(command.id)
            .await
            .context("Could not retrieve password recovery")?;

        let recovery = match recovery {
            Some(recovery) => recovery,
            None => {
                log::debug!("recovery not found");
                return Ok(PasswordRecoveryResult::InvalidRecovery);
            }
        };

        if recovery.recovery_expired() {
            self.password_recovery_repository
                .delete(recovery.id)
                .await
                .context("Could not delete expired password recovery")?;
            log::debug!("recovery expired");
            return Ok(PasswordRecoveryResult::InvalidRecovery);
        }

        if !recovery
            .token_matches(command.token)
            .context("Could not verify token")?
        {
            log::debug!("recovery token does not match");
            return Ok(PasswordRecoveryResult::InvalidRecovery);
        }

        let account = self
            .account_repository
            .find_by_id(recovery.user_id)
            .await
            .context("Could not retrieve account")?
            .context("Could not retrieve account: not found")?;

        let result = account
            .change_password(command.passwords)
            .context("Could not change password")?;

        let account = match result {
            ChangePasswordResult::Success(account) => account,
            ChangePasswordResult::InvalidPassword => {
                return Ok(PasswordRecoveryResult::InvalidPassword)
            }
        };

        // todo atomic! (means that password recovery is under account aggregate?)
        self.account_repository
            .save(account)
            .await
            .context("Coult not save updated password")?;
        self.password_recovery_repository
            .delete(recovery.id)
            .await
            .context("Could not delete finished recovery")?;

        Ok(PasswordRecoveryResult::Success)
    }
}
