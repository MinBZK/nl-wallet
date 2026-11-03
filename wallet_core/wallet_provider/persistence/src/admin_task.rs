use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::ConnectionTrait;
use sea_orm::DbErr;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::QuerySelect;
use sea_orm::Set;
use sea_orm::UpdateOne;
use uuid::Uuid;
use wallet_provider_domain::model::admin_task::AdminTask;
use wallet_provider_domain::model::admin_task::CreateAdminTask;
use wallet_provider_domain::model::admin_task::ExecuteAdminTask;
use wallet_provider_domain::model::admin_task::ReviewAdminTask;
use wallet_provider_domain::model::admin_task::TaskListView;
use wallet_provider_domain::model::admin_task::TaskPayload;
use wallet_provider_domain::model::admin_task::TaskStatus;
use wallet_provider_domain::model::admin_task::WithdrawAdminTask;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::admin_task;
use crate::transaction::Transaction;

impl TryFrom<CreateAdminTask> for admin_task::ActiveModel {
    type Error = PersistenceError;

    fn try_from(task: CreateAdminTask) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Set(task.id),
            created_at: Set(task.created_at.into()),
            created_by_user_id: Set(task.created_by_user_id),
            created_by_display_name: Set(task.created_by_display_name),
            task_type: Set(task.payload.task_type().to_string()),
            task_parameters: Set(task
                .payload
                .parameters_json()
                .map_err(PersistenceError::TaskParametersConversion)?),
            task_reason: Set(task.task_reason),
            status: Set(TaskStatus::Created.to_string()),
            withdrawal_comment: Set(None),
            withdrawn_at: Set(None),
            reviewed_at: Set(None),
            reviewed_by_user_id: Set(None),
            reviewed_by_display_name: Set(None),
            review_comment: Set(None),
            executed_at: Set(None),
            execution_successful: Set(None),
            execution_details: Set(None),
        })
    }
}

impl TryFrom<admin_task::Model> for AdminTask {
    type Error = PersistenceError;

    fn try_from(model: admin_task::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: model.id,
            created_at: model.created_at.into(),
            created_by_user_id: model.created_by_user_id,
            created_by_display_name: model.created_by_display_name,
            payload: TaskPayload::from_parts(
                model.task_type.parse().map_err(PersistenceError::EnumConversion)?,
                model.task_parameters,
            )
            .map_err(PersistenceError::TaskParametersConversion)?,
            task_reason: model.task_reason,
            status: model.status.parse().map_err(PersistenceError::EnumConversion)?,
            withdrawal_comment: model.withdrawal_comment,
            withdrawn_at: model.withdrawn_at.map(Into::into),
            reviewed_at: model.reviewed_at.map(Into::into),
            reviewed_by_user_id: model.reviewed_by_user_id,
            reviewed_by_display_name: model.reviewed_by_display_name,
            review_comment: model.review_comment,
            executed_at: model.executed_at.map(Into::into),
            execution_successful: model.execution_successful,
            execution_details: model.execution_details,
        })
    }
}

pub async fn insert_task<S, T>(db: &T, task: CreateAdminTask) -> Result<AdminTask, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let model: admin_task::ActiveModel = task.try_into()?;
    let task = admin_task::Entity::insert(model)
        .exec_with_returning(db.connection())
        .await
        .map_err(PersistenceError::Execution)?
        .try_into()?;
    Ok(task)
}

pub async fn find_task_by_id<S, T>(db: &T, task_id: Uuid) -> Result<Option<AdminTask>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    admin_task::Entity::find_by_id(task_id)
        .one(db.connection())
        .await
        .map_err(PersistenceError::Execution)?
        .map(TryInto::try_into)
        .transpose()
}

pub async fn find_task_by_id_for_update(
    db: &Transaction,
    task_id: Uuid,
) -> Result<Option<AdminTask>, PersistenceError> {
    admin_task::Entity::find_by_id(task_id)
        .lock_exclusive()
        .one(db.connection())
        .await
        .map_err(PersistenceError::Execution)?
        .map(TryInto::try_into)
        .transpose()
}

pub async fn withdraw_task<S, T>(db: &T, withdrawal: WithdrawAdminTask) -> Result<Option<AdminTask>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let store_withdraw = admin_task::ActiveModel {
        id: Set(withdrawal.task_id),
        status: Set(TaskStatus::Withdrawn.to_string()),
        withdrawal_comment: Set(Some(withdrawal.withdrawal_comment)),
        withdrawn_at: Set(Some(withdrawal.withdrawn_at.into())),
        ..Default::default()
    };

    let store_withdraw = admin_task::Entity::update(store_withdraw)
        .filter(admin_task::Column::Status.eq(TaskStatus::Created.to_string()));
    update_one(db, store_withdraw).await
}

