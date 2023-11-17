use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Transaction::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Transaction::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Transaction::Type).string().not_null())
                    .col(ColumnDef::new(Transaction::Timestamp).timestamp().not_null())
                    .col(ColumnDef::new(Transaction::RemotePartyCertificate).string().null())
                    .col(ColumnDef::new(Transaction::Status).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Transaction::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Transaction {
    Table,
    Id,
    Type,
    Timestamp,
    RemotePartyCertificate,
    Status,
}
