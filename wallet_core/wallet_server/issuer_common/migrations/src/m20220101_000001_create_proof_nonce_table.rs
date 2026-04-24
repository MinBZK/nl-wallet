use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProofNonce::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProofNonce::Id)
                            .big_integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(ProofNonce::Nonce).string().not_null().unique_key())
                    .col(
                        ColumnDef::new(ProofNonce::CreatedDateTime)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(ProofNonce::Table)
                    .name("proof_nonce_created_date_time")
                    .col(ProofNonce::CreatedDateTime)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ProofNonce {
    Table,
    Id,
    Nonce,
    CreatedDateTime,
}
