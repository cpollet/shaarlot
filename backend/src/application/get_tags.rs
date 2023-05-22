use crate::domain::repositories::BookmarkRepository;
use crate::domain::values::tag::{CountedTag, Sort};
use anyhow::Context;
use std::sync::Arc;

#[derive(Debug)]
pub struct GetTagsCommand {
    pub user_id: Option<i32>,
    pub sort: Sort,
}

#[derive(Clone)]
pub struct GetTagsUseCase {
    repository: Arc<dyn BookmarkRepository>,
}

impl GetTagsUseCase {
    pub fn new(repository: Arc<dyn BookmarkRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: GetTagsCommand) -> anyhow::Result<Vec<CountedTag>> {
        self.repository
            .find_tags(command.user_id, command.sort)
            .await
            .context("Could not find tags")
    }
}
