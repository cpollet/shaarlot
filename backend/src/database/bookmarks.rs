use chrono::Utc;
use entity::bookmark::{ActiveModel, Entity};
use entity::bookmark::{Column, Model};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Order, QueryOrder, Select,
    TryIntoModel,
};

pub enum SortOrder {
    CreationDateDesc,
    CreationDateAsc,
}

impl SortOrder {
    fn add_clause(self, select: Select<Entity>) -> Select<Entity> {
        match self {
            SortOrder::CreationDateDesc => select.order_by(Column::CreationDate, Order::Desc),
            SortOrder::CreationDateAsc => select.order_by(Column::CreationDate, Order::Asc),
        }
    }
}

impl TryFrom<&str> for SortOrder {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "creation_date:asc" => Ok(SortOrder::CreationDateAsc),
            "creation_date:desc" => Ok(SortOrder::CreationDateDesc),
            _ => Err(format!("{} is not valid", value)),
        }
    }
}

pub struct Query;

impl Query {
    pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
        Entity::find().all(db).await
    }

    pub async fn find_all_order_by(
        db: &DatabaseConnection,
        order: SortOrder,
    ) -> Result<Vec<Model>, DbErr> {
        order.add_clause(Entity::find()).all(db).await
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
            model.update_date = Set(Some(DateTimeWithTimeZone::from(Utc::now())));
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
