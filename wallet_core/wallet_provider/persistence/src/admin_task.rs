use chrono::DateTime;
use chrono::Utc;
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::ConnectionTrait;
use sea_orm::DbErr;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::QuerySelect;
use sea_orm::Set;
use uuid::Uuid;
use wallet_provider_domain::model::admin_task::AdminTask;
use wallet_provider_domain::model::admin_task::AdminTaskCreate;
use wallet_provider_domain::model::admin_task::AdminTaskReview;
use wallet_provider_domain::model::admin_task::TaskListView;
use wallet_provider_domain::model::admin_task::TaskStatus;
use wallet_provider_domain::repository::PersistenceError;

use crate::PersistenceConnection;
use crate::entity::admin_task;
use crate::transaction::Transaction;

impl From<AdminTaskCreate> for admin_task::ActiveModel {
    fn from(task: AdminTaskCreate) -> Self {
        Self {
            id: Set(task.id),
            created_at: Set(task.created_at.into()),
            created_by_user_id: Set(task.created_by_user_id),
            created_by_display_name: Set(task.created_by_display_name),
            task_type: Set(task.task_type.to_string()),
            task_parameters: Set(task.task_parameters),
            task_reason: Set(task.task_reason),
            status: Set(TaskStatus::Created.to_string()),
            withdrawn_at: Set(None),
            reviewed_at: Set(None),
            reviewed_by_user_id: Set(None),
            reviewed_by_display_name: Set(None),
            review_comment: Set(None),
            executed_at: Set(None),
            execution_result: Set(None),
            execution_result_details: Set(None),
        }
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
            task_type: model.task_type.parse().map_err(PersistenceError::EnumConversion)?,
            task_parameters: model.task_parameters,
            task_reason: model.task_reason,
            status: model.status.parse().map_err(PersistenceError::EnumConversion)?,
            withdrawn_at: model.withdrawn_at.map(Into::into),
            reviewed_at: model.reviewed_at.map(Into::into),
            reviewed_by_user_id: model.reviewed_by_user_id,
            reviewed_by_display_name: model.reviewed_by_display_name,
            review_comment: model.review_comment,
            executed_at: model.executed_at.map(Into::into),
            execution_result: model.execution_result,
            execution_result_details: model.execution_result_details,
        })
    }
}

pub async fn insert_task<S, T>(db: &T, task: AdminTaskCreate) -> Result<AdminTask, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let model: admin_task::ActiveModel = task.into();
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

pub async fn withdraw_task<S, T>(
    db: &T,
    task_id: Uuid,
    now: DateTime<Utc>,
) -> Result<Option<AdminTask>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let model = admin_task::ActiveModel {
        id: Set(task_id),
        status: Set(TaskStatus::Withdrawn.to_string()),
        withdrawn_at: Set(Some(now.into())),
        ..Default::default()
    };
    // Allow changing to Withdrawn status as well for idempotency.
    match admin_task::Entity::update(model)
        .filter(admin_task::Column::Status.is_in(&[TaskStatus::Created.to_string(), TaskStatus::Withdrawn.to_string()]))
        .exec(db.connection())
        .await
    {
        Ok(task) => Ok(Some(task.try_into()?)),
        Err(DbErr::RecordNotUpdated) => Ok(None),
        Err(error) => Err(PersistenceError::Execution(error)),
    }
}

