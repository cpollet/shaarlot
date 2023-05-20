use crate::domain::entities::Bookmark;
use crate::domain::repositories::BookmarkRepository;
use anyhow::Context;
use std::sync::Arc;

#[derive(Debug)]
pub struct FindBookmarkCommand {
    pub user_id: Option<i32>,
    pub bookmark_id: i32,
}

#[derive(Clone)]
pub struct FindBookmarkUseCase {
    repository: Arc<dyn BookmarkRepository>,
}

impl FindBookmarkUseCase {
    pub fn new(repository: Arc<dyn BookmarkRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: FindBookmarkCommand) -> anyhow::Result<Option<Bookmark>> {
        self.repository
            .find_by_id(command.user_id, command.bookmark_id)
            .await
            .context("Could not find bookmark")
    }
}
