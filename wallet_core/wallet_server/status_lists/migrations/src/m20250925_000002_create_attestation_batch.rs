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
                    .table(AttestationBatch::Table)
                    .if_not_exists()
                    .col(big_integer(AttestationBatch::Id).primary_key().auto_increment())
                    .col(uuid(AttestationBatch::BatchId))
                    .col(date_null(AttestationBatch::ExpirationDate))
                    .col(boolean(AttestationBatch::IsRevoked))
                    .col(json_binary(AttestationBatch::StatusListLocations))
                    .index(
                        Index::create()
                            .unique()
                            .name("attestation_batch_unique_batch_id")
                            .col(AttestationBatch::BatchId),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(AttestationBatch::Table)
                    .name("attestation_batch_expiration_date")
                    .col(AttestationBatch::ExpirationDate)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum AttestationBatch {
    Table,
    Id,
    BatchId,
    ExpirationDate,
    IsRevoked,
    StatusListLocations,
}
