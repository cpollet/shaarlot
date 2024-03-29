use crate::database::bookmarks;
use crate::database::bookmarks::Filter;
use entity::tag::{ActiveModel, Column, Entity, Model};
use entity::{bookmark, bookmark_tag};
use migration::JoinType;
use sea_orm::sea_query::SimpleExpr;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, DbErr, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Statement, TryIntoModel,
};
use sea_orm::{FromQueryResult, Order, Select};

pub enum SortOrder {
    Name,
    Count,
}

impl SortOrder {
    fn add_clause(&self, select: Select<Entity>) -> Select<Entity> {
        match self {
            SortOrder::Name => select.order_by(Column::Name, Order::Asc),
            SortOrder::Count => select.order_by_desc(SimpleExpr::Custom("\"count\"".to_owned())),
        }
    }
}

impl TryFrom<&str> for SortOrder {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "name" => Ok(SortOrder::Name),
            "count" => Ok(SortOrder::Count),
            _ => Err(format!("{} is not valid", value)),
        }
    }
}

pub struct Query;

#[derive(FromQueryResult)]
pub struct TagsAndCount {
    pub id: i32,
    pub name: String,
    pub count: i64,
}

impl Query {
    pub async fn find_by_name<C>(db: &C, name: &str) -> Result<Option<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        Entity::find().filter(Column::Name.eq(name)).one(db).await
    }

    pub async fn find_by_user_id_order_by<C>(
        db: &C,
        user_id: Option<i32>,
        order: SortOrder,
    ) -> Result<Vec<TagsAndCount>, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut select = Entity::find()
            .column_as(Column::Id.count(), "count")
            .join_rev(JoinType::Join, bookmark_tag::Relation::Tag.def())
            .join_rev(
                JoinType::Join,
                bookmark::Entity::belongs_to(bookmark_tag::Entity)
                    .from(bookmark::Column::Id)
                    .to(bookmark_tag::Column::BookmarkId)
                    .into(),
            )
            .filter(bookmarks::Query::visible_condition(user_id, Filter::All))
            .group_by(Column::Id)
            .group_by(Column::Name);
        select = order.add_clause(select);

        select.into_model::<TagsAndCount>().all(db).await
    }

    pub async fn find_by_bookmark_id<C>(db: &C, bookmark_id: i32) -> Result<Vec<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        Entity::find()
            .join_rev(JoinType::Join, bookmark_tag::Relation::Tag.def())
            .filter(bookmark_tag::Column::BookmarkId.eq(bookmark_id))
            .all(db)
            .await
    }
}

pub struct Mutation;

impl Mutation {
    pub async fn create_tag<C>(db: &C, name: String) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        if let Some(tag) = Query::find_by_name(db, &name).await? {
            return Ok(tag);
        }

        ActiveModel {
            name: Set(name),
            ..Default::default()
        }
        .save(db)
        .await
        .and_then(|m| m.try_into_model())
    }

    pub async fn delete_orphans<C>(db: &C) -> Result<u64, DbErr>
    where
        C: ConnectionTrait,
    {
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "delete from tag where id not in (select tag_id from bookmark_tag);".to_owned(),
        ))
        .await
        .map(|r| r.rows_affected())
    }
}
