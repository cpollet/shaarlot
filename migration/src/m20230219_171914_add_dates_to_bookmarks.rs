use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Bookmark::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Bookmark::CreationDate)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()".to_string()),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Bookmark::UpdateDate).timestamp_with_time_zone(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Bookmark::Table)
                    .drop_column(Bookmark::CreationDate)
                    .drop_column(Bookmark::UpdateDate)
                    .to_owned(),
            )
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Bookmark {
    Table,
    CreationDate,
    UpdateDate,
}
