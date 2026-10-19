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
use wallet_provider_domain::model::admin_task::TaskType;
use wallet_provider_domain::repository::AdminTaskRepository;
use wallet_provider_domain::repository::PersistenceError;

use super::LoggedInUser;

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

/// Creates, withdraws, reviews and lists admin tasks, enforcing the 4-eyes principle. Every action is audit-logged.
pub struct TaskService<R, T, U> {
    repository: R,
    time: T,
    uuid: U,
}

impl<R, T, U> TaskService<R, T, U> {
    pub fn new(repository: R, time: T, uuid: U) -> Self {
        Self { repository, time, uuid }
    }
}

impl<R, T, U> TaskService<R, T, U>
where
    R: AdminTaskRepository,
    T: Generator<DateTime<Utc>>,
    U: Generator<Uuid>,
{
    #[audited]
    pub async fn create_task(
        &self,
        #[audit] task_type: &str,
        #[audit] task_parameters: &JsonValue,
        #[audit] task_reason: &str,
        #[audit_user] user: &LoggedInUser,
        #[auditor] audit_log: &impl AuditLog,
    ) -> Result<AdminTask, CreateTaskError> {
        let parsed_task_type: TaskType = task_type
            .parse()
            .map_err(|_| CreateTaskError::UnknownTaskType(task_type.to_string()))?;

        if !user.authorized_task_types().contains(&parsed_task_type) {
            return Err(CreateTaskError::MissingPrivilege(parsed_task_type));
        }

        let task = self
            .repository
            .insert_task(AdminTaskCreate {
                id: self.uuid.generate(),
                created_at: self.time.generate(),
                created_by_user_id: user.user_id.clone(),
                created_by_display_name: user.display_name.clone(),
                task_type: parsed_task_type,
                task_parameters: task_parameters.clone(),
                task_reason: task_reason.to_string(),
            })
            .await
            .map_err(CreateTaskError::Storage)?;

        Ok(task)
    }

    /// Withdraws a task created by the caller that is still in `Created` status.
    #[audited]
    pub async fn withdraw_task(
        &self,
        #[audit] task_id: Uuid,
        #[audit_user] user: &LoggedInUser,
        #[auditor] audit_log: &impl AuditLog,
    ) -> Result<AdminTask, WithdrawTaskError> {
        let existing = self
            .repository
            .find_task_by_id(task_id)
            .await
            .map_err(WithdrawTaskError::Storage)?
            .ok_or(WithdrawTaskError::NotFound(task_id))?;

        // Enforce business logic rule: A user may only withdraw his/her own tasks
        if existing.created_by_user_id != user.user_id {
            return Err(WithdrawTaskError::InvalidState);
        }

        // Actually withdraw the task now, while enforcing that the task is still in the `Created` state, so it will not
        // override another review or withdrawal.
        self.repository
            .withdraw_task(task_id, self.time.generate())
            .await
            .map_err(WithdrawTaskError::Storage)?
            .ok_or(WithdrawTaskError::InvalidState)
    }

    /// Review a task created by another user that is still in `Created` status.
    #[audited]
    pub async fn review_task(
        &self,
        #[audit] task_id: Uuid,
        #[audit] verdict: ReviewVerdict,
        #[audit] review_comment: &str,
        #[audit_user] reviewer: &LoggedInUser,
        #[auditor] audit_log: &impl AuditLog,
    ) -> Result<AdminTask, ReviewTaskError> {
        let existing = self
            .repository
            .find_task_by_id(task_id)
            .await
            .map_err(ReviewTaskError::Storage)?
            .ok_or(ReviewTaskError::NotFound(task_id))?;

        // Enforce business rule: A reviewer may only review tasks of types it has the privilege for.
        if !reviewer.authorized_task_types().contains(&existing.task_type) {
            return Err(ReviewTaskError::MissingPrivilege(existing.task_type));
        }

        // Enforce business rule: A reviewer may NOT review its own tasks.
        if existing.created_by_user_id == reviewer.user_id {
            return Err(ReviewTaskError::InvalidState);
        }

        // Actually store the review now, while enforcing that the task is still in the `Created~ state, so it will not
        // override another review or withdrawal.
        let review = AdminTaskReview {
            id: existing.id,
            reviewed_at: self.time.generate(),
            reviewed_by_user_id: reviewer.user_id.clone(),
            reviewed_by_display_name: reviewer.display_name.clone(),
            review_comment: review_comment.to_string(),
            review_verdict: verdict,
        };
        self.repository
            .review_task(review)
            .await
            .map_err(ReviewTaskError::Storage)?
            .ok_or(ReviewTaskError::InvalidState)
    }

    #[audited]
    pub async fn list_tasks(
        &self,
        #[audit] view_kind: TaskListViewKind,
        #[audit_user] user: &LoggedInUser,
        #[auditor] audit_log: &impl AuditLog,
    ) -> Result<Vec<AdminTask>, ListTasksError> {
        let user_id = user.user_id.clone();

        let view = match view_kind {
            TaskListViewKind::Open => TaskListView::Open { user_id },
            TaskListViewKind::History => TaskListView::History { user_id },
            TaskListViewKind::Reviewable => TaskListView::Reviewable {
                user_id,
                authorized_task_types: user.authorized_task_types().into_iter().collect(),
            },
        };

        self.repository.list_tasks(view).await.map_err(ListTasksError::Storage)
    }
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

    fn user(user_id: &str, task_types: &[TaskType]) -> LoggedInUser {
        LoggedInUser {
            user_id: user_id.to_string(),
            display_name: user_id.replace("user-", "User "),
            privileges: task_types.iter().map(TaskType::to_string).collect(),
        }
    }

    fn service(
        repository: MockAdminTaskRepository,
        now: DateTime<Utc>,
    ) -> TaskService<MockAdminTaskRepository, FixedTime, FixedUuid> {
        service_with_id(repository, now, Uuid::new_v4())
    }

    fn service_with_id(
        repository: MockAdminTaskRepository,
        now: DateTime<Utc>,
        id: Uuid,
    ) -> TaskService<MockAdminTaskRepository, FixedTime, FixedUuid> {
        TaskService::new(repository, FixedTime(now), FixedUuid(id))
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

        let result = service_with_id(repository, Utc::now(), Uuid::new_v4())
            .create_task(
                "not_a_real_task_type",
                &json!({}),
                "reason",
                &user("user-1", &[TaskType::RevokeWallet]),
                &MockAuditLog,
            )
            .await;

        assert_matches!(result, Err(CreateTaskError::UnknownTaskType(t)) if t == "not_a_real_task_type");
    }

    #[tokio::test]
    async fn create_task_rejects_missing_privilege() {
        let repository = MockAdminTaskRepository::new();

        let result = service_with_id(repository, Utc::now(), Uuid::new_v4())
            .create_task(
                "revoke_wallet",
                &json!({}),
                "reason",
                &user("user-1", &[TaskType::BlockUser]),
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
                    && task.created_by_display_name == "User 1"
                    && task.task_type == TaskType::RevokeWallet
                    && task.task_reason == "because"
            })
            .returning(move |create| Ok(stub_task(create.id, create.task_type, &create.created_by_user_id)));

        let result = service_with_id(repository, now, id)
            .create_task(
                "revoke_wallet",
                &json!({}),
                "because",
                &user("user-1", &[TaskType::RevokeWallet]),
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

        let result = service(repository, Utc::now())
            .review_task(
                id,
                ReviewVerdict::Approve,
                "comment",
                &user("user-2", &[TaskType::RevokeWallet]),
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

        let result = service(repository, Utc::now())
            .review_task(
                id,
                ReviewVerdict::Approve,
                "comment",
                &user("user-1", &[TaskType::RevokeWallet]),
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

        let result = service(repository, Utc::now())
            .review_task(
                id,
                ReviewVerdict::Approve,
                "comment",
                &user("user-2", &[TaskType::BlockUser]),
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
                    && review.reviewed_by_display_name == "User 2"
                    && review.review_verdict == ReviewVerdict::Approve
                    && review.review_comment == "looks good"
            })
            .returning(move |_| {
                let mut task = stub_task(id, TaskType::RevokeWallet, "user-1");
                task.status = TaskStatus::Approved;
                Ok(Some(task))
            });

        let result = service(repository, now)
            .review_task(
                id,
                ReviewVerdict::Approve,
                "looks good",
                &user("user-2", &[TaskType::RevokeWallet]),
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

        service(repository, Utc::now())
            .list_tasks(TaskListViewKind::Open, &user("user-1", &[]), &MockAuditLog)
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

        service(repository, Utc::now())
            .list_tasks(TaskListViewKind::History, &user("user-1", &[]), &MockAuditLog)
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

        service(repository, Utc::now())
            .list_tasks(
                TaskListViewKind::Reviewable,
                &user("user-1", &[TaskType::RevokeWallet]),
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

        service(repository, Utc::now())
            .list_tasks(TaskListViewKind::Reviewable, &user("user-1", &[]), &MockAuditLog)
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
        let result = service(repository, Utc::now())
            .withdraw_task(id, &user("user-1", &[]), &MockAuditLog)
            .await;
        assert_matches!(result, Err(WithdrawTaskError::NotFound(task_id)) if task_id == id);
    }

    #[tokio::test]
    async fn withdraw_task_rejects_invalid_user() {
        let id = Uuid::new_v4();
        let task = stub_task(id, TaskType::RevokeWallet, "user-1");

        let mut repository = MockAdminTaskRepository::new();
        repository
            .expect_find_task_by_id()
            .times(1)
            .returning(move |_| Ok(Some(task.clone())));

        let result = service(repository, Utc::now())
            .withdraw_task(id, &user("another-user", &[]), &MockAuditLog)
            .await;
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
            .withf(move |task_id, timestamp| *task_id == id && *timestamp == now)
            .times(1)
            .returning(move |_, _| {
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
        let result = service(repository, now)
            .withdraw_task(id, &user("user-1", &[]), &MockAuditLog)
            .await;
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
        let result = service(repository, Utc::now())
            .withdraw_task(Uuid::new_v4(), &user("user-1", &[]), &MockAuditLog)
            .await;
        assert_matches!(result, Err(WithdrawTaskError::Storage(_)));
    }
}
