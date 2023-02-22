pub use sea_orm_migration::prelude::*;

mod m20230212_000001_create_table;
mod m20230213_195707_add_title_to_bookmarks;
mod m20230219_171914_add_dates_to_bookmarks;
mod m20230221_181655_create_table_users;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230212_000001_create_table::Migration),
            Box::new(m20230213_195707_add_title_to_bookmarks::Migration),
            Box::new(m20230219_171914_add_dates_to_bookmarks::Migration),
            Box::new(m20230221_181655_create_table_users::Migration),
        ]
    }
}
