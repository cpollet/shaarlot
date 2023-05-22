use crate::domain::entities::account::ValidateEmailError;
use crate::domain::repositories::AccountRepository;
use anyhow::Context;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub struct ValidateEmailCommand {
    pub email_token: Uuid,
}

pub enum ValidationResult {
    Validated,
    InvalidToken,
}

#[derive(Clone)]
pub struct ValidateEmailUseCase {
    repository: Arc<dyn AccountRepository>,
}

impl ValidateEmailUseCase {
    pub fn new(repository: Arc<dyn AccountRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: ValidateEmailCommand) -> anyhow::Result<ValidationResult> {
        let account = self
            .repository
            .find_by_email_token(command.email_token)
            .await
            .context("Could not find account")?;

        let account = match account {
            None => return Ok(ValidationResult::InvalidToken),
            Some(account) => account,
        };

        let account = match account.validate_email() {
            Ok(account) => account,
            Err(ValidateEmailError::InvalidToken) => return Ok(ValidationResult::InvalidToken),
        };

        self.repository
            .save(account)
            .await
            .context("Could not validate email address")
            .map(|_| ValidationResult::Validated)
    }
}
