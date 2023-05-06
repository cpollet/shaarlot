use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Pin::Table)
                    .if_not_exists()
                    .primary_key(
                        Index::create()
                            .table(Pin::Table)
                            .col(Pin::UserId)
                            .col(Pin::BookmarkId),
                    )
                    .col(ColumnDef::new(Pin::BookmarkId).integer().not_null())
                    .col(ColumnDef::new(Pin::UserId).integer().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .from(Pin::Table, Pin::UserId)
                    .to(Account::Table, Account::Id)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .from(Pin::Table, Pin::BookmarkId)
                    .to(Bookmark::Table, Bookmark::Id)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Pin::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Bookmark {
    Table,
    Id,
}

#[derive(Iden)]
enum Account {
    Table,
    Id,
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Pin {
    Table,
    UserId,
    BookmarkId,
}
