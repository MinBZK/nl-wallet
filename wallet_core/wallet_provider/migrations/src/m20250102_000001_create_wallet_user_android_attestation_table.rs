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
                    // Unfortunately `sea-orm` does not support an entity with a `Vec<Vec<u8>>` field if we use an array
                    // of binaries here, even though the cli will happily generate it. As a workaround we resort to
                    // using an array of Base64 encoded string.
                    .col(array(
                        WalletUserAndroidAttestation::CertificateChain,
                        ColumnType::String(StringLen::default()),
                    ))
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
