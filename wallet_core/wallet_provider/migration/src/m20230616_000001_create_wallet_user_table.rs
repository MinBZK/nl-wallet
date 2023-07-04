use async_trait::async_trait;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WalletUser::Table)
                    .col(ColumnDef::new(WalletUser::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(WalletUser::WalletId).string().not_null())
                    .col(ColumnDef::new(WalletUser::HwPubkey).string().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum WalletUser {
    Table,
    Id,
    WalletId,
    HwPubkey,
}
