use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PasswordRecovery::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PasswordRecovery::Id)
                            .string_len(36)
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PasswordRecovery::Token)
                            .string_len(97)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PasswordRecovery::GenerationDate)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()".to_string()),
                    )
                    .col(
                        ColumnDef::new(PasswordRecovery::UserId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PasswordRecovery::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum PasswordRecovery {
    Table,
    Id,
    Token,
    GenerationDate,
    UserId,
}
