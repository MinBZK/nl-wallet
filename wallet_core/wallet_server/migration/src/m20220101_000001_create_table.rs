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
                    .col(ColumnDef::new(SessionState::Type).string().not_null())
                    .col(ColumnDef::new(SessionState::Token).string().not_null())
                    .col(ColumnDef::new(SessionState::Data).json().not_null())
                    .col(ColumnDef::new(SessionState::Status).string().not_null())
                    .col(
                        ColumnDef::new(SessionState::LastActiveDateTime)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .primary_key(Index::create().col(SessionState::Type).col(SessionState::Token))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(SessionState::Table)
                    .col(SessionState::Type)
                    .col(SessionState::Status)
                    .col(SessionState::LastActiveDateTime)
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
    Type,
    Token,
    Data,
    Status,
    LastActiveDateTime,
}
