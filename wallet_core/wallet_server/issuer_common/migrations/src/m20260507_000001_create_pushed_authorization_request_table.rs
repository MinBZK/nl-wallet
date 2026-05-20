use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PushedAuthorizationRequest::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PushedAuthorizationRequest::Id)
                            .big_integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(PushedAuthorizationRequest::RequestUri)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(PushedAuthorizationRequest::Data)
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PushedAuthorizationRequest::ExpiresAt)
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
enum PushedAuthorizationRequest {
    Table,
    Id,
    RequestUri,
    Data,
    ExpiresAt,
}
