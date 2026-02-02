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
                    .table(StatusListItem::Table)
                    .if_not_exists()
                    .col(small_integer(StatusListItem::AttestationTypeId))
                    .col(big_integer(StatusListItem::SequenceNo))
                    .col(big_integer(StatusListItem::StatusListId))
                    .col(integer(StatusListItem::Index))
                    .primary_key(
                        Index::create()
                            .col(StatusListItem::AttestationTypeId)
                            .col(StatusListItem::SequenceNo),
                    )
                    // Skip foreign keys as it is a big working table
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum StatusListItem {
    Table,
    AttestationTypeId,
    SequenceNo,
    StatusListId,
    Index,
}
