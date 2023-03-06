use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column_if_not_exists(ColumnDef::new(User::EmailToken).string_len(36))
                    .add_column_if_not_exists(
                        ColumnDef::new(User::EmailTokenGenerationDate).timestamp_with_time_zone(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(User::Table)
                    .col(User::EmailToken)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::EmailToken)
                    .drop_column(User::EmailTokenGenerationDate)
                    .to_owned(),
            )
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum User {
    Table,
    EmailToken,
    EmailTokenGenerationDate,
}
