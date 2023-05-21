use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .from(PasswordRecovery::Table, PasswordRecovery::UserId)
                    .to(Account::Table, Account::Id)
                    .name("fk$password_recovery$user_id")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(PasswordRecovery::Table)
                    .name("fk$password_recovery$user_id")
                    .to_owned(),
            )
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum PasswordRecovery {
    Table,
    UserId,
}

#[derive(Iden)]
enum Account {
    Table,
    Id,
}
