use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StateBridge::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StateBridge::Id)
                            .big_integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(StateBridge::IssuerState)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(StateBridge::Entry).string().not_null())
                    .col(
                        ColumnDef::new(StateBridge::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum StateBridge {
    Table,
    Id,
    IssuerState,
    Entry,
    ExpiresAt,
}
