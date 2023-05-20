use crate::domain::entities::bookmark::{Bookmark, Filter, Pagination, Sort};
use crate::infrastructure::database::tags;
use chrono::{DateTime, Utc};
use entity::bookmark::{ActiveModel, Entity};
use entity::bookmark::{Column, Model};
use entity::{bookmark_tag, pin, tag};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sea_query::extension::postgres::PgExpr;
use sea_orm::sea_query::{Expr, IntoCondition};
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, FromQueryResult,
    JoinType, Order, QueryFilter, QueryOrder, QuerySelect, QueryTrait, RelationTrait, Select,
    TryIntoModel, Value,
};

// todo functions should return Model, repository should convert to entities

#[derive(Debug, Default)]
pub struct SearchCriteria {
    pub tags: Vec<String>,
    pub search: Vec<String>,
    pub filter: Filter,
}

pub struct Query;

enum SearchBy<'a> {
    Id(i32, Option<i32>),
    Criteria(&'a SearchCriteria, &'a Pagination, &'a Sort, Option<i32>),
}

impl Sort {
    fn add_clause(&self, select: Select<Entity>, user_id: Option<i32>) -> Select<Entity> {
        let select = match user_id {
            None => select,
            Some(_) => select.order_by_asc(pin::Column::UserId),
        };
        match self {
            Sort::CreationDateDesc => select.order_by(Column::CreationDate, Order::Desc),
            Sort::CreationDateAsc => select.order_by(Column::CreationDate, Order::Asc),
        }
    }
}

impl<'a> SearchBy<'a> {
    fn add_pinned_flag(select: Select<Entity>, user_id: Option<i32>) -> Select<Entity> {
        match user_id {
            Some(user_id) => select
                .column_as(
                    #[allow(deprecated)]
                    Expr::tbl(pin::Entity, pin::Column::UserId).is_not_null(),
                    "pinned",
                )
                .join_rev(
                    JoinType::LeftJoin,
                    pin::Relation::Bookmark
                        .def()
                        .on_condition(move |pin, _bookmark| {
                            #[allow(deprecated)]
                            Expr::tbl(pin, pin::Column::UserId)
                                .eq(user_id)
                                .into_condition()
                        }),
                ),
            None => select.column_as(Expr::value(Value::Bool(Some(false))), "pinned"),
        }
    }

    fn tags_condition(tags: &Vec<String>) -> Condition {
        let mut tags_condition = Condition::all();

        if tags.is_empty() {
            return tags_condition;
        }

        for expr in tags.iter().map(|t| {
            Column::Id.in_subquery(
                bookmark_tag::Entity::find()
                    .select_only()
                    .column(bookmark_tag::Column::BookmarkId)
                    .filter(
                        bookmark_tag::Column::TagId.in_subquery(
                            tag::Entity::find()
                                .select_only()
                                .column(tag::Column::Id)
                                .filter(tag::Column::Name.eq(t))
                                .into_query(),
                        ),
                    )
                    .into_query(),
            )
        }) {
            tags_condition = tags_condition.add(expr);
        }

        tags_condition
    }

    fn search_condition(search: &Vec<String>) -> Condition {
        let mut search_condition = Condition::all();

        if search.is_empty() {
            return search_condition;
        }

        for expr in search.iter().map(|t| {
            let like_expr = format!("%{}%", t);
            let inner = Condition::any();
            let inner = inner.add(Expr::col(Column::Title).ilike(like_expr.as_str()));
            let inner = inner.add(Expr::col(Column::Description).ilike(like_expr.as_str()));
            inner.add(Expr::col(Column::Url).ilike(like_expr.as_str()))
        }) {
            search_condition = search_condition.add(expr);
        }

        search_condition
    }
}

impl<'a> From<SearchBy<'a>> for Select<Entity> {
    fn from(value: SearchBy<'a>) -> Self {
        let (select, user_id) = match value {
            SearchBy::Id(id, user_id) => (
                Entity::find_by_id(id.to_owned())
                    .filter(Query::visible_condition(user_id.to_owned(), Filter::All)),
                user_id,
            ),
            SearchBy::Criteria(criteria, page, order, user_id) => (
                order.add_clause(
                    Entity::find()
                        .filter(Query::visible_condition(
                            user_id.to_owned(),
                            criteria.filter,
                        ))
                        .filter(SearchBy::tags_condition(&criteria.tags))
                        .filter(SearchBy::search_condition(&criteria.search))
                        .offset(page.size * page.page)
                        .limit(page.size),
                    user_id,
                ),
                user_id,
            ),
        };

        SearchBy::add_pinned_flag(select, user_id.to_owned())
    }
}

