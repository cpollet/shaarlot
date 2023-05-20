use chrono::Utc;
use entity::account::Model;
use entity::account::{ActiveModel, Column, Entity};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, TryIntoModel,
};
use uuid::Uuid;

pub struct Query;

impl Query {
    pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<Model>, DbErr> {
        Entity::find_by_id(id).one(db).await
    }

    pub async fn find_by_username(
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::Username.eq(username.to_lowercase()))
            .one(db)
            .await
    }

    pub async fn find_by_email(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::Email.eq(email.to_lowercase()))
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
            email: Set(None),
            new_email: Set(Some(email)),
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

    pub async fn update<C>(
        db: &C,
        user: Model,
        new_password: Option<String>,
        new_email: Option<(String, Uuid)>,
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut user = ActiveModel::from(user);

        if let Some((email, token)) = new_email {
            user.new_email = Set(Some(email));
            user.email_token = Set(Some(token.to_string()));
            user.email_token_generation_date = Set(Some(DateTimeWithTimeZone::from(Utc::now())));
        }

        if let Some(new_password) = new_password {
            user.password = Set(new_password);
        }

        user.update(db).await
    }
}
