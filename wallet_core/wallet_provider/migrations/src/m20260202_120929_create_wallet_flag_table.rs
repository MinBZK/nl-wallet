use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WalletFlag::Table)
                    .if_not_exists()
                    .col(text(WalletFlag::Name).primary_key())
                    .col(boolean(WalletFlag::Value))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum WalletFlag {
    Table,
    Name,
    Value,
}
