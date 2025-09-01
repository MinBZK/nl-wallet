use async_trait::async_trait;
use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

use crate::m20250102_000000_create_wallet_user_apple_attestation_table::WalletUserAppleAttestation;
use crate::m20250102_000001_create_wallet_user_android_attestation_table::WalletUserAndroidAttestation;

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
                    .col(timestamp_with_time_zone(WalletUser::AttestationDateTime))
                    .col(uuid_null(WalletUser::AppleAttestationId))
                    .col(uuid_null(WalletUser::AndroidAttestationId))
                    .col(string_null(WalletUser::RecoveryCode))
                    .check(SimpleExpr::or(
                        // Both of these columns should be used or neither.
                        Expr::col(WalletUser::EncryptedPreviousPinPubkeySec1)
                            .is_null()
                            .and(Expr::col(WalletUser::PreviousPinPubkeyIv).is_null()),
                        Expr::col(WalletUser::EncryptedPreviousPinPubkeySec1)
                            .is_not_null()
                            .and(Expr::col(WalletUser::PreviousPinPubkeyIv).is_not_null()),
                    ))
                    .check(SimpleExpr::or(
                        // One and only one of these foreign key columns should be used.
                        Expr::col(WalletUser::AppleAttestationId)
                            .is_not_null()
                            .and(Expr::col(WalletUser::AndroidAttestationId).is_null()),
                        Expr::col(WalletUser::AppleAttestationId)
                            .is_null()
                            .and(Expr::col(WalletUser::AndroidAttestationId).is_not_null()),
                    ))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_wallet_user_apple_attestation_id")
                            .from(WalletUser::Table, WalletUser::AppleAttestationId)
                            .to(WalletUserAppleAttestation::Table, WalletUserAppleAttestation::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_wallet_user_android_attestation_id")
                            .from(WalletUser::Table, WalletUser::AndroidAttestationId)
                            .to(WalletUserAndroidAttestation::Table, WalletUserAndroidAttestation::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("wallet_user_unique_apple_attestation_id")
                            .col(WalletUser::AppleAttestationId),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("wallet_user_unique_android_attestation_id")
                            .col(WalletUser::AndroidAttestationId),
                    )
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
    AttestationDateTime,
    AppleAttestationId,
    AndroidAttestationId,
    RecoveryCode,
}
