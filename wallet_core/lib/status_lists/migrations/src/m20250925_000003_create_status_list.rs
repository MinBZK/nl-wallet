use async_trait::async_trait;
use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

use crate::m20250925_000001_create_attestation_type::AttestationType;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StatusList::Table)
                    .if_not_exists()
                    .col(big_integer(StatusList::Id).primary_key().auto_increment())
                    .col(small_integer(StatusList::AttestationTypeId))
                    .col(string(StatusList::ExternalId))
                    .col(integer(StatusList::Available))
                    .col(integer(StatusList::Size))
                    .col(big_integer(StatusList::NextSequenceNo))
                    .index(
                        Index::create()
                            .unique()
                            .name("status_list_unique_external_id")
                            .col(StatusList::ExternalId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_status_list_attestation_type")
                            .from(StatusList::Table, StatusList::AttestationTypeId)
                            .to(AttestationType::Table, AttestationType::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum StatusList {
    Table,
    Id,
    AttestationTypeId,
    ExternalId,
    Available,
    Size,
    NextSequenceNo,
}
