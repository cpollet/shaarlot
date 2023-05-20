use crate::domain::entities::account::{Account, NextEmail, Password};
use crate::domain::repositories::AccountRepository;
use crate::infrastructure::database::accounts;
use anyhow::Context;
use async_trait::async_trait;
use chrono::Utc;
use entity::account;
use entity::account::{Column, Model};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::ActiveValue::Set;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel};
use uuid::Uuid;

impl TryFrom<Model> for Account {
    type Error = anyhow::Error;

    fn try_from(value: Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            next_email: NextEmail::try_from_model(&value)?,
            username: value.username,
            password: Password::Keep,
            creation_date: value.creation_date.with_timezone(&Utc),
            email: value.email,
        })
    }
}

impl NextEmail {
    fn try_from_model(model: &Model) -> anyhow::Result<Option<NextEmail>> {
        let next_email = match model.new_email {
            None => None,
            Some(_) => {
                let uuid = Uuid::try_from(
                    model
                        .email_token
                        .as_ref()
                        .context("No email_token found, but new_email is present")?
                        .as_str(),
                )
                .context("email_token is not a valid UUID")?;

                Some(NextEmail::new(
                    model
                        .new_email
                        .as_ref()
                        .expect("must have a new_email")
                        .clone(),
                    uuid,
                    model
                        .email_token_generation_date
                        .context("No email_token_generation_date found, but new_email is present")?
                        .with_timezone(&Utc),
                ))
            }
        };
        Ok(next_email)
    }
}

#[derive(Clone)]
pub struct DatabaseAccountRepository {
    pub database: DatabaseConnection,
}

#[async_trait]
impl AccountRepository for DatabaseAccountRepository {
    async fn save(&self, account: Account) -> anyhow::Result<Account> {
        let mut model = accounts::Query::find_by_id(&self.database, account.id)
            .await
            .context("Could not retrieve account")?
            .context("Could not retrieve account: not found")?
            .into_active_model();

        model.email = Set(account.email);
        model.new_email = Set(account.next_email.as_ref().map(|e| e.email().to_string()));
        model.email_token = Set(account.next_email.as_ref().map(|e| e.token().to_string()));
        model.email_token_generation_date = Set(account
            .next_email
            .map(|e| DateTimeWithTimeZone::from(e.token_generation_date().clone())));

        model
            .update(&self.database)
            .await
            .context("Could not update account")
            .and_then(Account::try_from)
    }

    async fn find_by_email_token(&self, token: Uuid) -> anyhow::Result<Option<Account>> {
        account::Entity::find()
            .filter(Column::EmailToken.eq(token))
            .one(&self.database)
            .await
            .context("Could not retrieve account by email token")?
            .map(Account::try_from)
            .transpose()
    }
}
