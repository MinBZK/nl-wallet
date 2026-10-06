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
    Storage(#[from] PersistenceError),
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
    privileges: &[String],
    repository: &R,
    time: &impl Generator<DateTime<Utc>>,
    uuid: &impl Generator<Uuid>,
    #[auditor] audit_log: &impl AuditLog,
) -> Result<AdminTask, CreateTaskError> {
    let parsed_task_type: TaskType = task_type
        .parse()
        .map_err(|_| CreateTaskError::UnknownTaskType(task_type.to_string()))?;

    if !privileges
        .iter()
        .any(|privilege| privilege == &parsed_task_type.to_string())
    {
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
        .await?;

    Ok(task)
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
    Storage(#[from] PersistenceError),
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
    privileges: &[String],
    repository: &R,
    time: &impl Generator<DateTime<Utc>>,
    #[auditor] audit_log: &impl AuditLog,
) -> Result<AdminTask, ReviewTaskError> {
    let existing = repository
        .find_task_by_id(task_id)
        .await?
        .ok_or(ReviewTaskError::NotFound(task_id))?;

    if existing.status != TaskStatus::Created || existing.created_by_user_id == reviewer_user_id {
        return Err(ReviewTaskError::InvalidState);
    }

    if !privileges
        .iter()
        .any(|privilege| privilege == &existing.task_type.to_string())
    {
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
        .await?
        .ok_or(ReviewTaskError::InvalidState)?;

    Ok(updated)
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
            &["revoke_wallet".to_string()],
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
            &["block_user".to_string()],
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
            &["revoke_wallet".to_string()],
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
            &["revoke_wallet".to_string()],
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
            &["revoke_wallet".to_string()],
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
            &["revoke_wallet".to_string()],
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
            &["block_user".to_string()],
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
            &["revoke_wallet".to_string()],
            &repository,
            &FixedTime(now),
            &MockAuditLog,
        )
        .await
        .expect("should review");

        assert_eq!(result.status, TaskStatus::Approved);
    }
}
