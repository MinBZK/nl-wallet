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
                    .table(AttestationType::Table)
                    .if_not_exists()
                    .col(small_integer(AttestationType::Id).primary_key().auto_increment())
                    .col(string(AttestationType::Name))
                    .col(big_integer(AttestationType::NextSequenceNo))
                    .index(
                        Index::create()
                            .unique()
                            .name("attestation_type_unique_name")
                            .col(AttestationType::Name),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum AttestationType {
    Table,
    Id,
    Name,
    NextSequenceNo,
}
