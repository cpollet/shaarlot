use entity::user::Model;
use entity::user::{ActiveModel, Column, Entity};
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
}

pub struct Mutation;

impl Mutation {
    pub async fn create(
        db: &DatabaseConnection,
        email: String,
        username: String,
        password: String,
    ) -> Result<Model, DbErr> {
        ActiveModel {
            email: Set(email),
            username: Set(username.to_lowercase()),
            password: Set(password),
            ..Default::default()
        }
        .save(db)
        .await
        .and_then(|m| m.try_into_model())
    }
}
