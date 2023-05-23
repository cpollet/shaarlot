use crate::domain::entities::bookmark::Bookmark;
use crate::domain::repositories::BookmarkRepository;
use chrono::Utc;
use std::sync::Arc;

pub struct CreateBookmarkCommand {
    pub user_id: i32,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub private: bool,
}

#[derive(Clone)]
pub struct CreateBookmarkUseCase {
    repository: Arc<dyn BookmarkRepository>,
}

impl CreateBookmarkUseCase {
    pub fn new(repository: Arc<dyn BookmarkRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: CreateBookmarkCommand) -> anyhow::Result<Bookmark> {
        self.repository
            .save(Bookmark {
                id: None,
                user_id: command.user_id,
                url: command.url,
                title: command.title,
                description: command.description,
                tags: command.tags,
                creation_date: Utc::now(),
                update_date: None,
                private: command.private,
                pinned: false,
            })
            .await
    }
}
