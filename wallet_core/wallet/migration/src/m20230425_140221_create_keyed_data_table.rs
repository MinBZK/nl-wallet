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
                    .table(KeyedData::Table)
                    .col(ColumnDef::new(KeyedData::Key).text().not_null().primary_key())
                    .col(ColumnDef::new(KeyedData::Data).json().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(KeyedData::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum KeyedData {
    Table,
    Key,
    Data,
}
