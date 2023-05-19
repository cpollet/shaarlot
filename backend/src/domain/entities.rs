use chrono::{DateTime, Utc};

pub struct Bookmark {
    pub id: i32,
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

pub type Tag = String;
