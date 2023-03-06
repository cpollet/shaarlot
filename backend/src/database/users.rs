use chrono::Utc;
use entity::user::Model;
use entity::user::{ActiveModel, Column, Entity};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    TryIntoModel,
};

pub struct Query;

impl Query {
    pub async fn find_by_username(
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::Username.eq(username.to_lowercase()))
            .one(db)
            .await
    }

    pub async fn find_by_email_token(
        db: &DatabaseConnection,
        email_token: &str,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::EmailToken.eq(email_token))
            .one(db)
            .await
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn create(
        db: &DatabaseConnection,
        email: String,
        username: String,
        password: String,
        email_token: String,
    ) -> Result<Model, DbErr> {
        ActiveModel {
            email: Set(email),
            username: Set(username.to_lowercase()),
            password: Set(password),
            email_token: Set(Some(email_token)),
            email_token_generation_date: Set(Some(DateTimeWithTimeZone::from(Utc::now()))),
            ..Default::default()
        }
        .save(db)
        .await
        .and_then(|m| m.try_into_model())
    }

    pub async fn remove_email_token(
        db: &DatabaseConnection,
        id: i32,
    ) -> Result<Option<Model>, DbErr> {
        let model = Entity::find_by_id(id)
            .one(db)
            .await?
            .map(Into::<ActiveModel>::into);
        if let Some(mut model) = model {
            model.email_token = Set(None);
            model.email_token_generation_date = Set(None);
            Ok(Some(model.update(db).await?))
        } else {
            Ok(None)
        }
    }
}
