use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(EventLog::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(EventLog::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(EventLog::Type).text().not_null())
                    .col(ColumnDef::new(EventLog::Timestamp).timestamp().not_null())
                    .col(ColumnDef::new(EventLog::RemotePartyCertificate).binary().not_null())
                    .col(ColumnDef::new(EventLog::Status).text().not_null())
                    .col(ColumnDef::new(EventLog::StatusDescription).text().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(EventLog::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum EventLog {
    Table,
    Id,
    Type,
    Timestamp,
    RemotePartyCertificate,
    Status,
    StatusDescription,
}
