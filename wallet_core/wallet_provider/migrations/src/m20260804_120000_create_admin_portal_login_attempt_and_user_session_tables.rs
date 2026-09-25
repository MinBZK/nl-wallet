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
                    .table(AdminPortalLoginAttempt::Table)
                    .if_not_exists()
                    .col(text(AdminPortalLoginAttempt::State).primary_key())
                    .col(string(AdminPortalLoginAttempt::Nonce))
                    .col(string(AdminPortalLoginAttempt::CodeVerifier))
                    .col(timestamp_with_time_zone(AdminPortalLoginAttempt::CreatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AdminPortalUserSession::Table)
                    .if_not_exists()
                    .col(text(AdminPortalUserSession::Id).primary_key())
                    .col(string(AdminPortalUserSession::DisplayName))
                    .col(array(AdminPortalUserSession::Roles, ColumnType::Text))
                    .col(string(AdminPortalUserSession::IdToken))
                    .col(timestamp_with_time_zone(AdminPortalUserSession::ExpiresAt))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AdminPortalLoginAttempt {
    Table,
    State,
    Nonce,
    CodeVerifier,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AdminPortalUserSession {
    Table,
    Id,
    DisplayName,
    Roles,
    IdToken,
    ExpiresAt,
}
