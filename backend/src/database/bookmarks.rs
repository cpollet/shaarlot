use entity::bookmark::Model;
use entity::bookmark::{ActiveModel, Entity};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, TryIntoModel};

pub struct Query;

impl Query {
    pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
        Entity::find().all(db).await
    }

    pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<Model>, DbErr> {
        Entity::find_by_id(id).one(db).await
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn create_bookmark(
        db: &DatabaseConnection,
        url: String,
        title: Option<String>,
        description: Option<String>,
    ) -> Result<Model, DbErr> {
        ActiveModel {
            url: Set(url),
            title: Set(title),
            description: Set(description),
            ..Default::default()
        }
        .save(db)
        .await
        .and_then(|m| m.try_into_model())
    }

    pub async fn update_bookmark(
        db: &DatabaseConnection,
        id: i32,
        url: String,
        title: Option<String>,
        description: Option<String>,
    ) -> Result<Option<Model>, DbErr> {
        let model = Entity::find_by_id(id)
            .one(db)
            .await?
            .map(Into::<ActiveModel>::into);
        if let Some(mut model) = model {
            model.url = Set(url);
            model.title = Set(title);
            model.description = Set(description);
            Ok(Some(model.update(db).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_bookmark(db: &DatabaseConnection, id: i32) -> Result<Option<()>, DbErr> {
        if Entity::delete_by_id(id).exec(db).await?.rows_affected == 1 {
            return Ok(Some(()));
        }
        Ok(None)
    }
}
