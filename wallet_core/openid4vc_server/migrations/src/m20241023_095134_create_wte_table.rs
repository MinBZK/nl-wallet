use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UsedWtes::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UsedWtes::UsedWteHash).binary_len(256 / 8).not_null())
                    .col(ColumnDef::new(UsedWtes::Expires).timestamp_with_time_zone().not_null())
                    .primary_key(Index::create().col(UsedWtes::UsedWteHash))
                    .to_owned(),
            )
            .await
    }

    // this table will be used as a cache and can be dropped on migration
}

#[derive(DeriveIden)]
enum UsedWtes {
    Table,
    UsedWteHash,
    Expires,
}
