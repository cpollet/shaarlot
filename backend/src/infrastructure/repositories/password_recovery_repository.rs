use crate::domain::entities::password_recovery::{ObfuscatedPasswordRecovery, PasswordRecovery};
use crate::domain::repositories::PasswordRecoveryRepository;
use anyhow::Context;
use async_trait::async_trait;
use chrono::Utc;
use entity::password_recovery;
use entity::password_recovery::{Column, Model};
use sea_orm::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone)]
pub struct DatabasePasswordRecoveryRepository {
    pub database: DatabaseConnection,
}

impl TryFrom<Model> for ObfuscatedPasswordRecovery {
    type Error = anyhow::Error;

    fn try_from(value: Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::from_str(value.id.as_str()).context("Invalid UUID")?,
            hashed_token: value.token,
            user_id: value.user_id,
            generation_date: value.generation_date.with_timezone(&Utc),
        })
    }
}

#[async_trait]
impl PasswordRecoveryRepository for DatabasePasswordRecoveryRepository {
    async fn save(
        &self,
        password_recovery: PasswordRecovery,
    ) -> anyhow::Result<ObfuscatedPasswordRecovery> {
        password_recovery::ActiveModel {
            id: Set(password_recovery.id.to_string()),
            user_id: Set(password_recovery.user_id),
            token: Set(password_recovery.hashed_token),
            ..Default::default()
        }
        .insert(&self.database)
        .await
        .context("Could not insert password recovery")
        .and_then(ObfuscatedPasswordRecovery::try_from)
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        password_recovery::Entity::delete_by_id(id.to_string())
            .exec(&self.database)
            .await
            .context("Could not delete recovery")?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ObfuscatedPasswordRecovery>> {
        password_recovery::Entity::find()
            .filter(Column::Id.eq(id.to_string()))
            .one(&self.database)
            .await
            .context("Could not find account by username")?
            .map(ObfuscatedPasswordRecovery::try_from)
            .transpose()
    }
}
