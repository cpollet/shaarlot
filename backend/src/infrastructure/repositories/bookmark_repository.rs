use crate::domain::entities::bookmark::{
    Bookmark, Bookmarks, Filter, Pagination, Sort as BookmarkSort, Sort,
};
use crate::domain::repositories::{BookmarkRepository, SearchCriteria};
use crate::domain::values::tag::{CountedTag, Sort as TagSort};

use crate::infrastructure::database::{bookmarks_tags, pins, tags};
use anyhow::{Context, Error};
use async_trait::async_trait;
use chrono::Utc;
use entity::bookmark::{ActiveModel, Column, Entity};
use entity::{bookmark_tag, pin, tag};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sea_query::extension::postgres::PgExpr;
use sea_orm::sea_query::{Expr, IntoCondition};
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbErr,
    EntityTrait, FromQueryResult, JoinType, NotSet, Order, QueryFilter, QueryOrder, QuerySelect,
    QueryTrait, RelationTrait, Select, TransactionTrait, TryIntoModel, Value,
};

#[derive(Clone)]
pub struct DatabaseBookmarkRepository {
    pub database: DatabaseConnection,
}

#[async_trait]
impl BookmarkRepository for DatabaseBookmarkRepository {
    async fn save(&self, bookmark: Bookmark) -> anyhow::Result<Bookmark> {
        let user_id = bookmark.user_id;
        let bookmark_id = self
            .database
            .transaction::<_, i32, DbErr>(|txn| {
                Box::pin(async move { Self::save_internal(txn, bookmark, false).await })
            })
            .await
            .context("Could not save bookmark")?;

        let bookmark = DatabaseBookmarkRepository::find_by(
            &self.database,
            SearchBy::Id(bookmark_id, Some(user_id)),
        )
        .await
        .context("Could not retrieve bookmark")?
        .pop()
        .context("Could not retrieve bookmark: not found")?;

        Ok(bookmark)
    }

    async fn import(&self, bookmarks: Vec<Bookmark>) -> anyhow::Result<i32> {
        let count = self
            .database
            .transaction::<_, i32, DbErr>(|txn| {
                Box::pin(async move {
                    let mut count = 0;
                    for bookmark in bookmarks {
                        Self::save_internal(txn, bookmark, true).await?;
                        count += 1;
                    }
                    Ok(count)
                })
            })
            .await
            .context("Could not save bookmarks")?;
        Ok(count)
    }

    async fn count(&self, user_id: Option<i32>) -> anyhow::Result<u64> {
        DatabaseBookmarkRepository::count(
            &self.database,
            user_id,
            &SearchCriteria {
                ..SearchCriteria::default()
            },
        )
        .await
        .context("Could not retrieve bookmarks count")
        .map(|c| c as u64)
    }

    async fn count_private(&self, user_id: i32) -> anyhow::Result<u64> {
        DatabaseBookmarkRepository::count(
            &self.database,
            Some(user_id),
            &SearchCriteria {
                filter: Filter::Private,
                ..SearchCriteria::default()
            },
        )
        .await
        .context("Could not retrieve bookmarks count")
        .map(|c| c as u64)
    }

    async fn find(
        &self,
        user_id: Option<i32>,
        tags: Vec<String>,
        search: Vec<String>,
        filter: Filter,
        pagination: Pagination,
        sort: BookmarkSort,
    ) -> anyhow::Result<(Bookmarks, u64)> {
        let criteria = SearchCriteria {
            tags,
            search,
            filter,
        };

        let bookmarks = DatabaseBookmarkRepository::find_by(
            &self.database,
            SearchBy::Criteria(&criteria, &pagination, &sort, user_id),
        )
        .await
        .context("Could not retrieve bookmarks")?;

        let count = DatabaseBookmarkRepository::count(&self.database, user_id, &criteria)
            .await
            .context("Could not retrieve bookmarks count")?;

        Ok((bookmarks, count as u64))
    }

    async fn find_by_id(&self, user_id: Option<i32>, id: i32) -> anyhow::Result<Option<Bookmark>> {
        DatabaseBookmarkRepository::find_by(&self.database, SearchBy::Id(id, user_id))
            .await
            .context("Could not retrieve bookmark")
            .map(|mut b| b.pop())
    }

