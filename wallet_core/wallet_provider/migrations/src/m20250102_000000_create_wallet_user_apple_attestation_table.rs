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
                    .table(WalletUserAppleAttestation::Table)
                    .col(pk_uuid(WalletUserAppleAttestation::Id))
                    .col(
                        big_integer(WalletUserAppleAttestation::AssertionCounter).check(
                            // Emulate a u32 with a CHECK constraint, since
                            // PostgreSQL does not support unsigned integers.
                            Expr::col(WalletUserAppleAttestation::AssertionCounter)
                                .gte(0)
                                .and(Expr::col(WalletUserAppleAttestation::AssertionCounter).lte(u32::MAX)),
                        ),
                    )
                    .col(binary(WalletUserAppleAttestation::AttestationData))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum WalletUserAppleAttestation {
    Table,
    Id,
    AssertionCounter,
    AttestationData,
}
