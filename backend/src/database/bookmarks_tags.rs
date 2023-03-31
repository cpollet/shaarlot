use entity::bookmark_tag::{ActiveModel, Column, Entity, Model};
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, TryIntoModel,
};

pub struct Mutation;

impl Mutation {
    pub async fn create_link<C>(db: &C, bookmark_id: i32, tag_id: i32) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        ActiveModel {
            bookmark_id: Set(bookmark_id),
            tag_id: Set(tag_id),
        }
        .insert(db)
        .await
        .and_then(|m| m.try_into_model())
    }

    pub async fn delete_all_links<C>(db: &C, bookmark_id: i32) -> Result<u64, DbErr>
    where
        C: ConnectionTrait,
    {
        Entity::delete_many()
            .filter(Column::BookmarkId.eq(bookmark_id))
            .exec(db)
            .await
            .map(|r| r.rows_affected)
    }
}
