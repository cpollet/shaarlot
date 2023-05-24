use crate::application::delete_bookmark::DeleteResult::{Deleted, Forbidden};
use crate::domain::repositories::BookmarkRepository;
use anyhow::Context;
use std::sync::Arc;

#[derive(Debug)]
pub struct DeleteBookmarkCommand {
    pub bookmark_id: i32,
    pub user_id: i32,
}

pub enum DeleteResult {
    Deleted,
    Forbidden,
}

#[derive(Clone)]
pub struct DeleteBookmarkUseCase {
    repository: Arc<dyn BookmarkRepository>,
}

impl DeleteBookmarkUseCase {
    pub fn new(repository: Arc<dyn BookmarkRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        command: DeleteBookmarkCommand,
    ) -> anyhow::Result<Option<DeleteResult>> {
        // todo Result<(), DeleteBookmarkError>
        let bookmark = self
            .repository
            .find_by_id(Some(command.user_id), command.bookmark_id)
            .await
            .context("Could not retrieve bookmark")?
            .context("Could not retrieve bookmark: not found")?;

        if !bookmark.is_owner(command.user_id) {
            return Ok(Some(Forbidden));
        }

        self.repository
            .delete(command.bookmark_id)
            .await
            .context("Could not delete bookmark")?;

        Ok(Some(Deleted))
    }
}
