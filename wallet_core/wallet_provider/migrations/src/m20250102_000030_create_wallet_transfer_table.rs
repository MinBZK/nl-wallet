use async_trait::async_trait;
use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

use crate::m20250102_000010_create_wallet_user_table::WalletUser;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WalletTransfer::Table)
                    .col(pk_uuid(WalletTransfer::Id))
                    .col(uuid(WalletTransfer::DestinationWalletUserId))
                    .col(uuid(WalletTransfer::TransferSessionId))
                    .col(string(WalletTransfer::DestinationWalletAppVersion))
                    .col(string(WalletTransfer::State).default("created"))
                    .col(timestamp_with_time_zone(WalletTransfer::Created))
                    .col(binary_null(WalletTransfer::EncryptedWalletData))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_wallet_transfer_destination_wallet_user_id")
                            .from(WalletTransfer::Table, WalletTransfer::DestinationWalletUserId)
                            .to(WalletUser::Table, WalletUser::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("wallet_transfer_unique_destination_wallet_id")
                            .col(WalletTransfer::DestinationWalletUserId),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("wallet_transfer_unique_transfer_session_id")
                            .col(WalletTransfer::TransferSessionId),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum WalletTransfer {
    Table,
    Id,
    DestinationWalletUserId,
    DestinationWalletAppVersion,
    TransferSessionId,
    State,
    Created,
    EncryptedWalletData,
}
