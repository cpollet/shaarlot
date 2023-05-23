use crate::domain::entities::account::Account;
use crate::domain::entities::bookmark::{
    Bookmark, Bookmarks, Filter, Pagination, Sort as BookmarkSort,
};
use crate::domain::values::tag::{CountedTag, Sort as TagSort};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait BookmarkRepository: Sync + Send {
    async fn save(&self, bookmark: Bookmark) -> anyhow::Result<Bookmark>;

    async fn count(&self, user_id: Option<i32>) -> anyhow::Result<u64>;

    async fn count_private(&self, user_id: i32) -> anyhow::Result<u64>;

    async fn find(
        &self,
        user_id: Option<i32>,
        tags: Vec<String>,
        search: Vec<String>,
        filter: Filter,
        pagination: Pagination,
        sort: BookmarkSort,
    ) -> anyhow::Result<(Bookmarks, u64)>;

    async fn find_by_id(&self, user_id: Option<i32>, id: i32) -> anyhow::Result<Option<Bookmark>>;

    async fn find_id_by_url(&self, user_id: i32, url: &str) -> anyhow::Result<Option<i32>>;

    async fn delete(&self, id: i32) -> anyhow::Result<()>;

    async fn find_tags(
        &self,
        user_id: Option<i32>,
        sort: TagSort,
    ) -> anyhow::Result<Vec<CountedTag>>;
}

#[async_trait]
pub trait AccountRepository: Sync + Send {
    async fn save(&self, account: Account) -> anyhow::Result<Account>;

    async fn find_by_id(&self, id: i32) -> anyhow::Result<Option<Account>>;

    async fn find_by_email_token(&self, token: Uuid) -> anyhow::Result<Option<Account>>;

    async fn find_by_username(&self, email: &str) -> anyhow::Result<Option<Account>>;

    async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<Account>>;

    async fn find_by_recovery_id(&self, id: Uuid) -> anyhow::Result<Option<Account>>;
}
