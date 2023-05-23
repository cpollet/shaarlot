use crate::domain::entities::bookmark::{
    Bookmark, Bookmarks, Filter, Pagination, Sort as BookmarkSort,
};
use crate::domain::repositories::BookmarkRepository;
use crate::domain::values::tag::{CountedTag, Sort as TagSort};

use crate::infrastructure::database::bookmarks::SearchCriteria;
use crate::infrastructure::database::{bookmarks, bookmarks_tags, pins, tags};
use anyhow::{Context, Error};
use async_trait::async_trait;
use entity::bookmark::{Column, Entity};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect, TransactionTrait,
};

#[derive(Clone)]
pub struct DatabaseBookmarkRepository {
    pub database: DatabaseConnection,
}

#[async_trait]
impl BookmarkRepository for DatabaseBookmarkRepository {
    async fn save(&self, bookmark: Bookmark) -> anyhow::Result<Bookmark> {
        let bookmark_id = self
            .database
            .transaction::<_, i32, DbErr>(|txn| {
                Box::pin(async move {
                    let tags = {
                        let mut tags = Vec::new();
                        for tag in bookmark.tags {
                            tags.push(tags::Mutation::create_tag(txn, tag.to_lowercase()).await?)
                        }
                        tags
                    };

                    let bookmark_id = match bookmark.id {
                        None => {
                            bookmarks::Mutation::create_bookmark(
                                txn,
                                bookmark.url,
                                bookmark.title,
                                bookmark.description,
                                bookmark.user_id,
                                bookmark.private,
                            )
                            .await?
                            .id
                        }
                        Some(bookmark_id) => {
                            bookmarks::Mutation::update_bookmark(
                                txn,
                                bookmark_id,
                                bookmark.url,
                                bookmark.title,
                                bookmark.description,
                                bookmark.private,
                            )
                            .await?
                            .ok_or(DbErr::Custom(format!(
                                "Bookmark '{}' not found",
                                bookmark_id
                            )))?
                            .id
                        }
                    };

                    if bookmark.pinned {
                        pins::Mutation::pin(txn, bookmark_id, bookmark.user_id).await?;
                    } else {
                        pins::Mutation::unpin(txn, bookmark_id, bookmark.user_id).await?;
                    }

                    bookmarks_tags::Mutation::delete_all_links(txn, bookmark_id).await?;
                    for tag in tags {
                        bookmarks_tags::Mutation::create_link(txn, bookmark_id, tag.id).await?;
                    }
                    tags::Mutation::delete_orphans(txn).await?;

                    Ok(bookmark_id)
                })
            })
            .await
            .context("Could not save bookmark")?;

        let bookmark = bookmarks::Query::find_visible_by_id(
            &self.database,
            bookmark_id,
            Some(bookmark.user_id),
        )
        .await
        .context("Could not retrieve bookmark")?
        .context("Could not retrieve bookmark: not found")?;

        Ok(bookmark)
    }

    async fn count(&self, user_id: Option<i32>) -> anyhow::Result<u64> {
        bookmarks::Query::count(
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
        bookmarks::Query::count(
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

        let bookmarks =
            bookmarks::Query::find(&self.database, &criteria, &pagination, &sort, user_id)
                .await
                .context("Could not retrieve bookmarks")?;

        let count = bookmarks::Query::count(&self.database, user_id, &criteria)
            .await
            .context("Could not retrieve bookmarks count")?;

        Ok((bookmarks, count as u64))
    }

    async fn find_by_id(&self, user_id: Option<i32>, id: i32) -> anyhow::Result<Option<Bookmark>> {
        bookmarks::Query::find_visible_by_id(&self.database, id, user_id)
            .await
            .context("Could not retrieve bookmark")
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
                    bookmarks::Mutation::delete_bookmark(txn, id).await?;
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
