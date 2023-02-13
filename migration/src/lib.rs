pub use sea_orm_migration::prelude::*;

mod m20230212_000001_create_table;
mod m20230213_195707_add_title_to_bookmarks;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230212_000001_create_table::Migration),
            Box::new(m20230213_195707_add_title_to_bookmarks::Migration),
        ]
    }
}
