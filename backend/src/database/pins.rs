use entity::pin::{ActiveModel, Column, Entity};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter};

pub struct Mutation;

impl Mutation {
    pub async fn unpin<C>(db: &C, bookmark_id: i32, user_id: i32) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        Entity::delete(ActiveModel {
            bookmark_id: Set(bookmark_id),
            user_id: Set(user_id),
        })
        .exec(db)
        .await
        .map_err(|e| {
            log::error!("unpin {:?}", e);
            e
        })
        .map(|_| ())
    }

    pub async fn pin<C>(db: &C, bookmark_id: i32, user_id: i32) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        if Entity::find()
            .filter(Column::BookmarkId.eq(bookmark_id))
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await?
            .is_none()
        {
            log::info!("pinning");
            ActiveModel {
                bookmark_id: Set(bookmark_id),
                user_id: Set(user_id),
            }
            .insert(db)
            .await
            .map_err(|e| {
                log::error!("{:?}", e);
                e
            })?;
        }

        log::info!("already pinned");
        Ok(())
    }
}