    async fn find_id_by_url(&self, user_id: i32, url: &str) -> anyhow::Result<Option<i32>> {
        Entity::find()
            .select_only()
            .column(Column::Id)
            .filter(Column::Url.eq(url))
            .filter(Column::UserId.eq(user_id))
            .into_tuple()
            .one(&self.database)
            .await
            .map_err(Error::msg)
            .context("Could not find by url")
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        self.database
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    bookmarks_tags::Mutation::delete_all_links(txn, id).await?;
                    tags::Mutation::delete_orphans(txn).await?;
                    Entity::delete_by_id(id).exec(txn).await?;
                    Ok(())
                })
            })
            .await
            .context("Could not delete bookmark")
    }

    async fn find_tags(
        &self,
        user_id: Option<i32>,
        sort: TagSort,
    ) -> anyhow::Result<Vec<CountedTag>> {
        tags::Query::find_by_user_id_order_by(&self.database, user_id, sort)
            .await
            .context("Could not load tags")
    }
}

impl DatabaseBookmarkRepository {
    async fn save_internal<C>(db: &C, bookmark: Bookmark, force_date: bool) -> Result<i32, DbErr>
    where
        C: ConnectionTrait,
    {
        let tags = {
            let mut tags = Vec::new();
            for tag in bookmark.tags {
                tags.push(tags::Mutation::create_tag(db, tag.to_lowercase()).await?)
            }
            tags
        };

        let bookmark_id = match bookmark.id {
            None => {
                let creation_date = if force_date {
                    Set(bookmark.creation_date.into())
                } else {
                    NotSet
                };
                let update_date = if force_date {
                    Set(bookmark.update_date.map(|d| d.into()))
                } else {
                    NotSet
                };

                ActiveModel {
                    url: Set(bookmark.url),
                    title: Set(bookmark.title),
                    description: Set(bookmark.description),
                    user_id: Set(bookmark.user_id),
                    private: Set(bookmark.private),
                    creation_date,
                    update_date,
                    ..Default::default()
                }
                .save(db)
                .await
                .and_then(|m| m.try_into_model())?
                .id
            }
            Some(bookmark_id) => {
                if force_date {
                    panic!("Cannot use force_date when updating existing bookmark");
                }

                let mut model = Entity::find_by_id(bookmark_id)
                    .one(db)
                    .await?
                    .map(Into::<ActiveModel>::into)
                    .ok_or(DbErr::Custom(format!(
                        "Bookmark '{}' not found",
                        bookmark_id
                    )))?;

                model.url = Set(bookmark.url);
                model.title = Set(bookmark.title);
                model.description = Set(bookmark.description);
                model.update_date = Set(Some(DateTimeWithTimeZone::from(Utc::now())));
                model.private = Set(bookmark.private);
                model.update(db).await?.id
            }
        };

        if bookmark.pinned {
            pins::Mutation::pin(db, bookmark_id, bookmark.user_id).await?;
        } else {
            pins::Mutation::unpin(db, bookmark_id, bookmark.user_id).await?;
        }

        bookmarks_tags::Mutation::delete_all_links(db, bookmark_id).await?;
        for tag in tags {
            bookmarks_tags::Mutation::create_link(db, bookmark_id, tag.id).await?;
        }
        tags::Mutation::delete_orphans(db).await?;

        Ok(bookmark_id)
    }

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
            .filter(criteria.filter.visible_condition(user_id))
            .filter(criteria.tags_condition())
            .filter(criteria.search_condition())
            .into_tuple()
            .one(db)
            .await?;
        Ok(r.unwrap_or_default())
    }
}

impl SearchCriteria {
    fn tags_condition(&self) -> Condition {
        let mut tags_condition = Condition::all();

        for expr in self.tags.iter().map(|t| {
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

    fn search_condition(&self) -> Condition {
        let mut search_condition = Condition::all();

        for expr in self.search.iter().map(|t| {
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

impl Filter {
    pub fn visible_condition(&self, user_id: Option<i32>) -> Condition {
        match self {
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

pub enum SearchBy<'a> {
    Id(i32, Option<i32>),
    Criteria(&'a SearchCriteria, &'a Pagination, &'a Sort, Option<i32>),
}

impl<'a> From<SearchBy<'a>> for Select<Entity> {
    fn from(value: SearchBy<'a>) -> Self {
        let (select, user_id) = match value {
            SearchBy::Id(id, user_id) => (
                Entity::find_by_id(id.to_owned()).filter(Filter::All.visible_condition(user_id)),
                user_id,
            ),
            SearchBy::Criteria(criteria, page, order, user_id) => (
                order.add_clause(
                    Entity::find()
                        .filter(criteria.filter.visible_condition(user_id))
                        .filter(criteria.tags_condition())
                        .filter(criteria.search_condition())
                        .offset(page.size * page.page)
                        .limit(page.size),
                    user_id,
                ),
                user_id,
            ),
        };

        add_pinned_flag(select, user_id.to_owned())
    }
}

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
