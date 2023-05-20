use crate::domain::entities::{Bookmarks, Filter, Pagination, Sort};
use crate::domain::repositories::BookmarkRepository;
use std::sync::Arc;

#[derive(Debug)]
pub struct SearchBookmarkCommand {
    pub user_id: Option<i32>,
    pub tags: Vec<String>,
    pub search: Vec<String>,
    pub filter: Filter,
    pub pagination: Pagination,
    pub sort: Sort,
}

pub struct SearchResult {
    pub bookmarks: Bookmarks,
    pub total_count: u64,
}

#[derive(Clone)]
pub struct SearchBookmarkUseCase {
    repository: Arc<dyn BookmarkRepository>,
}

impl SearchBookmarkUseCase {
    pub fn new(repository: Arc<dyn BookmarkRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: SearchBookmarkCommand) -> anyhow::Result<SearchResult> {
        self.repository
            .find(
                command.user_id,
                command.tags,
                command.search,
                command.filter,
                command.pagination,
                command.sort,
            )
            .await
            .map(|r| SearchResult {
                bookmarks: r.0,
                total_count: r.1,
            })
    }
}
