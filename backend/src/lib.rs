use crate::application::create_account::CreateAccountUseCase;
use crate::application::create_bookmark::CreateBookmarkUseCase;
use crate::application::create_password_recovery::CreatePasswordRecoveryUseCase;
use crate::application::delete_bookmark::DeleteBookmarkUseCase;
use crate::application::find_bookmark::FindBookmarkUseCase;
use crate::application::get_bookmark_stats::GetBookmarksStatsUseCase;
use crate::application::get_tags::GetTagsUseCase;
use crate::application::get_url_details::GetUrlDetailsUseCase;
use crate::application::perform_password_recovery::PerformPasswordRecoveryUseCase;
use crate::application::search_bookmarks::SearchBookmarkUseCase;
use crate::application::update_bookmark::UpdateBookmarkUseCase;
use crate::application::validate_email::ValidateEmailUseCase;
use crate::domain::repositories::AccountRepository;
use crate::infrastructure::mailer::Mailer;
use reqwest::Client;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod url;

#[derive(Clone)]
pub struct AppState {
    // todo remove
    pub database: DatabaseConnection,
    // todo remove
    pub mailer: Arc<Mailer>,
    pub account_repository: Arc<dyn AccountRepository>,
    pub ignored_query_params: Vec<&'static str>,
    // todo remove
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
    pub get_tags: GetTagsUseCase,
    pub get_url_details: GetUrlDetailsUseCase,
    pub create_account: CreateAccountUseCase,
}
