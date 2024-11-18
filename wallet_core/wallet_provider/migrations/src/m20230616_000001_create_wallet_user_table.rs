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
                    .col(ColumnDef::new(WalletUser::WalletId).string().not_null().unique_key())
                    .col(ColumnDef::new(WalletUser::HwPubkeyDer).binary().not_null())
                    .col(ColumnDef::new(WalletUser::EncryptedPinPubkeySec1).binary().not_null())
                    .col(ColumnDef::new(WalletUser::PinPubkeyIv).binary().not_null())
                    .col(
                        ColumnDef::new(WalletUser::EncryptedPreviousPinPubkeySec1)
                            .binary()
                            .null(),
                    )
                    .col(ColumnDef::new(WalletUser::PreviousPinPubkeyIv).binary().null())
                    .col(
                        ColumnDef::new(WalletUser::InstructionSequenceNumber)
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(WalletUser::PinEntries)
                            .small_unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(WalletUser::LastUnsuccessfulPin)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(WalletUser::IsBlocked)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(WalletUser::HasWte).boolean().not_null().default(false))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum WalletUser {
    Table,
    Id,
    WalletId,
    HwPubkeyDer,
    EncryptedPinPubkeySec1,
    PinPubkeyIv,
    EncryptedPreviousPinPubkeySec1,
    PreviousPinPubkeyIv,
    InstructionSequenceNumber,
    PinEntries,
    LastUnsuccessfulPin,
    IsBlocked,
    HasWte,
}
