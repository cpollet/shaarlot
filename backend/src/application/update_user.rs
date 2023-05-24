use crate::domain::entities::account::{
    ClearPassword, Event, UpdateEmailError, UpdatePasswordError,
};
use crate::domain::repositories::AccountRepository;
use crate::infrastructure::mailer::Mailer;
use anyhow::{Context, Error};
use lettre::Address;
use std::sync::Arc;

pub struct UpdateAccountCommand {
    pub user_id: i32,
    pub password: ClearPassword,
    pub new_passwords: Option<(ClearPassword, ClearPassword)>,
    pub new_email: Address,
}

pub enum UpdateAccountError {
    AccountDoesNotExist,
    CurrentPasswordIncorrect,
    InvalidPassword,
    Error(Error),
}

#[derive(Clone)]
pub struct UpdateAccountUseCase {
    account_repository: Arc<dyn AccountRepository>,
    mailer: Arc<Mailer>,
}

impl UpdateAccountUseCase {
    pub fn new(account_repository: Arc<dyn AccountRepository>, mailer: Arc<Mailer>) -> Self {
        Self {
            account_repository,
            mailer,
        }
    }

    pub async fn execute(&self, command: UpdateAccountCommand) -> Result<(), UpdateAccountError> {
        let mut account = self
            .account_repository
            .find_by_id(command.user_id)
            .await
            .context("Could not find account")
            .map_err(UpdateAccountError::Error)?
            .ok_or(UpdateAccountError::AccountDoesNotExist)?;

        if let Some(passwords) = command.new_passwords {
            account
                .update_password(&command.password, passwords)
                .map_err(|e| match e {
                    UpdatePasswordError::CurrentPasswordIncorrect => {
                        UpdateAccountError::CurrentPasswordIncorrect
                    }
                    UpdatePasswordError::InvalidPassword => UpdateAccountError::InvalidPassword,
                    UpdatePasswordError::Error(e) => {
                        UpdateAccountError::Error(e.context("Could not update password"))
                    }
                })?
        }

        account
            .update_email(&command.password, command.new_email)
            .map_err(|e| match e {
                UpdateEmailError::CurrentPasswordIncorrect => {
                    UpdateAccountError::CurrentPasswordIncorrect
                }
                UpdateEmailError::Error(e) => {
                    UpdateAccountError::Error(e.context("Could not update email"))
                }
            })?;

        let events = account.events().clone();

        log::info!("{:?}", &events);

        let account = self
            .account_repository
            .save(account)
            .await
            .map_err(|e| UpdateAccountError::Error(e.context("Could not update account")))?;

        for event in events {
            match event {
                Event::EmailUpdated => {
                    let next_email = account.next_email().expect("next email must exist");
                    self.mailer
                        .send_email_token(next_email.token(), next_email.email().clone());
                    self.mailer.send_email_updated(
                        account.email().expect("email must exist").clone(),
                        next_email.email().clone(),
                    );
                }
                Event::PasswordUpdated => {
                    log::info!("Send password updated email");
                    self.mailer
                        .send_password_updated(account.email().expect("email must exist").clone());
                }
                e => {
                    log::error!("Unhandled event: {:?}", e)
                }
            }
        }

        Ok(())
    }
}
