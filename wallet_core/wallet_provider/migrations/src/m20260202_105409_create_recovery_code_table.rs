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
                    .table(RecoveryCode::Table)
                    .if_not_exists()
                    .col(pk_auto(RecoveryCode::Id))
                    .col(string(RecoveryCode::RecoveryCode))
                    .col(boolean(RecoveryCode::IsDenied).default(false))
                    .index(
                        Index::create()
                            .unique()
                            .name("recovery_code_unique_recovery_code")
                            .col(RecoveryCode::RecoveryCode),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(RecoveryCode::Table)
                    .name("recovery_code_is_denied")
                    .col(RecoveryCode::IsDenied)
                    .and_where(Expr::col(RecoveryCode::IsDenied).eq(true))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum RecoveryCode {
    Table,
    Id,
    #[expect(clippy::enum_variant_names, reason = "There is not better name for this column")]
    RecoveryCode,
    IsDenied,
}