pub async fn review_task<S, T>(db: &T, review: AdminTaskReview) -> Result<Option<AdminTask>, PersistenceError>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let model = admin_task::ActiveModel {
        id: Set(review.id),
        status: Set(review.review_verdict.resulting_status().to_string()),
        reviewed_at: Set(Some(review.reviewed_at.into())),
        reviewed_by_user_id: Set(Some(review.reviewed_by_user_id)),
        reviewed_by_display_name: Set(Some(review.reviewed_by_display_name)),
        review_comment: Set(Some(review.review_comment)),
        ..Default::default()
    };
    // Allow changing to review verdict status as well for idempotency.
    match admin_task::Entity::update(model)
        .filter(admin_task::Column::Status.is_in(&[
            TaskStatus::Created.to_string(),
            review.review_verdict.resulting_status().to_string(),
        ]))
        .exec(db.connection())
        .await
    {
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
    use chrono::FixedOffset;
    use wallet_provider_domain::model::admin_task::TaskType;

    use super::*;

    fn model() -> admin_task::Model {
        let offset = FixedOffset::east_opt(3600).unwrap();
        admin_task::Model {
            id: Uuid::from_u128(42),
            created_at: DateTime::from_timestamp(1_700_000_000, 123_456_000)
                .unwrap()
                .with_timezone(&offset),
            created_by_user_id: "creator-id".into(),
            created_by_display_name: "Creator name".into(),
            task_type: "block_user".into(),
            task_parameters: sea_orm::JsonValue::from("parameters"),
            task_reason: "task reason".into(),
            status: "approved".into(),
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
            execution_result: Some("success".into()),
            execution_result_details: Some(sea_orm::JsonValue::from("execution details")),
        }
    }

    #[test]
    fn create_maps_fields_and_initializes_lifecycle_fields() {
        for (task_type, stored_type) in [
            (TaskType::RevokeWallet, "revoke_wallet"),
            (TaskType::BlockUser, "block_user"),
            (TaskType::UnblockUser, "unblock_user"),
            (TaskType::RevokeSolution, "revoke_solution"),
        ] {
            let row = model();
            let create = AdminTaskCreate {
                id: row.id,
                created_at: row.created_at.into(),
                created_by_user_id: row.created_by_user_id.clone(),
                created_by_display_name: row.created_by_display_name.clone(),
                task_type,
                task_parameters: row.task_parameters.clone(),
                task_reason: row.task_reason.clone(),
            };
            let active = admin_task::ActiveModel::from(create);
            assert_eq!(active.id, Set(row.id));
            assert_eq!(
                active.created_at,
                Set(row.created_at.with_timezone(&Utc).fixed_offset())
            );
            assert_eq!(active.created_by_user_id, Set(row.created_by_user_id));
            assert_eq!(active.created_by_display_name, Set(row.created_by_display_name));
            assert_eq!(active.task_type, Set(stored_type.to_owned()));
            assert_eq!(active.task_parameters, Set(row.task_parameters));
            assert_eq!(active.task_reason, Set(row.task_reason));
            assert_eq!(active.status, Set("created".to_owned()));
            assert_eq!(active.withdrawn_at, Set(None));
            assert_eq!(active.reviewed_at, Set(None));
            assert_eq!(active.reviewed_by_user_id, Set(None));
            assert_eq!(active.reviewed_by_display_name, Set(None));
            assert_eq!(active.review_comment, Set(None));
            assert_eq!(active.executed_at, Set(None));
            assert_eq!(active.execution_result, Set(None));
            assert_eq!(active.execution_result_details, Set(None));
        }
    }

    #[test]
    fn model_maps_all_fields() {
        let (stored_type, task_type) = ("revoke_wallet", TaskType::RevokeWallet);
        let (stored_status, status) = ("created", TaskStatus::Created);
        let row = admin_task::Model {
            task_type: stored_type.into(),
            status: stored_status.into(),
            ..model()
        };
        let expected = AdminTask {
            id: row.id,
            created_at: row.created_at.into(),
            created_by_user_id: row.created_by_user_id.clone(),
            created_by_display_name: row.created_by_display_name.clone(),
            task_type,
            task_parameters: row.task_parameters.clone(),
            task_reason: row.task_reason.clone(),
            status,
            withdrawn_at: row.withdrawn_at.map(Into::into),
            reviewed_at: row.reviewed_at.map(Into::into),
            reviewed_by_user_id: row.reviewed_by_user_id.clone(),
            reviewed_by_display_name: row.reviewed_by_display_name.clone(),
            review_comment: row.review_comment.clone(),
            executed_at: row.executed_at.map(Into::into),
            execution_result: row.execution_result.clone(),
            execution_result_details: row.execution_result_details.clone(),
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
    fn model_preserves_absent_optional_fields() {
        let row = admin_task::Model {
            withdrawn_at: None,
            reviewed_at: None,
            reviewed_by_user_id: None,
            reviewed_by_display_name: None,
            review_comment: None,
            executed_at: None,
            execution_result: None,
            execution_result_details: None,
            ..model()
        };
        let task = AdminTask::try_from(row).unwrap();
        assert_eq!(task.withdrawn_at, None);
        assert_eq!(task.reviewed_at, None);
        assert_eq!(task.reviewed_by_user_id, None);
        assert_eq!(task.reviewed_by_display_name, None);
        assert_eq!(task.review_comment, None);
        assert_eq!(task.executed_at, None);
        assert_eq!(task.execution_result, None);
        assert_eq!(task.execution_result_details, None);
    }
}
