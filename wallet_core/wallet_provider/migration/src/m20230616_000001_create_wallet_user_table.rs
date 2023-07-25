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
                    .col(ColumnDef::new(WalletUser::PinPubkeyDer).binary().not_null())
                    .col(
                        ColumnDef::new(WalletUser::InstructionSequenceNumber)
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(WalletUser::InstructionChallenge).binary().null())
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
    HwPubkeyDer,
    PinPubkeyDer,
    InstructionSequenceNumber,
    InstructionChallenge,
    PinEntries,
    LastUnsuccessfulPin,
    IsBlocked,
}
