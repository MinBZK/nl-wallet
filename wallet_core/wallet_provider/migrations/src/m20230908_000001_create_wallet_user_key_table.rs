use async_trait::async_trait;
use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

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
                    .col(pk_uuid(WalletUserKey::Id))
                    .col(uuid(WalletUserKey::WalletUserId))
                    .col(string(WalletUserKey::Identifier))
                    .col(binary(WalletUserKey::EncryptedPrivateKey))
                    .col(binary(WalletUserKey::PublicKey))
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
