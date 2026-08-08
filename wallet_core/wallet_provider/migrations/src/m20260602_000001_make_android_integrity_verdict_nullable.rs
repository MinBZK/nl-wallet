use async_trait::async_trait;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Devices without Google Play services (e.g. GrapheneOS, LineageOS, /e/OS) or without a
        // Google account cannot produce a Play Integrity verdict and register using hardware key
        // attestation alone. For those wallet users no integrity verdict is stored, so the column
        // must allow NULL values.
        manager
            .alter_table(
                Table::alter()
                    .table(WalletUserAndroidAttestation::Table)
                    .modify_column(
                        ColumnDef::new(WalletUserAndroidAttestation::IntegrityVerdictJson)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum WalletUserAndroidAttestation {
    Table,
    IntegrityVerdictJson,
}
