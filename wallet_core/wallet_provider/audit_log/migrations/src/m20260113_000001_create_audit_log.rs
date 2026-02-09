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
                    .table(AuditLog::Table)
                    .if_not_exists()
                    .col(integer(AuditLog::Id).primary_key().auto_increment())
                    .col(uuid(AuditLog::CorrelationId))
                    .col(timestamp_with_time_zone(AuditLog::Timestamp))
                    .col(string_null(AuditLog::Operation))
                    .col(json_null(AuditLog::Params))
                    .col(boolean_null(AuditLog::IsSuccess))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum AuditLog {
    Table,
    Id,
    CorrelationId,
    Timestamp,
    Operation,
    Params,
    IsSuccess,
}
