pub use sea_orm_migration::prelude::*;

mod m20230212_000001_create_table;
mod m20230213_195707_add_title_to_bookmarks;
mod m20230219_171914_add_dates_to_bookmarks;
mod m20230221_181655_create_table_users;
mod m20230227_211829_add_email_to_user;
mod m20230306_194756_add_email_token_to_user;
mod m20230311_105845_add_new_email_to_user;
mod m20230312_124742_create_table_password_recovery;
mod m20230314_214602_rename_user_to_account;
mod m20230314_225232_add_user_id_to_bookmark;
mod m20230331_194725_create_tags_tables;
mod m20230414_180807_add_private_to_bookmark;
mod m20230506_102057_add_pin_table;
mod m20230521_114151_add_fk_from_password_recovery_to_account;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230212_000001_create_table::Migration),
            Box::new(m20230213_195707_add_title_to_bookmarks::Migration),
            Box::new(m20230219_171914_add_dates_to_bookmarks::Migration),
            Box::new(m20230221_181655_create_table_users::Migration),
            Box::new(m20230227_211829_add_email_to_user::Migration),
            Box::new(m20230306_194756_add_email_token_to_user::Migration),
            Box::new(m20230311_105845_add_new_email_to_user::Migration),
            Box::new(m20230312_124742_create_table_password_recovery::Migration),
            Box::new(m20230314_214602_rename_user_to_account::Migration),
            Box::new(m20230314_225232_add_user_id_to_bookmark::Migration),
            Box::new(m20230331_194725_create_tags_tables::Migration),
            Box::new(m20230414_180807_add_private_to_bookmark::Migration),
            Box::new(m20230506_102057_add_pin_table::Migration),
            Box::new(m20230521_114151_add_fk_from_password_recovery_to_account::Migration),
        ]
    }
}
