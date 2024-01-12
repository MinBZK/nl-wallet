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
                    .table(HistoryEvent::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(HistoryEvent::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(HistoryEvent::Type).text().not_null())
                    .col(ColumnDef::new(HistoryEvent::Timestamp).timestamp().not_null())
                    .col(ColumnDef::new(HistoryEvent::RemotePartyCertificate).binary().not_null())
                    .col(ColumnDef::new(HistoryEvent::Status).text().not_null())
                    .col(ColumnDef::new(HistoryEvent::StatusDescription).text().null())
                    .col(ColumnDef::new(HistoryEvent::Attributes).binary().null())
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
                    .table(HistoryEventDocType::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(HistoryEventDocType::HistoryEventId).uuid().not_null())
                    .col(ColumnDef::new(HistoryEventDocType::HistoryDocTypeId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(HistoryEventDocType::HistoryEventId)
                            .col(HistoryEventDocType::HistoryDocTypeId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HistoryEventDocType::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(HistoryDocType::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(HistoryEvent::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum HistoryEvent {
    Table,
    Id,
    Type,
    Timestamp,
    RemotePartyCertificate,
    Status,
    StatusDescription,
    Attributes,
}

#[derive(DeriveIden)]
enum HistoryDocType {
    Table,
    Id,
    DocType,
}

#[derive(DeriveIden)]
enum HistoryEventDocType {
    Table,
    HistoryEventId,
    HistoryDocTypeId,
}
