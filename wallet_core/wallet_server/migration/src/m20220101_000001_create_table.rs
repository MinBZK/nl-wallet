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
                    .table(SessionState::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SessionState::Token).string().not_null().primary_key())
                    .col(ColumnDef::new(SessionState::Data).json().not_null())
                    .col(ColumnDef::new(SessionState::Type).text().not_null())
                    .col(
                        ColumnDef::new(SessionState::ExpirationDateTime)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    // this table will be used as a cache and can be dropped on migration
}

#[derive(DeriveIden)]
enum SessionState {
    Table,
    Token,
    Data,
    ExpirationDateTime,
    Type,
}
