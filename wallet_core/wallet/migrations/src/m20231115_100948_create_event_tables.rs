use async_trait::async_trait;
use sea_orm_migration::prelude::*;

use crate::m20230922_095234_create_attestation_tables::Attestation;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DisclosureEvent::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(DisclosureEvent::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(DisclosureEvent::Timestamp).timestamp().not_null())
                    .col(
                        ColumnDef::new(DisclosureEvent::RelyingPartyCertificate)
                            .binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DisclosureEvent::Status).text().not_null())
                    .col(ColumnDef::new(DisclosureEvent::Type).text().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(IssuanceEvent::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(IssuanceEvent::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(IssuanceEvent::Timestamp).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(IssuanceEventAttestation::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(IssuanceEventAttestation::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(IssuanceEventAttestation::IssuanceEventId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(IssuanceEventAttestation::AttestationId).uuid())
                    .col(
                        ColumnDef::new(IssuanceEventAttestation::AttestationPresentation)
                            .json()
                            .not_null(),
                    )
                    .col(ColumnDef::new(IssuanceEventAttestation::Renewed).boolean().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                IssuanceEventAttestation::Table,
                                IssuanceEventAttestation::IssuanceEventId,
                            )
                            .to(IssuanceEvent::Table, IssuanceEvent::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(IssuanceEventAttestation::Table, IssuanceEventAttestation::AttestationId)
                            .to(Attestation::Table, Attestation::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(DisclosureEventAttestation::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DisclosureEventAttestation::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DisclosureEventAttestation::DisclosureEventId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DisclosureEventAttestation::AttestationId).uuid())
                    .col(
                        ColumnDef::new(DisclosureEventAttestation::AttestationPresentation)
                            .json()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                DisclosureEventAttestation::Table,
                                DisclosureEventAttestation::DisclosureEventId,
                            )
                            .to(DisclosureEvent::Table, DisclosureEvent::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                DisclosureEventAttestation::Table,
                                DisclosureEventAttestation::AttestationId,
                            )
                            .to(Attestation::Table, Attestation::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DisclosureEventAttestation::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(IssuanceEventAttestation::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(DisclosureEvent::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(IssuanceEvent::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum IssuanceEvent {
    Table,
    Id,
    Timestamp,
}

#[derive(DeriveIden)]
enum DisclosureEvent {
    Table,
    Id,
    Timestamp,
    RelyingPartyCertificate,
    Status,
    Type,
}

#[derive(DeriveIden)]
enum IssuanceEventAttestation {
    Table,
    Id,
    IssuanceEventId,
    AttestationId,
    AttestationPresentation,
    Renewed,
}

#[derive(DeriveIden)]
enum DisclosureEventAttestation {
    Table,
    Id,
    DisclosureEventId,
    AttestationId,
    AttestationPresentation,
}
