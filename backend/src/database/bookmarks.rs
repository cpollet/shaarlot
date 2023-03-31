use chrono::Utc;
use entity::bookmark::{ActiveModel, Entity};
use entity::bookmark::{Column, Model};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DbErr, EntityTrait, Order, QueryOrder, Select, TryIntoModel,
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
    pub async fn find_all<C>(db: &C) -> Result<Vec<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        Entity::find().all(db).await
    }

    pub async fn find_all_order_by<C>(
        db: &C,
        order: SortOrder,
    ) -> Result<Vec<(Model, Vec<entity::tag::Model>)>, DbErr>
    where
        C: ConnectionTrait,
    {
        order
            .add_clause(Entity::find())
            .find_with_related(entity::tag::Entity)
            .all(db)
            .await
    }

    pub async fn find_by_id<C>(
        db: &C,
        id: i32,
    ) -> Result<Option<(Model, Vec<entity::tag::Model>)>, DbErr>
    where
        C: ConnectionTrait,
    {
        Ok(Entity::find_by_id(id)
            .find_with_related(entity::tag::Entity)
            .all(db)
            .await?
            .pop())
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn create_bookmark<C>(
        db: &C,
        url: String,
        title: Option<String>,
        description: Option<String>,
        user_id: i32,
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        ActiveModel {
            url: Set(url),
            title: Set(title),
            description: Set(description),
            user_id: Set(user_id),
            ..Default::default()
        }
        .save(db)
        .await
        .and_then(|m| m.try_into_model())
    }

    pub async fn update_bookmark<C>(
        db: &C,
        id: i32,
        url: String,
        title: Option<String>,
        description: Option<String>,
    ) -> Result<Option<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
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

    pub async fn delete_bookmark<C>(db: &C, id: i32) -> Result<Option<()>, DbErr>
    where
        C: ConnectionTrait,
    {
        if Entity::delete_by_id(id).exec(db).await?.rows_affected == 1 {
            return Ok(Some(()));
        }
        Ok(None)
    }
}
