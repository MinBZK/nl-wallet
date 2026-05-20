use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PkceFlow::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PkceFlow::Id)
                            .big_integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(PkceFlow::WalletCodeChallenge)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(PkceFlow::UpstreamCodeVerifier).string().not_null())
                    .col(
                        ColumnDef::new(PkceFlow::ExpiresAt)
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
enum PkceFlow {
    Table,
    Id,
    WalletCodeChallenge,
    UpstreamCodeVerifier,
    ExpiresAt,
}
