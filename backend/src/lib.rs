use crate::application::create_bookmark::CreateBookmarkUseCase;
use crate::application::create_password_recovery::CreatePasswordRecoveryUseCase;
use crate::application::delete_bookmark::DeleteBookmarkUseCase;
use crate::application::find_bookmark::FindBookmarkUseCase;
use crate::application::get_bookmark_stats::GetBookmarksStatsUseCase;
use crate::application::perform_password_recovery::PerformPasswordRecoveryUseCase;
use crate::application::search_bookmarks::SearchBookmarkUseCase;
use crate::application::update_bookmark::UpdateBookmarkUseCase;
use crate::application::validate_email::ValidateEmailUseCase;
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
    // todo remove
    pub mailer: Mailer,
    pub ignored_query_params: Vec<&'static str>,
    pub http_client: Client,
    pub demo: bool,
    pub create_bookmark: CreateBookmarkUseCase,
    pub search_bookmarks: SearchBookmarkUseCase,
    pub find_bookmark: FindBookmarkUseCase,
    pub update_bookmark: UpdateBookmarkUseCase,
    pub delete_bookmark: DeleteBookmarkUseCase,
    pub get_bookmarks_stats: GetBookmarksStatsUseCase,
    pub validate_email: ValidateEmailUseCase,
    pub create_password_recovery: CreatePasswordRecoveryUseCase,
    pub perform_password_recovery: PerformPasswordRecoveryUseCase,
}
