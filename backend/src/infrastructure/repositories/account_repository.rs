use crate::domain::entities::account::{Account, NextEmail, Password};
use crate::domain::entities::password_recovery::{
    ClearPasswordRecovery, HashedPasswordRecovery, PasswordRecovery,
};
use crate::domain::repositories::AccountRepository;
use anyhow::Context;
use async_trait::async_trait;
use chrono::Utc;
use entity::account::{
    ActiveModel as AccountActiveModel, Column as AccountColumn, Entity as AccountEntity,
    Model as AccountModel,
};
use entity::password_recovery::{
    ActiveModel as PasswordRecoveryActiveModel, Column as PasswordRecoveryColumn,
    Entity as PasswordRecoveryEntity, Model as PasswordRecoveryModel,
};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sea_query::Query;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use sea_orm::{ColumnTrait, Condition, NotSet};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

impl Account {
    fn into_active_model(self) -> AccountActiveModel {
        AccountActiveModel {
            id: Unchanged(self.id),
            username: Unchanged(self.username),
            password: NotSet,
            creation_date: Unchanged(DateTimeWithTimeZone::from(self.creation_date)),
            email: Set(self.email),
            email_token: Set(self.next_email.as_ref().map(|e| e.token().to_string())),
            email_token_generation_date: Set(self
                .next_email
                .as_ref()
                .map(|e| DateTimeWithTimeZone::from(*e.token_generation_date()))),
            new_email: Set(self.next_email.map(|e| e.email().to_string())),
        }
    }
}

impl TryFrom<(AccountModel, Vec<PasswordRecoveryModel>)> for Account {
    type Error = anyhow::Error;

    fn try_from(value: (AccountModel, Vec<PasswordRecoveryModel>)) -> Result<Self, Self::Error> {
        let account = value.0;

        let mut password_recoveries = HashMap::with_capacity(value.1.len());
        for password_recovery in value.1 {
            let password_recovery = PasswordRecovery::try_from(password_recovery)
                .context("Invalid password recovery")?;
            password_recoveries.insert(password_recovery.id(), password_recovery);
        }

        Ok(Self {
            id: account.id,
            next_email: NextEmail::try_from_model(&account)?,
            username: account.username,
            password: Password::Keep,
            creation_date: account.creation_date.with_timezone(&Utc),
            email: account.email,
            password_recoveries,
        })
    }
}

impl TryFrom<PasswordRecoveryModel> for PasswordRecovery {
    type Error = anyhow::Error;

    fn try_from(value: PasswordRecoveryModel) -> Result<Self, Self::Error> {
        Ok(PasswordRecovery::Hashed(HashedPasswordRecovery {
            id: Uuid::from_str(value.id.as_str()).context("Invalid UUID")?,
            hashed_token: value.token,
            user_id: value.user_id,
            generation_date: value.generation_date.with_timezone(&Utc),
        }))
    }
}

impl ClearPasswordRecovery {
    fn into_active_model(self) -> PasswordRecoveryActiveModel {
        PasswordRecoveryActiveModel {
            id: Set(self.id().to_string()),
            token: Set(self.hashed_token),
            user_id: Set(self.user_id),
            ..Default::default()
        }
    }
}

impl NextEmail {
    fn try_from_model(model: &AccountModel) -> anyhow::Result<Option<NextEmail>> {
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
        // fixme transaction
        PasswordRecoveryEntity::delete_many()
            .filter(
                Condition::all()
                    .add(PasswordRecoveryColumn::UserId.eq(account.id))
                    .add(
                        PasswordRecoveryColumn::Id.is_not_in(
                            account
                                .password_recoveries
                                .keys()
                                .map(|id| id.to_string())
                                .collect::<Vec<String>>(),
                        ),
                    ),
            )
            .exec(&self.database)
            .await
            .context("Could not delete expired password recoveries")?;

        let mut account = account;

        let password_recoveries = account
            .take_password_recoveries()
            .into_iter()
            .filter_map(|v| match v {
                PasswordRecovery::Clear(r) => Some(r.into_active_model()),
                _ => None,
            })
            .collect::<Vec<PasswordRecoveryActiveModel>>();

        for password_recovery in password_recoveries {
            password_recovery
                .insert(&self.database)
                .await
                .context("Could not save new password recovers")?;
        }

        let account = account.into_active_model();
        let account = account
            .update(&self.database)
            .await
            .context("Could not update account")?;

        self.find_by_id(account.id)
            .await
            .transpose()
            .expect("account must exist")
    }

    async fn find_by_id(&self, id: i32) -> anyhow::Result<Option<Account>> {
        AccountEntity::find()
            .find_with_related(PasswordRecoveryEntity)
            .filter(AccountColumn::Id.eq(id))
            .all(&self.database)
            .await
            .context("Could not find account by id")?
            .pop()
            .map(Account::try_from)
            .transpose()
    }

    async fn find_by_email_token(&self, token: Uuid) -> anyhow::Result<Option<Account>> {
        AccountEntity::find()
            .find_with_related(PasswordRecoveryEntity)
            .filter(AccountColumn::EmailToken.eq(token.to_string()))
            .all(&self.database)
            .await
            .context("Could not find account by email token")?
            .pop()
            .map(Account::try_from)
            .transpose()
    }

    async fn find_by_username(&self, username: &str) -> anyhow::Result<Option<Account>> {
        AccountEntity::find()
            .find_with_related(PasswordRecoveryEntity)
            .filter(AccountColumn::Username.eq(username.to_lowercase()))
            .all(&self.database)
            .await
            .context("Could not find account by username")?
            .pop()
            .map(Account::try_from)
            .transpose()
    }

    async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<Account>> {
        AccountEntity::find()
            .find_with_related(PasswordRecoveryEntity)
            .filter(AccountColumn::Email.eq(email.to_lowercase()))
            .all(&self.database)
            .await
            .context("Could not find account by email")?
            .pop()
            .map(Account::try_from)
            .transpose()
    }

    async fn find_by_recovery_id(&self, id: Uuid) -> anyhow::Result<Option<Account>> {
        let account_ids_stmt = Query::select()
            .column(PasswordRecoveryColumn::UserId)
            .from(PasswordRecoveryEntity)
            .and_where(PasswordRecoveryColumn::Id.eq(id.to_string()))
            .to_owned();

        AccountEntity::find()
            .find_with_related(PasswordRecoveryEntity)
            .filter(Condition::any().add(AccountColumn::Id.in_subquery(account_ids_stmt)))
            .all(&self.database)
            .await
            .context("Could not find account by recovery id")?
            .pop()
            .map(Account::try_from)
            .transpose()
    }
}
