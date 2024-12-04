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
                    .col(ColumnDef::new(IssuanceHistoryEvent::Attributes).json().not_null())
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
                    .col(ColumnDef::new(DisclosureHistoryEvent::Attributes).json().null())
                    .col(ColumnDef::new(DisclosureHistoryEvent::Type).text().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(HistoryDocType::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(HistoryDocType::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(HistoryDocType::DocType).text().unique_key().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(IssuanceHistoryEventDocType::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(IssuanceHistoryEventDocType::IssuanceHistoryEventId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IssuanceHistoryEventDocType::HistoryDocTypeId)
                            .uuid()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(IssuanceHistoryEventDocType::IssuanceHistoryEventId)
                            .col(IssuanceHistoryEventDocType::HistoryDocTypeId),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(DisclosureHistoryEventDocType::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DisclosureHistoryEventDocType::DisclosureHistoryEventId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DisclosureHistoryEventDocType::HistoryDocTypeId)
                            .uuid()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(DisclosureHistoryEventDocType::DisclosureHistoryEventId)
                            .col(DisclosureHistoryEventDocType::HistoryDocTypeId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DisclosureHistoryEventDocType::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(IssuanceHistoryEventDocType::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(HistoryDocType::Table).to_owned())
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
    Attributes,
}

#[derive(DeriveIden)]
enum DisclosureHistoryEvent {
    Table,
    Id,
    Timestamp,
    RelyingPartyCertificate,
    Status,
    Attributes,
    Type,
}

#[derive(DeriveIden)]
enum HistoryDocType {
    Table,
    Id,
    DocType,
}

#[derive(DeriveIden)]
enum IssuanceHistoryEventDocType {
    Table,
    IssuanceHistoryEventId,
    HistoryDocTypeId,
}

#[derive(DeriveIden)]
enum DisclosureHistoryEventDocType {
    Table,
    DisclosureHistoryEventId,
    HistoryDocTypeId,
}
