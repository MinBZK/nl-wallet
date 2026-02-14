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
                    .table(DeniedRecoveryCode::Table)
                    .if_not_exists()
                    .col(pk_auto(DeniedRecoveryCode::Id))
                    .col(string(DeniedRecoveryCode::RecoveryCode))
                    .col(boolean(DeniedRecoveryCode::IsDenied).default(false))
                    .index(
                        Index::create()
                            .unique()
                            .name("denied_recovery_code_unique_recovery_code")
                            .col(DeniedRecoveryCode::RecoveryCode),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(DeniedRecoveryCode::Table)
                    .name("denied_recovery_code_is_denied")
                    .col(DeniedRecoveryCode::IsDenied)
                    .and_where(Expr::col(DeniedRecoveryCode::IsDenied).eq(true))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum DeniedRecoveryCode {
    Table,
    Id,
    RecoveryCode,
    IsDenied,
}
