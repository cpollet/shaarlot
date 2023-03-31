use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tag::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Tag::Name)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(BookmarkTag::Table)
                    .if_not_exists()
                    .primary_key(
                        Index::create()
                            .table(BookmarkTag::Table)
                            .col(BookmarkTag::TagId)
                            .col(BookmarkTag::BookmarkId),
                    )
                    .col(ColumnDef::new(BookmarkTag::BookmarkId).integer().not_null())
                    .col(ColumnDef::new(BookmarkTag::TagId).integer().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .from(BookmarkTag::Table, BookmarkTag::TagId)
                    .to(Tag::Table, Tag::Id)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .from(BookmarkTag::Table, BookmarkTag::BookmarkId)
                    .to(Bookmark::Table, Bookmark::Id)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BookmarkTag::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tag::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Bookmark {
    Table,
    Id,
}

#[derive(Iden)]
enum Tag {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
enum BookmarkTag {
    Table,
    BookmarkId,
    TagId,
}
