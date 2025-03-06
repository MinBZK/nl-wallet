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
                    .table(IssuanceHistoryEvent::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(IssuanceHistoryEvent::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(IssuanceHistoryEvent::Timestamp).timestamp().not_null())
                    .col(ColumnDef::new(IssuanceHistoryEvent::Attestations).json().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(DisclosureHistoryEvent::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DisclosureHistoryEvent::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DisclosureHistoryEvent::Timestamp).timestamp().not_null())
                    .col(
                        ColumnDef::new(DisclosureHistoryEvent::RelyingPartyCertificate)
                            .binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DisclosureHistoryEvent::Status).text().not_null())
                    .col(ColumnDef::new(DisclosureHistoryEvent::Attestations).json().null())
                    .col(ColumnDef::new(DisclosureHistoryEvent::Type).text().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(HistoryAttestationType::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(HistoryAttestationType::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(HistoryAttestationType::AttestationType)
                            .text()
                            .unique_key()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(IssuanceHistoryEventAttestationType::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(IssuanceHistoryEventAttestationType::IssuanceHistoryEventId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IssuanceHistoryEventAttestationType::HistoryAttestationTypeId)
                            .uuid()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(IssuanceHistoryEventAttestationType::IssuanceHistoryEventId)
                            .col(IssuanceHistoryEventAttestationType::HistoryAttestationTypeId),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(DisclosureHistoryEventAttestationType::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DisclosureHistoryEventAttestationType::DisclosureHistoryEventId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DisclosureHistoryEventAttestationType::HistoryAttestationTypeId)
                            .uuid()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(DisclosureHistoryEventAttestationType::DisclosureHistoryEventId)
                            .col(DisclosureHistoryEventAttestationType::HistoryAttestationTypeId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(DisclosureHistoryEventAttestationType::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(IssuanceHistoryEventAttestationType::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(HistoryAttestationType::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(DisclosureHistoryEvent::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(IssuanceHistoryEvent::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum IssuanceHistoryEvent {
    Table,
    Id,
    Timestamp,
    Attestations,
}

#[derive(DeriveIden)]
enum DisclosureHistoryEvent {
    Table,
    Id,
    Timestamp,
    RelyingPartyCertificate,
    Status,
    Attestations,
    Type,
}

#[derive(DeriveIden)]
enum HistoryAttestationType {
    Table,
    Id,
    AttestationType,
}

#[derive(DeriveIden)]
enum IssuanceHistoryEventAttestationType {
    Table,
    IssuanceHistoryEventId,
    HistoryAttestationTypeId,
}

#[derive(DeriveIden)]
enum DisclosureHistoryEventAttestationType {
    Table,
    DisclosureHistoryEventId,
    HistoryAttestationTypeId,
}
