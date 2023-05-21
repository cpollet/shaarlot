use crate::domain::entities::account::Account;
use crate::domain::entities::bookmark::{Bookmark, Bookmarks, Filter, Pagination, Sort};
use crate::domain::entities::password_recovery::{ObfuscatedPasswordRecovery, PasswordRecovery};
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
        sort: Sort,
    ) -> anyhow::Result<(Bookmarks, u64)>;

    // todo change user_id, or order
    // todo rename find_visible_by_id
    async fn find_by_id(&self, user_id: Option<i32>, id: i32) -> anyhow::Result<Option<Bookmark>>;

    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}

#[async_trait]
pub trait AccountRepository: Sync + Send {
    async fn save(&self, account: Account) -> anyhow::Result<Account>;

    async fn find_by_id(&self, id: i32) -> anyhow::Result<Option<Account>>;

    async fn find_by_email_token(&self, token: Uuid) -> anyhow::Result<Option<Account>>;

    async fn find_by_username(&self, email: &str) -> anyhow::Result<Option<Account>>;

    async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<Account>>;
}

#[async_trait]
pub trait PasswordRecoveryRepository: Sync + Send {
    async fn save(
        &self,
        password_recovery: PasswordRecovery,
    ) -> anyhow::Result<ObfuscatedPasswordRecovery>;

    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ObfuscatedPasswordRecovery>>;
}
