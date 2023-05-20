use crate::application::create_bookmark::CreateBookmarkUseCase;
use crate::application::delete_bookmark::DeleteBookmarkUseCase;
use crate::application::find_bookmark::FindBookmarkUseCase;
use crate::application::get_bookmark_stats::GetBookmarksStatsUseCase;
use crate::application::search_bookmarks::SearchBookmarkUseCase;
use crate::application::update_bookmark::UpdateBookmarkUseCase;
use crate::infrastructure::mailer::Mailer;
use reqwest::Client;
use sea_orm::DatabaseConnection;

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod url;

#[derive(Clone)]
pub struct AppState {
    pub database: DatabaseConnection,
    pub mailer: Mailer,
    pub ignored_query_params: Vec<&'static str>,
    pub http_client: Client,
    pub demo: bool,
    pub create_bookmark_usecase: CreateBookmarkUseCase,
    pub search_bookmarks_usecase: SearchBookmarkUseCase,
    pub find_bookmark_usecase: FindBookmarkUseCase,
    pub update_bookmark_usecase: UpdateBookmarkUseCase,
    pub delete_bookmark_usecase: DeleteBookmarkUseCase,
    pub get_bookmarks_stats: GetBookmarksStatsUseCase,
}
