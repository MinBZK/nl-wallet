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
                    .table(AttestationGroup::Table)
                    .if_not_exists()
                    .col(small_integer(AttestationGroup::Id).primary_key().auto_increment())
                    .col(string(AttestationGroup::Name))
                    .col(big_integer(AttestationGroup::NextSequenceNo))
                    .index(
                        Index::create()
                            .unique()
                            .name("attestation_group_unique_name")
                            .col(AttestationGroup::Name),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum AttestationGroup {
    Table,
    Id,
    Name,
    NextSequenceNo,
}
