use crate::domain::entities::account::{Account, HashedPassword, NextEmail};
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
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use sea_orm::{ColumnTrait, Condition, NotSet};
use sea_orm::{DbErr, QueryFilter, TransactionError, TransactionTrait};

use lettre::Address;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use uuid::Uuid;

impl Account {
    fn into_active_model(self) -> AccountActiveModel {
        match self.id {
            None => AccountActiveModel {
                username: Set(self.username.to_lowercase()),
                password: Set(self.password.expose_secret_as_str().to_string()),
                email: Set(None),
                email_token: Set(self.next_email.as_ref().map(|e| e.token().to_string())),
                email_token_generation_date: Set(self
                    .next_email
                    .as_ref()
                    .map(|e| DateTimeWithTimeZone::from(*e.token_generation_date()))),
                new_email: Set(self.next_email.map(|e| e.email().to_string())),
                ..Default::default()
            },
            Some(id) => AccountActiveModel {
                id: Unchanged(id),
                username: Unchanged(self.username),
                password: self
                    .new_password
                    .map(|h| h.expose_secret_as_str().to_string())
                    .map(Set)
                    .unwrap_or(NotSet),
                creation_date: Unchanged(DateTimeWithTimeZone::from(self.creation_date)),
                email: Set(self.email.map(|e| e.to_string())),
                email_token: Set(self.next_email.as_ref().map(|e| e.token().to_string())),
                email_token_generation_date: Set(self
                    .next_email
                    .as_ref()
                    .map(|e| DateTimeWithTimeZone::from(*e.token_generation_date()))),
                new_email: Set(self.next_email.map(|e| e.email().to_string())),
            },
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
            id: Some(account.id),
            next_email: NextEmail::try_from_model(&account)?,
            username: account.username,
            password: HashedPassword::from(account.password),
            new_password: None,
            creation_date: account.creation_date.with_timezone(&Utc),
            email: account
                .email
                .map(|e| Address::from_str(&e).context("Invalid email address"))
                .transpose()?,
            password_recoveries,
            password_verified: false,
            events: HashSet::new(),
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
                    Address::from_str(model.new_email.as_ref().expect("must have a new_email"))
                        .context("Invalid email address")?,
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

#[derive(Debug)]
enum SaveDbErr {
    DeleteExpiredRecoveries(DbErr),
    InsertRecovery(DbErr),
    UpdateAccount(DbErr),
}

impl Display for SaveDbErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveDbErr::DeleteExpiredRecoveries(e) => {
                write!(f, "Could not delete expired recoveries: {:?}", e)
            }
            SaveDbErr::InsertRecovery(e) => write!(f, "Could not insert account: {:?}", e),
            SaveDbErr::UpdateAccount(e) => write!(f, "Could not update account: {:?}", e),
        }
    }
}

impl Error for SaveDbErr {}

impl DatabaseAccountRepository {
    async fn insert(&self, account: Account) -> anyhow::Result<Account> {
        let account = account.into_active_model();

        let account = account
            .insert(&self.database)
            .await
            .context("Could not insert account")?;

        self.find_by_id(account.id)
            .await
            .transpose()
            .expect("account must exist")
    }

    async fn update(&self, account: Account) -> anyhow::Result<Account> {
        let result = self
            .database
            .transaction::<_, i32, SaveDbErr>(|txn| {
                Box::pin(async move {
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
                        .exec(txn)
                        .await
                        .map_err(SaveDbErr::DeleteExpiredRecoveries)?;

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
                            .insert(txn)
                            .await
                            .map_err(SaveDbErr::InsertRecovery)?;
                    }

                    let account = account.into_active_model();
                    let account = account
                        .update(txn)
                        .await
                        .map_err(SaveDbErr::UpdateAccount)?;

                    Ok(account.id)
                })
            })
            .await;

        let id = match result {
            Ok(id) => id,
            Err(TransactionError::Transaction(e)) => {
                return Err(e).context("Could not save account")
            }
            Err(TransactionError::Connection(e)) => {
                return Err(e).context("Count not save account")
            }
        };

        self.find_by_id(id)
            .await
            .transpose()
            .expect("account must exist")
    }
}

#[async_trait]
impl AccountRepository for DatabaseAccountRepository {
    async fn save(&self, account: Account) -> anyhow::Result<Account> {
        match account.id {
            None => self.insert(account).await,
            Some(_) => self.update(account).await,
        }
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