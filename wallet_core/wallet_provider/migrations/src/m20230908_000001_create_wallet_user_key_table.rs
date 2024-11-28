use async_trait::async_trait;
use sea_orm_migration::prelude::*;

use crate::m20230616_000001_create_wallet_user_table::WalletUser;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WalletUserKey::Table)
                    .col(ColumnDef::new(WalletUserKey::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(WalletUserKey::WalletUserId).uuid().not_null())
                    .col(ColumnDef::new(WalletUserKey::Identifier).string().not_null())
                    .col(ColumnDef::new(WalletUserKey::EncryptedPrivateKey).binary().not_null())
                    .col(ColumnDef::new(WalletUserKey::PublicKey).binary().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_wallet_user_id")
                            .from(WalletUserKey::Table, WalletUserKey::WalletUserId)
                            .to(WalletUser::Table, WalletUser::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("wallet_user_key_unique_identifier_wallet_user_id")
                            .col(WalletUserKey::Identifier)
                            .col(WalletUserKey::WalletUserId),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum WalletUserKey {
    Table,
    Id,
    WalletUserId,
    Identifier,
    EncryptedPrivateKey,
    PublicKey,
}
