use crate::domain::repositories::BookmarkRepository;
use anyhow::Context;
use std::sync::Arc;

#[derive(Debug)]
pub struct GetBookmarksStatsCommand {
    pub user_id: Option<i32>,
}

pub struct BookmarkStats {
    pub total: u64,
    pub private: u64,
}

#[derive(Clone)]
pub struct GetBookmarksStatsUseCase {
    repository: Arc<dyn BookmarkRepository>,
}

impl GetBookmarksStatsUseCase {
    pub fn new(repository: Arc<dyn BookmarkRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        command: GetBookmarksStatsCommand,
    ) -> anyhow::Result<BookmarkStats> {
        let total = self
            .repository
            .count(command.user_id)
            .await
            .context("Could not retrieve total bookmarks count")?;
        let private = match command.user_id {
            None => 0,
            Some(user_id) => self
                .repository
                .count_private(user_id)
                .await
                .context("Could not retrieve private bookmarks count")?,
        };

        Ok(BookmarkStats { total, private })
    }
}
