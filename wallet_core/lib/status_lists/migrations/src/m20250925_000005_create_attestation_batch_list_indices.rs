use async_trait::async_trait;
use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

use crate::m20250925_000002_create_attestation_batch::AttestationBatch;
use crate::m20250925_000003_create_status_list::StatusList;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AttestationBatchListIndices::Table)
                    .if_not_exists()
                    .col(big_integer(AttestationBatchListIndices::AttestationBatchId))
                    .col(big_integer(AttestationBatchListIndices::StatusListId))
                    .col(array(AttestationBatchListIndices::Indices, ColumnType::Integer))
                    .primary_key(
                        Index::create()
                            .col(AttestationBatchListIndices::AttestationBatchId)
                            .col(AttestationBatchListIndices::StatusListId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_attestation_batch_list_indices_attestation_batch")
                            .from(
                                AttestationBatchListIndices::Table,
                                AttestationBatchListIndices::AttestationBatchId,
                            )
                            .to(AttestationBatch::Table, AttestationBatch::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_attestation_batch_list_indices_status_list")
                            .from(
                                AttestationBatchListIndices::Table,
                                AttestationBatchListIndices::StatusListId,
                            )
                            .to(StatusList::Table, StatusList::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum AttestationBatchListIndices {
    Table,
    AttestationBatchId,
    StatusListId,
    Indices,
}
