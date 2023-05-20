use crate::application::update_bookmark::UpdateResult::{Forbidden, Updated};
use crate::domain::entities::Bookmark;
use crate::domain::repositories::BookmarkRepository;
use anyhow::Context;
use std::sync::Arc;

pub struct UpdateBookmarkCommand {
    pub id: i32,
    pub user_id: i32,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub private: bool,
    pub pinned: bool,
}

pub enum UpdateResult {
    Updated(Bookmark),
    Forbidden,
}

#[derive(Clone)]
pub struct UpdateBookmarkUseCase {
    repository: Arc<dyn BookmarkRepository>,
}

impl UpdateBookmarkUseCase {
    pub fn new(repository: Arc<dyn BookmarkRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        command: UpdateBookmarkCommand,
    ) -> anyhow::Result<Option<UpdateResult>> {
        let bookmark = self
            .repository
            .find_by_id(Some(command.user_id), command.id)
            .await
            .context("Could not retrieve bookmark")?
            .context("Could not retrieve bookmark: not found")?;

        if !bookmark.is_owner(command.user_id) {
            return Ok(Some(Forbidden));
        }

        let bookmark = {
            let mut bookmark = bookmark;
            bookmark.url = command.url;
            bookmark.title = command.title;
            bookmark.description = command.description;
            bookmark.tags = command.tags;
            bookmark.private = command.private;
            bookmark.pinned = command.pinned;
            bookmark
        };

        Ok(Some(Updated(self.repository.save(bookmark).await?)))
    }
}
