use std::collections::HashSet;
use std::error::Error;

use audit_log::audited;
use audit_log::model::AuditLog;
use audit_log::model::FromAuditLogError;
use chrono::DateTime;
use chrono::Utc;
use serde_json::Value as JsonValue;
use utils::generator::Generator;
use uuid::Uuid;
use wallet_provider_domain::model::admin_task::AdminTask;
use wallet_provider_domain::model::admin_task::AdminTaskCreate;
use wallet_provider_domain::model::admin_task::AdminTaskReview;
use wallet_provider_domain::model::admin_task::ReviewVerdict;
use wallet_provider_domain::model::admin_task::TaskListView;
use wallet_provider_domain::model::admin_task::TaskStatus;
use wallet_provider_domain::model::admin_task::TaskType;
use wallet_provider_domain::repository::AdminTaskRepository;
use wallet_provider_domain::repository::PersistenceError;

#[derive(Debug, thiserror::Error)]
pub enum CreateTaskError {
    #[error("unknown task type: {0}")]
    UnknownTaskType(String),
    #[error("missing privilege for task type {0}")]
    MissingPrivilege(TaskType),
    #[error("persistence error: {0}")]
    Storage(#[source] PersistenceError),
    #[error("error while auditing: {0}")]
    AuditLog(#[source] Box<dyn Error + Send + Sync>),
}

impl FromAuditLogError for CreateTaskError {
    fn from_audit_log_error(audit_log_error: Box<dyn Error + Send + Sync>) -> Self {
        Self::AuditLog(audit_log_error)
    }
}

#[expect(clippy::too_many_arguments, reason = "audited function")]
#[audited]
pub async fn create_task<R: AdminTaskRepository>(
    #[audit] task_type: &str,
    #[audit] task_parameters: &JsonValue,
    #[audit] task_reason: &str,
    #[audit_user] user_id: &str,
    user_display_name: &str,
    authorized_task_types: &HashSet<TaskType>,
    repository: &R,
    time: &impl Generator<DateTime<Utc>>,
    uuid: &impl Generator<Uuid>,
    #[auditor] audit_log: &impl AuditLog,
) -> Result<AdminTask, CreateTaskError> {
    let parsed_task_type: TaskType = task_type
        .parse()
        .map_err(|_| CreateTaskError::UnknownTaskType(task_type.to_string()))?;

    if !authorized_task_types.contains(&parsed_task_type) {
        return Err(CreateTaskError::MissingPrivilege(parsed_task_type));
    }

    let task = repository
        .insert_task(AdminTaskCreate {
            id: uuid.generate(),
            created_at: time.generate(),
            created_by_user_id: user_id.to_string(),
            created_by_display_name: user_display_name.to_string(),
            task_type: parsed_task_type,
            task_parameters: task_parameters.clone(),
            task_reason: task_reason.to_string(),
        })
        .await
        .map_err(CreateTaskError::Storage)?;

    Ok(task)
}

#[derive(Debug, thiserror::Error)]
pub enum WithdrawTaskError {
    #[error("task not found: {0}")]
    NotFound(Uuid),
    #[error("task cannot be withdrawn: not created by caller, or not in 'created' status")]
    InvalidState,
    #[error("persistence error: {0}")]
    Storage(#[source] PersistenceError),
    #[error("error while auditing: {0}")]
    AuditLog(#[source] Box<dyn Error + Send + Sync>),
}

impl FromAuditLogError for WithdrawTaskError {
    fn from_audit_log_error(audit_log_error: Box<dyn Error + Send + Sync>) -> Self {
        Self::AuditLog(audit_log_error)
    }
}

/// Withdraws a task created by the caller that is still in `Created` status.
#[audited]
pub async fn withdraw_task<R: AdminTaskRepository>(
    #[audit] task_id: Uuid,
    #[audit_user] user_id: &str,
    repository: &R,
    time: &impl Generator<DateTime<Utc>>,
    #[auditor] audit_log: &impl AuditLog,
) -> Result<AdminTask, WithdrawTaskError> {
    // Look up the task to distinguish a missing task (NotFound) from an existing task with the wrong owner or status
    // (InvalidState). The conditional update alone returns None for all of these cases.
    let existing = repository
        .find_task_by_id(task_id)
        .await
        .map_err(WithdrawTaskError::Storage)?
        .ok_or(WithdrawTaskError::NotFound(task_id))?;

    if existing.created_by_user_id != user_id || existing.status != TaskStatus::Created {
        return Err(WithdrawTaskError::InvalidState);
    }

    // Actually try to withdraw the task now, this repeats the ownership and status checks atomically. A concurrent
    // withdrawal or review could have changed it in between; None then becomes InvalidState.
    //
    // A transaction alone does not prevent this race at PostgreSQL's default isolation level; the conditional update
    // does.
    repository
        .withdraw_task(task_id, user_id, time.generate())
        .await
        .map_err(WithdrawTaskError::Storage)?
        .ok_or(WithdrawTaskError::InvalidState)
}

#[derive(Debug, thiserror::Error)]
pub enum ReviewTaskError {
    #[error("task not found: {0}")]
    NotFound(Uuid),
    #[error("missing privilege for task type {0}")]
    MissingPrivilege(TaskType),
    #[error("task cannot be reviewed: created by caller, or not in 'created' status")]
    InvalidState,
    #[error("persistence error: {0}")]
    Storage(#[source] PersistenceError),
    #[error("error while auditing: {0}")]
    AuditLog(#[source] Box<dyn Error + Send + Sync>),
}

impl FromAuditLogError for ReviewTaskError {
    fn from_audit_log_error(audit_log_error: Box<dyn Error + Send + Sync>) -> Self {
        Self::AuditLog(audit_log_error)
    }
}

#[expect(clippy::too_many_arguments, reason = "audited function")]
#[audited]
pub async fn review_task<R: AdminTaskRepository>(
    #[audit] task_id: Uuid,
    #[audit] verdict: ReviewVerdict,
    #[audit] review_comment: &str,
    #[audit_user] reviewer_user_id: &str,
    reviewer_display_name: &str,
    authorized_task_types: &HashSet<TaskType>,
    repository: &R,
    time: &impl Generator<DateTime<Utc>>,
    #[auditor] audit_log: &impl AuditLog,
) -> Result<AdminTask, ReviewTaskError> {
    let existing = repository
        .find_task_by_id(task_id)
        .await
        .map_err(ReviewTaskError::Storage)?
        .ok_or(ReviewTaskError::NotFound(task_id))?;

    if existing.status != TaskStatus::Created || existing.created_by_user_id == reviewer_user_id {
        return Err(ReviewTaskError::InvalidState);
    }

    if !authorized_task_types.contains(&existing.task_type) {
        return Err(ReviewTaskError::MissingPrivilege(existing.task_type));
    }

    let review = AdminTaskReview {
        id: existing.id,
        reviewed_at: time.generate(),
        reviewed_by_user_id: reviewer_user_id.to_string(),
        reviewed_by_display_name: reviewer_display_name.to_string(),
        review_comment: review_comment.to_string(),
        review_verdict: verdict,
    };
    let updated = repository
        .review_task(review)
        .await
        .map_err(ReviewTaskError::Storage)?
        .ok_or(ReviewTaskError::InvalidState)?;

    Ok(updated)
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString, serde::Serialize, serde::Deserialize,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TaskListViewKind {
    Open,
    History,
    Reviewable,
}

#[derive(Debug, thiserror::Error)]
pub enum ListTasksError {
    #[error("persistence error: {0}")]
    Storage(#[source] PersistenceError),
    #[error("error while auditing: {0}")]
    AuditLog(#[source] Box<dyn Error + Send + Sync>),
}

impl FromAuditLogError for ListTasksError {
    fn from_audit_log_error(audit_log_error: Box<dyn Error + Send + Sync>) -> Self {
        Self::AuditLog(audit_log_error)
    }
}

#[audited]
pub async fn list_tasks<R: AdminTaskRepository>(
    #[audit] view_kind: TaskListViewKind,
    #[audit_user] user_id: &str,
    authorized_task_types: &HashSet<TaskType>,
    repository: &R,
    #[auditor] audit_log: &impl AuditLog,
) -> Result<Vec<AdminTask>, ListTasksError> {
    let user_id = user_id.to_string();

    let view = match view_kind {
        TaskListViewKind::Open => TaskListView::Open { user_id },
        TaskListViewKind::History => TaskListView::History { user_id },
        TaskListViewKind::Reviewable => TaskListView::Reviewable {
            user_id,
            authorized_task_types: authorized_task_types.iter().copied().collect(),
        },
    };

    let tasks = repository.list_tasks(view).await.map_err(ListTasksError::Storage)?;

    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use audit_log::model::mock::MockAuditLog;
    use chrono::DateTime;
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;
    use wallet_provider_domain::model::admin_task::TaskStatus;
    use wallet_provider_domain::model::admin_task::TaskType;
    use wallet_provider_persistence::repositories::mock::MockAdminTaskRepository;

    use super::*;

    struct FixedTime(DateTime<Utc>);
    impl Generator<DateTime<Utc>> for FixedTime {
        fn generate(&self) -> DateTime<Utc> {
            self.0
        }
    }

    struct FixedUuid(Uuid);
    impl Generator<Uuid> for FixedUuid {
        fn generate(&self) -> Uuid {
            self.0
        }
    }

    fn stub_task(id: Uuid, task_type: TaskType, created_by_user_id: &str) -> AdminTask {
        AdminTask {
            id,
            created_at: Utc::now(),
            created_by_user_id: created_by_user_id.to_string(),
            created_by_display_name: "Display Name".to_string(),
            task_type,
            task_parameters: json!({}),
            task_reason: "reason".to_string(),
            status: TaskStatus::Created,
            withdrawn_at: None,
            reviewed_at: None,
            reviewed_by_user_id: None,
            reviewed_by_display_name: None,
            review_comment: None,
            executed_at: None,
            execution_result: None,
            execution_result_details: None,
        }
    }

    #[tokio::test]
    async fn create_task_rejects_unknown_task_type() {
        let repository = MockAdminTaskRepository::new();

        let result = create_task(
            "not_a_real_task_type",
            &json!({}),
            "reason",
            "user-1",
            "User One",
            &HashSet::from([TaskType::RevokeWallet]),
            &repository,
            &FixedTime(Utc::now()),
            &FixedUuid(Uuid::new_v4()),
            &MockAuditLog,
        )
        .await;

        assert_matches!(result, Err(CreateTaskError::UnknownTaskType(t)) if t == "not_a_real_task_type");
    }

    #[tokio::test]
    async fn create_task_rejects_missing_privilege() {
        let repository = MockAdminTaskRepository::new();

        let result = create_task(
            "revoke_wallet",
            &json!({}),
            "reason",
            "user-1",
            "User One",
            &HashSet::from([TaskType::BlockUser]),
            &repository,
            &FixedTime(Utc::now()),
            &FixedUuid(Uuid::new_v4()),
            &MockAuditLog,
        )
        .await;

        assert_matches!(result, Err(CreateTaskError::MissingPrivilege(TaskType::RevokeWallet)));
    }

    #[tokio::test]
    async fn create_task_inserts_task_with_created_status_when_privileged() {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let mut repository = MockAdminTaskRepository::new();
        repository
            .expect_insert_task()
            .withf(move |task| {
                task.id == id
                    && task.created_at == now
                    && task.created_by_user_id == "user-1"
                    && task.created_by_display_name == "User One"
                    && task.task_type == TaskType::RevokeWallet
                    && task.task_reason == "because"
            })
            .returning(move |create| Ok(stub_task(create.id, create.task_type, &create.created_by_user_id)));

        let result = create_task(
            "revoke_wallet",
            &json!({}),
            "because",
            "user-1",
            "User One",
            &HashSet::from([TaskType::RevokeWallet]),
            &repository,
            &FixedTime(now),
            &FixedUuid(id),
            &MockAuditLog,
        )
        .await
        .expect("task should be created");

        assert_eq!(result.id, id);
        assert_eq!(result.status, TaskStatus::Created);
    }

    #[tokio::test]
    async fn review_task_returns_not_found_for_unknown_task() {
        let id = Uuid::new_v4();
        let mut repository = MockAdminTaskRepository::new();
        repository.expect_find_task_by_id().returning(|_| Ok(None));

        let result = review_task(
            id,
            ReviewVerdict::Approve,
            "comment",
            "user-2",
            "User Two",
            &HashSet::from([TaskType::RevokeWallet]),
            &repository,
            &FixedTime(Utc::now()),
            &MockAuditLog,
        )
        .await;

        assert_matches!(result, Err(ReviewTaskError::NotFound(returned_id)) if returned_id == id);
    }

    #[tokio::test]
    async fn review_task_rejects_the_tasks_own_creator() {
        let id = Uuid::new_v4();
        let mut repository = MockAdminTaskRepository::new();
        repository
            .expect_find_task_by_id()
            .returning(move |_| Ok(Some(stub_task(id, TaskType::RevokeWallet, "user-1"))));

        let result = review_task(
            id,
            ReviewVerdict::Approve,
            "comment",
            "user-1",
            "User One",
            &HashSet::from([TaskType::RevokeWallet]),
            &repository,
            &FixedTime(Utc::now()),
            &MockAuditLog,
        )
        .await;

        assert_matches!(result, Err(ReviewTaskError::InvalidState));
    }

    #[tokio::test]
    async fn review_task_rejects_a_task_not_in_created_status() {
        let id = Uuid::new_v4();
        let task = {
            let mut task = stub_task(id, TaskType::RevokeWallet, "user-1");
            task.status = TaskStatus::Withdrawn;
            task
        };
        let mut repository = MockAdminTaskRepository::new();
        repository
            .expect_find_task_by_id()
            .returning(move |_| Ok(Some(task.clone())));

        let result = review_task(
            id,
            ReviewVerdict::Approve,
            "comment",
            "user-2",
            "User Two",
            &HashSet::from([TaskType::RevokeWallet]),
            &repository,
            &FixedTime(Utc::now()),
            &MockAuditLog,
        )
        .await;

        assert_matches!(result, Err(ReviewTaskError::InvalidState));
    }

    #[tokio::test]
    async fn review_task_rejects_a_reviewer_without_privilege() {
        let id = Uuid::new_v4();
        let mut repository = MockAdminTaskRepository::new();
        repository
            .expect_find_task_by_id()
            .returning(move |_| Ok(Some(stub_task(id, TaskType::RevokeWallet, "user-1"))));

        let result = review_task(
            id,
            ReviewVerdict::Approve,
            "comment",
            "user-2",
            "User Two",
            &HashSet::from([TaskType::BlockUser]),
            &repository,
            &FixedTime(Utc::now()),
            &MockAuditLog,
        )
        .await;

        assert_matches!(result, Err(ReviewTaskError::MissingPrivilege(TaskType::RevokeWallet)));
    }

    #[tokio::test]
    async fn review_task_approves_for_a_privileged_non_creator() {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let mut repository = MockAdminTaskRepository::new();
        repository
            .expect_find_task_by_id()
            .returning(move |_| Ok(Some(stub_task(id, TaskType::RevokeWallet, "user-1"))));
        repository
            .expect_review_task()
            .withf(|review| {
                review.reviewed_by_user_id == "user-2"
                    && review.reviewed_by_display_name == "User Two"
                    && review.review_verdict == ReviewVerdict::Approve
                    && review.review_comment == "looks good"
            })
            .returning(move |_| {
                let mut task = stub_task(id, TaskType::RevokeWallet, "user-1");
                task.status = TaskStatus::Approved;
                Ok(Some(task))
            });

        let result = review_task(
            id,
            ReviewVerdict::Approve,
            "looks good",
            "user-2",
            "User Two",
            &HashSet::from([TaskType::RevokeWallet]),
            &repository,
            &FixedTime(now),
            &MockAuditLog,
        )
        .await
        .expect("should review");

        assert_eq!(result.status, TaskStatus::Approved);
    }

    #[tokio::test]
    async fn list_tasks_open_queries_for_the_caller_as_creator() {
        let mut repository = MockAdminTaskRepository::new();
        repository
            .expect_list_tasks()
            .withf(|view| matches!(view, TaskListView::Open { user_id } if user_id == "user-1"))
            .returning(|_| Ok(vec![]));

        list_tasks(
            TaskListViewKind::Open,
            "user-1",
            &HashSet::new(),
            &repository,
            &MockAuditLog,
        )
        .await
        .expect("should list");
    }

    #[tokio::test]
    async fn list_tasks_history_queries_for_the_caller() {
        let mut repository = MockAdminTaskRepository::new();
        repository
            .expect_list_tasks()
            .withf(|view| matches!(view, TaskListView::History { user_id } if user_id == "user-1"))
            .returning(|_| Ok(vec![]));

        list_tasks(
            TaskListViewKind::History,
            "user-1",
            &HashSet::new(),
            &repository,
            &MockAuditLog,
        )
        .await
        .expect("should list");
    }

    #[tokio::test]
    async fn list_tasks_reviewable_passes_authorized_task_types() {
        let mut repository = MockAdminTaskRepository::new();
        repository
            .expect_list_tasks()
            .withf(|view| {
                matches!(
                    view,
                    TaskListView::Reviewable { user_id, authorized_task_types }
                        if user_id == "user-1" && authorized_task_types == &[TaskType::RevokeWallet]
                )
            })
            .returning(|_| Ok(vec![]));

        list_tasks(
            TaskListViewKind::Reviewable,
            "user-1",
            &HashSet::from([TaskType::RevokeWallet]),
            &repository,
            &MockAuditLog,
        )
        .await
        .expect("should list");
    }

    #[tokio::test]
    async fn list_tasks_reviewable_without_authorized_task_types_still_calls_repository_with_empty_types() {
        let mut repository = MockAdminTaskRepository::new();
        repository
            .expect_list_tasks()
            .withf(|view| matches!(view, TaskListView::Reviewable { authorized_task_types, .. } if authorized_task_types.is_empty()))
            .returning(|_| Ok(vec![]));

        list_tasks(
            TaskListViewKind::Reviewable,
            "user-1",
            &HashSet::new(),
            &repository,
            &MockAuditLog,
        )
        .await
        .expect("should list");
    }

    #[tokio::test]
    async fn withdraw_task_returns_not_found() {
        let id = Uuid::new_v4();
        let mut repository = MockAdminTaskRepository::new();
        repository
            .expect_find_task_by_id()
            .withf(move |task_id| *task_id == id)
            .times(1)
            .returning(|_| Ok(None));
        let result = withdraw_task(id, "user-1", &repository, &FixedTime(Utc::now()), &MockAuditLog).await;
        assert_matches!(result, Err(WithdrawTaskError::NotFound(task_id)) if task_id == id);
    }

    #[rstest::rstest]
    #[case::other_user("user-2", TaskStatus::Created)]
    #[case::already_withdrawn("user-1", TaskStatus::Withdrawn)]
    #[case::already_approved("user-1", TaskStatus::Approved)]
    #[case::already_rejected("user-1", TaskStatus::Rejected)]
    #[tokio::test]
    async fn withdraw_task_rejects_invalid_state(#[case] user_id: &str, #[case] status: TaskStatus) {
        let id = Uuid::new_v4();
        let mut task = stub_task(id, TaskType::RevokeWallet, "user-1");
        task.status = status;
        let mut repository = MockAdminTaskRepository::new();
        repository
            .expect_find_task_by_id()
            .times(1)
            .returning(move |_| Ok(Some(task.clone())));
        let result = withdraw_task(id, user_id, &repository, &FixedTime(Utc::now()), &MockAuditLog).await;
        assert_matches!(result, Err(WithdrawTaskError::InvalidState));
    }

    #[rstest::rstest]
    #[case::successful_withdrawal(false, false)]
    #[case::task_changed_after_lookup(true, false)]
    #[case::storage_failure(false, true)]
    #[tokio::test]
    async fn withdraw_task_handles_atomic_update(#[case] task_changed_after_lookup: bool, #[case] storage_error: bool) {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let mut repository = MockAdminTaskRepository::new();
        repository
            .expect_find_task_by_id()
            .times(1)
            .returning(move |_| Ok(Some(stub_task(id, TaskType::RevokeWallet, "user-1"))));
        repository
            .expect_withdraw_task()
            .withf(move |task_id, user_id, timestamp| *task_id == id && user_id == "user-1" && *timestamp == now)
            .times(1)
            .returning(move |_, _, _| {
                if storage_error {
                    return Err(PersistenceError::NoRowsUpdated);
                }
                if task_changed_after_lookup {
                    return Ok(None);
                }
                let mut task = stub_task(id, TaskType::RevokeWallet, "user-1");
                task.status = TaskStatus::Withdrawn;
                task.withdrawn_at = Some(now);
                Ok(Some(task))
            });
        let result = withdraw_task(id, "user-1", &repository, &FixedTime(now), &MockAuditLog).await;
        if storage_error {
            assert_matches!(result, Err(WithdrawTaskError::Storage(_)));
        } else if task_changed_after_lookup {
            assert_matches!(result, Err(WithdrawTaskError::InvalidState));
        } else {
            let task = result.unwrap();
            assert_eq!(task.status, TaskStatus::Withdrawn);
            assert_eq!(task.withdrawn_at, Some(now));
            assert_eq!(task.reviewed_at, None);
            assert_eq!(task.executed_at, None);
            assert_eq!(task.execution_result, None);
            assert_eq!(task.execution_result_details, None);
        }
    }

    #[tokio::test]
    async fn withdraw_task_reports_lookup_failure() {
        let mut repository = MockAdminTaskRepository::new();
        repository
            .expect_find_task_by_id()
            .times(1)
            .returning(|_| Err(PersistenceError::NoRowsUpdated));
        let result = withdraw_task(
            Uuid::new_v4(),
            "user-1",
            &repository,
            &FixedTime(Utc::now()),
            &MockAuditLog,
        )
        .await;
        assert_matches!(result, Err(WithdrawTaskError::Storage(_)));
    }
}
