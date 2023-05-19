use entity::password_recovery::{ActiveModel, Column, Entity, Model};
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, TryIntoModel,
};

pub struct Query;

impl Query {
    pub async fn find_by_id(db: &DatabaseConnection, id: &str) -> Result<Option<Model>, DbErr> {
        Entity::find().filter(Column::Id.eq(id)).one(db).await
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn create(
        db: &DatabaseConnection,
        id: &str,
        user_id: i32,
        token: String,
    ) -> Result<Model, DbErr> {
        ActiveModel {
            id: Set(id.to_string()),
            user_id: Set(user_id),
            token: Set(token),
            ..Default::default()
        }
        .insert(db)
        .await
        .and_then(|m| m.try_into_model())
    }

    pub async fn delete<C>(db: &C, id: &str) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        Entity::delete_by_id(id).exec(db).await?;
        Ok(())
    }
}
