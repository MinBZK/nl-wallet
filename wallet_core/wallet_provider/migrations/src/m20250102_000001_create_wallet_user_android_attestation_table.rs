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
                    .table(WalletUserAndroidAttestation::Table)
                    .col(pk_uuid(WalletUserAndroidAttestation::Id))
                    .col(array(WalletUserAndroidAttestation::CertificateChain, ColumnType::Blob))
                    // This data is stored as a string, so that the original JSON response
                    // body as returned by the Google Play Integrity API can be preserved.
                    .col(string(WalletUserAndroidAttestation::IntegrityVerdictJson))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum WalletUserAndroidAttestation {
    Table,
    Id,
    CertificateChain,
    IntegrityVerdictJson,
}
