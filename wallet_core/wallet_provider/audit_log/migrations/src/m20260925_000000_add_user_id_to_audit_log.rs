use async_trait::async_trait;
use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AuditLog::Table)
                    .add_column(string_null(AuditLog::UserId))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum AuditLog {
    Table,
    UserId,
}
