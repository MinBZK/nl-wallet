use async_trait::async_trait;
use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WalletUser::Table)
                    .col(pk_uuid(WalletUser::Id))
                    .col(string_uniq(WalletUser::WalletId))
                    .col(binary(WalletUser::HwPubkeyDer))
                    .col(binary(WalletUser::EncryptedPinPubkeySec1))
                    .col(binary(WalletUser::PinPubkeyIv))
                    .col(binary_null(WalletUser::EncryptedPreviousPinPubkeySec1))
                    .col(binary_null(WalletUser::PreviousPinPubkeyIv))
                    .col(unsigned(WalletUser::InstructionSequenceNumber).default(0))
                    .col(small_unsigned(WalletUser::PinEntries).default(0))
                    .col(timestamp_with_time_zone_null(WalletUser::LastUnsuccessfulPin))
                    .col(boolean(WalletUser::IsBlocked).default(false))
                    .col(boolean(WalletUser::HasWte).default(false))
                    .check(SimpleExpr::or(
                        // Both of these columns should be used or neither.
                        Expr::col(WalletUser::EncryptedPreviousPinPubkeySec1)
                            .is_null()
                            .and(Expr::col(WalletUser::PreviousPinPubkeyIv).is_null()),
                        Expr::col(WalletUser::EncryptedPreviousPinPubkeySec1)
                            .is_not_null()
                            .and(Expr::col(WalletUser::PreviousPinPubkeyIv).is_not_null()),
                    ))
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