// todo move to repository?
impl From<(BookmarkModelWithPinned, Vec<tag::Model>)> for Bookmark {
    fn from(value: (BookmarkModelWithPinned, Vec<tag::Model>)) -> Self {
        Self {
            id: Some(value.0.id),
            user_id: value.0.user_id,
            url: value.0.url,
            title: value.0.title,
            description: value.0.description,
            tags: value.1.into_iter().map(|t| t.name).collect(),
            creation_date: value.0.creation_date.with_timezone(&Utc),
            update_date: value.0.update_date.map(|d| d.with_timezone(&Utc)),
            private: value.0.private,
            pinned: value.0.pinned,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FromQueryResult)]
struct BookmarkModelWithPinned {
    pub id: i32,
    pub url: String,
    pub description: Option<String>,
    pub title: Option<String>,
    pub creation_date: DateTimeWithTimeZone,
    pub update_date: Option<DateTimeWithTimeZone>,
    pub user_id: i32,
    pub private: bool,
    pub pinned: bool,
}

impl Query {
    async fn find_by<'a, C>(db: &C, search_by: SearchBy<'a>) -> Result<Vec<Bookmark>, DbErr>
    where
        C: ConnectionTrait,
    {
        let bookmarks = Into::<Select<Entity>>::into(search_by)
            .into_model::<BookmarkModelWithPinned>()
            .all(db)
            .await
            .map_err(|e| {
                log::error!("{:?}", e);
                e
            })?;

        let mut tagged_bookmarks = Vec::with_capacity(bookmarks.len());
        for bookmark in bookmarks {
            let tags = tags::Query::find_by_bookmark_id(db, bookmark.id).await?;
            tagged_bookmarks.push(Bookmark::from((bookmark, tags)));
        }

        Ok(tagged_bookmarks)
    }

    pub async fn find<'a, C>(
        db: &C,
        criteria: &'a SearchCriteria,
        page: &'a Pagination,
        order: &'a Sort,
        user_id: Option<i32>,
    ) -> Result<Vec<Bookmark>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::find_by(db, SearchBy::Criteria(criteria, page, order, user_id)).await
    }

    pub fn visible_condition(user_id: Option<i32>, filter: Filter) -> Condition {
        match filter {
            Filter::All => {
                let mut visible = Condition::any().add(Column::Private.eq(false));
                if let Some(user_id) = user_id {
                    visible = visible.add(Column::UserId.eq(user_id));
                }
                visible
            }
            Filter::Private => {
                if user_id.is_none() {
                    return Condition::all().add(Column::UserId.is_null());
                }

                let mut visible = Condition::all().add(Column::Private.eq(true));
                if let Some(user_id) = user_id {
                    visible = visible.add(Column::UserId.eq(user_id));
                }
                visible
            }
            Filter::Public => Condition::any().add(Column::Private.eq(false)),
        }
    }

    pub async fn count<C>(
        db: &C,
        user_id: Option<i32>,
        criteria: &SearchCriteria,
    ) -> Result<i64, DbErr>
    where
        C: ConnectionTrait,
    {
        let r: Option<i64> = Entity::find()
            .select_only()
            .column_as(Expr::col(Column::Id).count(), "count")
            .filter(Self::visible_condition(user_id, criteria.filter))
            .filter(SearchBy::tags_condition(&criteria.tags))
            .filter(SearchBy::search_condition(&criteria.search))
            .into_tuple()
            .one(db)
            .await?;
        Ok(r.unwrap_or_default())
    }

    pub async fn find_visible_by_id<C>(
        db: &C,
        id: i32,
        user_id: Option<i32>,
    ) -> Result<Option<Bookmark>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::find_by(db, SearchBy::Id(id, user_id))
            .await
            .map(|mut r| r.pop())
    }

    pub async fn find_by_url<C>(db: &C, user_id: i32, url: &str) -> Result<Option<i32>, DbErr>
    where
        C: ConnectionTrait,
    {
        Ok(Entity::find()
            // todo select only
            .filter(Column::Url.eq(url))
            .filter(Column::UserId.eq(user_id))
            .all(db)
            .await?
            .pop()
            .map(|m| m.id))
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
        private: bool,
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        ActiveModel {
            url: Set(url),
            title: Set(title),
            description: Set(description),
            user_id: Set(user_id),
            private: Set(private),
            ..Default::default()
        }
        .save(db)
        .await
        .and_then(|m| m.try_into_model())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn import_bookmark<C>(
        db: &C,
        url: String,
        title: Option<String>,
        description: Option<String>,
        creation_date: DateTime<Utc>,
        update_date: Option<DateTime<Utc>>,
        user_id: i32,
        private: bool,
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        ActiveModel {
            id: Default::default(),
            url: Set(url),
            title: Set(title),
            description: Set(description),
            user_id: Set(user_id),
            private: Set(private),
            creation_date: Set(creation_date.into()),
            update_date: Set(update_date.map(|d| d.into())),
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
        private: bool,
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
            model.private = Set(private);
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