pub async fn review_task<S, T>(db: &T, review: ReviewAdminTask) -> Result<Option<AdminTask>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let store_task_review = admin_task::ActiveModel {
        id: Set(review.task_id),
        status: Set(review.review_verdict.resulting_status().to_string()),
        reviewed_at: Set(Some(review.reviewed_at.into())),
        reviewed_by_user_id: Set(Some(review.reviewed_by_user_id)),
        reviewed_by_display_name: Set(Some(review.reviewed_by_display_name)),
        review_comment: Set(Some(review.review_comment)),
        ..Default::default()
    };

    let store_task_review = admin_task::Entity::update(store_task_review)
        .filter(admin_task::Column::Status.eq(TaskStatus::Created.to_string()));
    update_one(db, store_task_review).await
}

pub async fn store_execution_result<S, T>(
    db: &T,
    execution: ExecuteAdminTask,
) -> Result<Option<AdminTask>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let store_execution_result = admin_task::ActiveModel {
        id: Set(execution.task_id),
        executed_at: Set(Some(execution.executed_at.into())),
        execution_successful: Set(Some(execution.execution_successful)),
        execution_details: Set(Some(execution.execution_details)),
        ..Default::default()
    };

    let store_execution_result = admin_task::Entity::update(store_execution_result)
        .filter(admin_task::Column::Status.eq(TaskStatus::Approved.to_string()))
        .filter(admin_task::Column::ExecutedAt.is_null());
    update_one(db, store_execution_result).await
}

/// Helper function to update one `AdminTask` and convert the result.
async fn update_one<S, T>(
    db: &T,
    update_query: UpdateOne<admin_task::ActiveModel>,
) -> Result<Option<AdminTask>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    match update_query.exec(db.connection()).await {
        Ok(task) => Ok(Some(task.try_into()?)),
        Err(DbErr::RecordNotUpdated) => Ok(None),
        Err(error) => Err(PersistenceError::Execution(error)),
    }
}

