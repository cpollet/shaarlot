use crate::domain::values::tag::Tag;
use chrono::{DateTime, Utc};

pub struct Bookmark {
    // todo remove pub
    pub id: Option<i32>,
    pub user_id: i32,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<Tag>,
    pub creation_date: DateTime<Utc>,
    pub update_date: Option<DateTime<Utc>>,
    pub private: bool,
    pub pinned: bool,
}

impl Bookmark {
    pub fn is_owner(&self, user_id: i32) -> bool {
        self.user_id == user_id
    }
}

pub type Bookmarks = Vec<Bookmark>;

#[derive(Clone, Copy, Debug, Default)]
pub enum Filter {
    #[default]
    All,
    Private,
    Public,
}

#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: u64,
    pub size: u64,
}

#[derive(Clone, Copy, Debug)]
pub enum Sort {
    CreationDateDesc,
    CreationDateAsc,
}
