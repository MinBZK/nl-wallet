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
                    .table(AdminTask::Table)
                    .if_not_exists()
                    .col(uuid(AdminTask::Id).primary_key())
                    .col(timestamp_with_time_zone(AdminTask::CreatedAt))
                    .col(string(AdminTask::CreatedByUserId))
                    .col(string(AdminTask::CreatedByDisplayName))
                    .col(string(AdminTask::TaskType))
                    .col(json(AdminTask::TaskParameters).default(Expr::cust("'{}'::jsonb")))
                    .col(string(AdminTask::TaskReason))
                    .col(string(AdminTask::Status))
                    .col(timestamp_with_time_zone_null(AdminTask::WithdrawnAt))
                    .col(timestamp_with_time_zone_null(AdminTask::ReviewedAt))
                    .col(string_null(AdminTask::ReviewedByUserId))
                    .col(string_null(AdminTask::ReviewedByDisplayName))
                    .col(string_null(AdminTask::ReviewComment))
                    .col(timestamp_with_time_zone_null(AdminTask::ExecutedAt))
                    .col(string_null(AdminTask::ExecutionResult))
                    .col(json_null(AdminTask::ExecutionResultDetails))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_admin_task_created_by_user_id")
                    .table(AdminTask::Table)
                    .col(AdminTask::CreatedByUserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_admin_task_created_task_type")
                    .table(AdminTask::Table)
                    .col(AdminTask::TaskType)
                    .and_where(Expr::col(AdminTask::Status).eq("created"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_admin_task_reviewed_by_user_id")
                    .table(AdminTask::Table)
                    .col(AdminTask::ReviewedByUserId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum AdminTask {
    Table,
    Id,
    CreatedAt,
    CreatedByUserId,
    CreatedByDisplayName,
    TaskType,
    TaskParameters,
    TaskReason,
    Status,
    WithdrawnAt,
    ReviewedAt,
    ReviewedByUserId,
    ReviewedByDisplayName,
    ReviewComment,
    ExecutedAt,
    ExecutionResult,
    ExecutionResultDetails,
}