pub async fn list_tasks<S, T>(db: &T, view: TaskListView) -> Result<Vec<AdminTask>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let condition =
        match view {
            TaskListView::Open { user_id } => Condition::all()
                .add(admin_task::Column::Status.eq(TaskStatus::Created.to_string()))
                .add(admin_task::Column::CreatedByUserId.eq(user_id)),
            TaskListView::History { user_id } => Condition::all()
                .add(admin_task::Column::Status.is_in(
                    [&TaskStatus::Approved, &TaskStatus::Rejected, &TaskStatus::Withdrawn].map(ToString::to_string),
                ))
                .add(
                    Condition::any()
                        .add(admin_task::Column::CreatedByUserId.eq(&user_id))
                        .add(admin_task::Column::ReviewedByUserId.eq(&user_id)),
                ),
            TaskListView::Reviewable {
                user_id,
                authorized_task_types,
            } => {
                if authorized_task_types.is_empty() {
                    return Ok(vec![]);
                }
                Condition::all()
                    .add(admin_task::Column::Status.eq(TaskStatus::Created.to_string()))
                    .add(admin_task::Column::CreatedByUserId.ne(user_id))
                    .add(admin_task::Column::TaskType.is_in(authorized_task_types.iter().map(ToString::to_string)))
            }
        };
    let tasks = admin_task::Entity::find()
        .filter(condition)
        .order_by_asc(admin_task::Column::CreatedAt)
        .all(db.connection())
        .await
        .map_err(PersistenceError::Execution)?
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<_, _>>()?;
    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use chrono::FixedOffset;
    use chrono::Utc;
    use serde_json::json;
    use wallet_provider_domain::model::admin_task::RevokeWalletParameters;

    use super::*;

    fn payload() -> TaskPayload {
        TaskPayload::RevokeWallet(RevokeWalletParameters {
            wallet_ids: vec!["wallet-1".to_string().into()].try_into().unwrap(),
        })
    }

    fn model() -> admin_task::Model {
        let offset = FixedOffset::east_opt(3600).unwrap();
        admin_task::Model {
            id: Uuid::from_u128(42),
            created_at: DateTime::from_timestamp(1_700_000_000, 123_456_000)
                .unwrap()
                .with_timezone(&offset),
            created_by_user_id: "creator-id".into(),
            created_by_display_name: "Creator name".into(),
            task_type: "revoke_wallet".into(),
            task_parameters: json!({"walletIds": ["wallet-1"]}),
            task_reason: "task reason".into(),
            status: "approved".into(),
            withdrawal_comment: None,
            withdrawn_at: None,
            reviewed_at: Some(
                DateTime::from_timestamp(1_700_000_100, 0)
                    .unwrap()
                    .with_timezone(&offset),
            ),
            reviewed_by_user_id: Some("reviewer-id".into()),
            reviewed_by_display_name: Some("Reviewer name".into()),
            review_comment: Some("review comment".into()),
            executed_at: Some(
                DateTime::from_timestamp(1_700_000_200, 0)
                    .unwrap()
                    .with_timezone(&offset),
            ),
            execution_successful: Some(true),
            execution_details: Some("execution details".into()),
        }
    }

    #[test]
    fn create_maps_fields_and_initializes_lifecycle_fields() {
        let row = model();
        let create = CreateAdminTask {
            id: row.id,
            created_at: row.created_at.into(),
            created_by_user_id: row.created_by_user_id.clone(),
            created_by_display_name: row.created_by_display_name.clone(),
            payload: payload(),
            task_reason: row.task_reason.clone(),
        };
        let active = admin_task::ActiveModel::try_from(create).unwrap();
        assert_eq!(active.id, Set(row.id));
        assert_eq!(
            active.created_at,
            Set(row.created_at.with_timezone(&Utc).fixed_offset())
        );
        assert_eq!(active.created_by_user_id, Set(row.created_by_user_id));
        assert_eq!(active.created_by_display_name, Set(row.created_by_display_name));
        assert_eq!(active.task_type, Set(row.task_type));
        assert_eq!(active.task_parameters, Set(row.task_parameters));
        assert_eq!(active.task_reason, Set(row.task_reason));
        assert_eq!(active.status, Set("created".to_owned()));
        assert_eq!(active.withdrawal_comment, Set(None));
        assert_eq!(active.withdrawn_at, Set(None));
        assert_eq!(active.reviewed_at, Set(None));
        assert_eq!(active.reviewed_by_user_id, Set(None));
        assert_eq!(active.reviewed_by_display_name, Set(None));
        assert_eq!(active.review_comment, Set(None));
        assert_eq!(active.executed_at, Set(None));
        assert_eq!(active.execution_successful, Set(None));
        assert_eq!(active.execution_details, Set(None));
    }

    #[test]
    fn model_maps_all_fields() {
        let (stored_status, status) = ("created", TaskStatus::Created);
        let row = admin_task::Model {
            status: stored_status.into(),
            ..model()
        };
        let expected = AdminTask {
            id: row.id,
            created_at: row.created_at.into(),
            created_by_user_id: row.created_by_user_id.clone(),
            created_by_display_name: row.created_by_display_name.clone(),
            payload: payload(),
            task_reason: row.task_reason.clone(),
            status,
            withdrawal_comment: row.withdrawal_comment.clone(),
            withdrawn_at: row.withdrawn_at.map(Into::into),
            reviewed_at: row.reviewed_at.map(Into::into),
            reviewed_by_user_id: row.reviewed_by_user_id.clone(),
            reviewed_by_display_name: row.reviewed_by_display_name.clone(),
            review_comment: row.review_comment.clone(),
            executed_at: row.executed_at.map(Into::into),
            execution_successful: row.execution_successful,
            execution_details: row.execution_details.clone(),
        };
        assert_eq!(AdminTask::try_from(row).unwrap(), expected);
    }

    #[test]
    fn model_with_invalid_task_type_or_status_fails_conversion() {
        for row in [
            admin_task::Model {
                task_type: "unknown".into(),
                ..model()
            },
            admin_task::Model {
                status: "unknown".into(),
                ..model()
            },
        ] {
            assert!(matches!(
                AdminTask::try_from(row),
                Err(PersistenceError::EnumConversion(_))
            ));
        }
    }

    #[test]
    fn model_with_invalid_task_parameters_fails_conversion() {
        let row = admin_task::Model {
            task_parameters: json!({}),
            ..model()
        };
        assert!(matches!(
            AdminTask::try_from(row),
            Err(PersistenceError::TaskParametersConversion(_))
        ));
    }

    #[test]
    fn model_preserves_absent_optional_fields() {
        let row = admin_task::Model {
            withdrawn_at: None,
            reviewed_at: None,
            reviewed_by_user_id: None,
            reviewed_by_display_name: None,
            review_comment: None,
            executed_at: None,
            execution_successful: None,
            execution_details: None,
            ..model()
        };
        let task = AdminTask::try_from(row).unwrap();
        assert_eq!(task.withdrawn_at, None);
        assert_eq!(task.reviewed_at, None);
        assert_eq!(task.reviewed_by_user_id, None);
        assert_eq!(task.reviewed_by_display_name, None);
        assert_eq!(task.review_comment, None);
        assert_eq!(task.executed_at, None);
        assert_eq!(task.execution_successful, None);
        assert_eq!(task.execution_details, None);
    }
}
