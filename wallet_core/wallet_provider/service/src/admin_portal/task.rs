use std::collections::HashSet;
use std::error::Error;

use audit_log::audited;
use audit_log::model::AuditLog;
use audit_log::model::FromAuditLogError;
use chrono::DateTime;
use chrono::Utc;
use crypto::keys::SecureEcdsaKey;
use token_status_list::status_list_service::StatusListService;
use utils::generator::Generator;
use uuid::Uuid;
use wallet_provider_domain::model::admin_task::AdminTask;
use wallet_provider_domain::model::admin_task::CreateAdminTask;
use wallet_provider_domain::model::admin_task::ExecuteAdminTask;
use wallet_provider_domain::model::admin_task::ReviewAdminTask;
use wallet_provider_domain::model::admin_task::ReviewVerdict;
use wallet_provider_domain::model::admin_task::RevokeWalletParameters;
use wallet_provider_domain::model::admin_task::TaskListView;
use wallet_provider_domain::model::admin_task::TaskPayload;
use wallet_provider_domain::model::admin_task::TaskStatus;
use wallet_provider_domain::model::admin_task::TaskType;
use wallet_provider_domain::model::admin_task::WithdrawAdminTask;
use wallet_provider_domain::model::wallet_user::WalletId;
use wallet_provider_domain::repository::AdminPortalRepository;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::PersistenceError;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletUserRepository;

use super::AdminPortalService;
use super::LoggedInUser;
use crate::account_server::UserState;
use crate::admin_portal::execution::revoke_wallets;

#[derive(Debug, thiserror::Error)]
pub enum RetrieveTaskError {
    #[error("task not found by ID: {0}")]
    NotFound(Uuid),
    #[error("unauthorized to view task")]
    UnAuthorized,
    #[error("persistence error: {0}")]
    Storage(#[source] PersistenceError),
    #[error("error while auditing: {0}")]
    AuditLog(#[source] Box<dyn Error + Send + Sync>),
}

impl FromAuditLogError for RetrieveTaskError {
    fn from_audit_log_error(audit_log_error: Box<dyn Error + Send + Sync>) -> Self {
        Self::AuditLog(audit_log_error)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateTaskError {
    #[error("missing privilege for task type {0}")]
    MissingPrivilege(TaskType),
    #[error("wallet IDs not found: {0:?}")]
    WalletIdsNotFound(HashSet<WalletId>),
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

#[derive(Debug, thiserror::Error)]
pub enum ExecuteTaskError {
    #[error("task not found: {0}")]
    NotFound(Uuid),
    #[error("task execution result cannot be stored: not in 'approved' status, or already executed")]
    InvalidState,
    #[error("persistence error: {0}")]
    Storage(#[source] PersistenceError),
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

impl<R, T, U> AdminPortalService<R, T, U>
where
    R: AdminPortalRepository,
    T: Generator<DateTime<Utc>>,
    U: Generator<Uuid>,
{
    #[audited]
    pub async fn retrieve_task(
        &self,
        #[audit] task_id: Uuid,
        #[audit_user] user: &LoggedInUser,
        #[auditor] audit_log: &impl AuditLog,
    ) -> Result<AdminTask, RetrieveTaskError> {
        let task = self
            .repository
            .find_task_by_id(task_id)
            .await
            .map_err(RetrieveTaskError::Storage)?
            .ok_or(RetrieveTaskError::NotFound(task_id))?;

        if task.created_by_user_id == user.user_id
            || task
                .reviewed_by_user_id
                .as_ref()
                .is_some_and(|reviewer| *reviewer == user.user_id)
            || user.is_authorized_to(&task.payload.task_type())
        {
            Ok(task)
        } else {
            Err(RetrieveTaskError::UnAuthorized)
        }
    }

    #[audited]
    pub async fn create_task<TX, UR, F, H>(
        &self,
        #[audit(flatten)] payload: &TaskPayload,
        #[audit] task_reason: &str,
        #[audit_user] user: &LoggedInUser,
        user_state: &UserState<UR, F, H, impl SecureEcdsaKey, impl StatusListService>,
        #[auditor] audit_log: &impl AuditLog,
    ) -> Result<AdminTask, CreateTaskError>
    where
        TX: Committable,
        UR: TransactionStarter<TransactionType = TX> + WalletUserRepository<TransactionType = TX>,
    {
        let task_type = payload.task_type();
        if !user.is_authorized_to(&task_type) {
            return Err(CreateTaskError::MissingPrivilege(task_type));
        }

        // Verify payload
        match &payload {
            TaskPayload::RevokeWallet(revoke_wallet_parameters) => {
                self.verify_revoke_wallet_parameters(revoke_wallet_parameters, user_state)
                    .await?;
            }
            TaskPayload::BlockUser {} => todo!("PVW-6176"),
            TaskPayload::UnblockUser {} => todo!("PVW-6177"),
            TaskPayload::RevokeSolution {} => todo!("PVW-6178"),
        }

        let task = self
            .repository
            .insert_task(CreateAdminTask {
                id: self.uuid.generate(),
                created_at: self.time.generate(),
                created_by_user_id: user.user_id.clone(),
                created_by_display_name: user.display_name.clone(),
                payload: payload.clone(),
                task_reason: task_reason.to_string(),
            })
            .await
            .map_err(CreateTaskError::Storage)?;

        Ok(task)
    }

    async fn verify_revoke_wallet_parameters<TX, UR, F, H>(
        &self,
        revoke_wallet_parameters: &RevokeWalletParameters,
        user_state: &UserState<UR, F, H, impl SecureEcdsaKey, impl StatusListService>,
    ) -> Result<(), CreateTaskError>
    where
        TX: Committable,
        UR: TransactionStarter<TransactionType = TX> + WalletUserRepository<TransactionType = TX>,
    {
        let transaction = user_state
            .repositories
            .begin_transaction()
            .await
            .map_err(CreateTaskError::Storage)?;

        let wallet_ids: HashSet<WalletId> = revoke_wallet_parameters.wallet_ids.iter().cloned().collect();
        let found_wallet_ids = user_state
            .repositories
            .find_wallet_user_id_by_wallet_ids(&transaction, &wallet_ids)
            .await
            .map_err(CreateTaskError::Storage)?;

        transaction.commit().await.map_err(CreateTaskError::Storage)?;

        if found_wallet_ids.len() != wallet_ids.len() {
            let not_found_ids: HashSet<WalletId> = wallet_ids
                .difference(&found_wallet_ids.into_keys().collect())
                .cloned()
                .collect();
            return Err(CreateTaskError::WalletIdsNotFound(not_found_ids));
        }
        Ok(())
    }

    /// Withdraws a task created by the caller that is still in `Created` status.
    #[audited]
    pub async fn withdraw_task(
        &self,
        #[audit] task_id: Uuid,
        #[audit] withdrawal_comment: &str,
        #[audit_user] user: &LoggedInUser,
        #[auditor] audit_log: &impl AuditLog,
    ) -> Result<AdminTask, WithdrawTaskError> {
        let transaction = self
            .repository
            .begin_transaction()
            .await
            .map_err(WithdrawTaskError::Storage)?;
        let existing = self
            .repository
            .find_task_by_id_for_update(&transaction, task_id)
            .await
            .map_err(WithdrawTaskError::Storage)?
            .ok_or(WithdrawTaskError::NotFound(task_id))?;

        // Enforce business logic rule: A user may only withdraw his/her own tasks
        if existing.created_by_user_id != user.user_id {
            return Err(WithdrawTaskError::InvalidState);
        }

        // Idempotency check, if the task is already withdrawn, return it
        if existing.status == TaskStatus::Withdrawn {
            transaction.commit().await.map_err(WithdrawTaskError::Storage)?;
            return Ok(existing);
        }

        // Actually withdraw the task now, while enforcing that the task is still in the `Created` state, so it will not
        // override another review or withdrawal.
        let withdrawal = WithdrawAdminTask {
            task_id,
            withdrawal_comment: withdrawal_comment.to_string(),
            withdrawn_at: self.time.generate(),
        };
        let task = self
            .repository
            .withdraw_task(&transaction, withdrawal)
            .await
            .map_err(WithdrawTaskError::Storage)?
            .ok_or(WithdrawTaskError::InvalidState)?;

        transaction.commit().await.map_err(WithdrawTaskError::Storage)?;
        Ok(task)
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
        let transaction = self
            .repository
            .begin_transaction()
            .await
            .map_err(ReviewTaskError::Storage)?;
        let existing = self
            .repository
            .find_task_by_id_for_update(&transaction, task_id)
            .await
            .map_err(ReviewTaskError::Storage)?
            .ok_or(ReviewTaskError::NotFound(task_id))?;

        // Enforce business rule: A reviewer may only review tasks of types it has the privilege for.
        let task_type = existing.payload.task_type();
        if !reviewer.is_authorized_to(&task_type) {
            return Err(ReviewTaskError::MissingPrivilege(task_type));
        }

        // Enforce business rule: A reviewer may NOT review its own tasks.
        if existing.created_by_user_id == reviewer.user_id {
            return Err(ReviewTaskError::InvalidState);
        }

        // Idempotency check, if the verdict has already been applied to the task, return it
        if existing.status == verdict.resulting_status()
            && existing.reviewed_by_user_id == Some(reviewer.user_id.clone())
        {
            transaction.commit().await.map_err(ReviewTaskError::Storage)?;
            return Ok(existing);
        }

        // Actually store the review now, while enforcing that the task is still in the `Created~ state, so it will not
        // override another review or withdrawal.
        let review = ReviewAdminTask {
            task_id: existing.id,
            reviewed_at: self.time.generate(),
            reviewed_by_user_id: reviewer.user_id.clone(),
            reviewed_by_display_name: reviewer.display_name.clone(),
            review_comment: review_comment.to_string(),
            review_verdict: verdict,
        };
        let task = self
            .repository
            .review_task(&transaction, review)
            .await
            .map_err(ReviewTaskError::Storage)?
            .ok_or(ReviewTaskError::InvalidState)?;

        transaction.commit().await.map_err(ReviewTaskError::Storage)?;

        Ok(task)
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
                authorized_task_types: user.authorized_task_types().collect(),
            },
        };

        self.repository.list_tasks(view).await.map_err(ListTasksError::Storage)
    }

    /// Executes an approved task and stores the execution result.
    ///
    /// A failed execution is not an error, but is stored as an unsuccessful execution result.
    /// This runs in its own transaction, which locks the task until the execution result is stored.
    /// This executes a task only once, even for concurrent approvals.
    /// Note that this does not audit the execution, that is done per task type at a deeper level.
    pub async fn execute_task<TX, UR, F, H>(
        &self,
        task_id: Uuid,
        approver: &LoggedInUser,
        user_state: &UserState<UR, F, H, impl SecureEcdsaKey, impl StatusListService>,
        audit_log: &impl AuditLog,
    ) -> Result<AdminTask, ExecuteTaskError>
    where
        TX: Committable,
        UR: TransactionStarter<TransactionType = TX> + WalletUserRepository<TransactionType = TX>,
    {
        let transaction = self
            .repository
            .begin_transaction()
            .await
            .map_err(ExecuteTaskError::Storage)?;
        let task = self
            .repository
            .find_task_by_id_for_update(&transaction, task_id)
            .await
            .map_err(ExecuteTaskError::Storage)?
            .ok_or(ExecuteTaskError::NotFound(task_id))?;

        // Only approved tasks without execution result can be executed
        if task.status != TaskStatus::Approved || task.executed_at.is_some() {
            transaction.commit().await.map_err(ExecuteTaskError::Storage)?;
            return Ok(task);
        }

        let (execution_successful, execution_details) = match &task.payload {
            TaskPayload::RevokeWallet(revoke_wallet_parameters) => {
                let wallet_ids = revoke_wallet_parameters.wallet_ids.iter().cloned().collect();
                match revoke_wallets(task.id, &wallet_ids, approver, user_state, &self.time, audit_log).await {
                    Ok(revoked_count) => (true, format!("revoked {revoked_count} wallet(s)")),
                    Err(error) => (false, error.to_string()),
                }
            }
            TaskPayload::BlockUser {} => todo!("PVW-6176"),
            TaskPayload::UnblockUser {} => todo!("PVW-6177"),
            TaskPayload::RevokeSolution {} => todo!("PVW-6178"),
        };

        let execution = ExecuteAdminTask {
            task_id,
            executed_at: self.time.generate(),
            execution_successful,
            execution_details,
        };
        let task = self
            .repository
            .store_execution_result(&transaction, execution)
            .await
            .map_err(ExecuteTaskError::Storage)?
            .ok_or(ExecuteTaskError::InvalidState)?;

        transaction.commit().await.map_err(ExecuteTaskError::Storage)?;
        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use audit_log::model::mock::MockAuditLog;
    use chrono::DateTime;
    use chrono::Utc;
    use crypto::kid::Kid;
    use crypto::trust_anchor::TrustAnchors;
    use hsm::model::mock::MockPkcs11Client;
    use hsm::service::HsmError;
    use p256::ecdsa::SigningKey;
    use rstest::fixture;
    use rstest::rstest;
    use token_status_list::status_list_service::mock::MockStatusListService;
    use utils::vec_nonempty;
    use uuid::Uuid;
    use wallet_provider_domain::model::admin_task::RevokeWalletParameters;
    use wallet_provider_domain::model::admin_task::TaskStatus;
    use wallet_provider_domain::model::admin_task::TaskType;
    use wallet_provider_domain::model::wallet_user::RevokedWalletUsers;
    use wallet_provider_domain::repository::MockTransaction;
    use wallet_provider_persistence::repositories::mock::MockAdminPortalRepository;
    use wallet_provider_persistence::repositories::mock::MockTransactionalWalletUserRepository;

    use super::*;
    use crate::account_server::mock::mock_status_list_service;
    use crate::account_server::mock::user_state;
    use crate::admin_portal::test_support::user;
    use crate::flags::mock::StubWalletFlags;
    use crate::wallet_certificate::mock::setup_hsm;

    const CREATOR: &str = "user-1";
    const REVIEWER: &str = "user-2";
    const TASK_REASON: &str = "because";
    const WITHDRAWAL_COMMENT: &str = "no longer needed";
    const REVIEW_COMMENT: &str = "looks good";

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

    #[fixture]
    fn now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).expect("timestamp should be in range")
    }

    #[fixture]
    fn task_id() -> Uuid {
        Uuid::new_v4()
    }

    #[fixture]
    fn creator() -> LoggedInUser {
        user(CREATOR, &[TaskType::RevokeWallet])
    }

    #[fixture]
    fn reviewer() -> LoggedInUser {
        user(REVIEWER, &[TaskType::RevokeWallet])
    }

    #[fixture]
    fn unprivileged_reviewer() -> LoggedInUser {
        user(REVIEWER, &[TaskType::BlockUser])
    }

    /// Creates the service under test, generating `now` as time and `id` as UUID.
    fn service(
        repository: MockAdminPortalRepository,
        now: DateTime<Utc>,
        id: Uuid,
    ) -> AdminPortalService<MockAdminPortalRepository, FixedTime, FixedUuid> {
        AdminPortalService::new(
            super::super::AdminPortalConfig {
                keycloak_url: "https://keycloak.invalid/".parse().unwrap(),
                keycloak_realm: "test".to_string(),
                keycloak_client_id: "test".to_string(),
                public_url: "https://portal.example.org/".parse().unwrap(),
                session_ttl: std::time::Duration::from_mins(15),
                login_attempt_ttl: std::time::Duration::from_mins(10),
            },
            reqwest::Client::new(),
            repository,
            FixedTime(now),
            FixedUuid(id),
        )
    }

    fn revoke_wallet_payload() -> TaskPayload {
        TaskPayload::RevokeWallet(RevokeWalletParameters {
            wallet_ids: vec_nonempty!["wallet-1".to_string().into()].into(),
        })
    }

    /// The state of a task, as returned by the mocked repository.
    #[derive(Debug, Clone, Copy)]
    enum State {
        Created,
        Withdrawn,
        Approved,
        Rejected,
        Executed,
    }

    /// Returns a revoke wallet task created by [`CREATOR`] in `state`, reviewed by [`REVIEWER`] if applicable.
    fn task_in(state: State, id: Uuid) -> AdminTask {
        let created = AdminTask::from(CreateAdminTask {
            id,
            created_at: DateTime::UNIX_EPOCH,
            created_by_user_id: CREATOR.to_string(),
            created_by_display_name: "User 1".to_string(),
            payload: revoke_wallet_payload(),
            task_reason: TASK_REASON.to_string(),
        });
        let reviewed = |verdict: ReviewVerdict| AdminTask {
            status: verdict.resulting_status(),
            reviewed_at: Some(DateTime::UNIX_EPOCH),
            reviewed_by_user_id: Some(REVIEWER.to_string()),
            reviewed_by_display_name: Some("User 2".to_string()),
            review_comment: Some(REVIEW_COMMENT.to_string()),
            ..created.clone()
        };

        match state {
            State::Created => created,
            State::Withdrawn => AdminTask {
                status: TaskStatus::Withdrawn,
                withdrawal_comment: Some(WITHDRAWAL_COMMENT.to_string()),
                withdrawn_at: Some(DateTime::UNIX_EPOCH),
                ..created
            },
            State::Approved => reviewed(ReviewVerdict::Approve),
            State::Rejected => reviewed(ReviewVerdict::Reject),
            State::Executed => AdminTask {
                executed_at: Some(DateTime::UNIX_EPOCH),
                execution_successful: Some(true),
                execution_details: Some("executed".to_string()),
                ..reviewed(ReviewVerdict::Approve)
            },
        }
    }

    /// Expects one transaction to be started.
    fn expect_transaction(repository: &mut MockAdminPortalRepository) {
        repository
            .expect_begin_transaction()
            .times(1)
            .returning(|| Ok(MockTransaction));
    }

    /// Expects task `id` to be locked once, returning `task`.
    fn expect_find_task(repository: &mut MockAdminPortalRepository, id: Uuid, task: Option<AdminTask>) {
        repository
            .expect_find_task_by_id_for_update()
            .times(1)
            .return_once(move |_, task_id| {
                assert_eq!(task_id, id);
                Ok(task)
            });
    }

    /// Expects the execution result to be stored once, returning the task with that result applied.
    fn expect_store_execution_result(repository: &mut MockAdminPortalRepository, task: AdminTask, now: DateTime<Utc>) {
        repository
            .expect_store_execution_result()
            .times(1)
            .return_once(move |_, execution| {
                assert_eq!(execution.task_id, task.id);
                assert_eq!(execution.executed_at, now);
                Ok(Some(AdminTask {
                    executed_at: Some(execution.executed_at),
                    execution_successful: Some(execution.execution_successful),
                    execution_details: Some(execution.execution_details),
                    ..task
                }))
            });
    }

    /// Wallet user repository that revokes all requested wallet ids that are in `existing_wallet_ids`.
    fn wallet_user_repository(existing_wallet_ids: &'static [&'static str]) -> MockTransactionalWalletUserRepository {
        wallet_user_repository_with_count(existing_wallet_ids, None)
    }

    fn wallet_user_repository_with_count(
        existing_wallet_ids: &'static [&'static str],
        revoked_count: Option<u64>,
    ) -> MockTransactionalWalletUserRepository {
        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_wallet_user_id_by_wallet_ids()
            .returning(move |_, wallet_ids| {
                Ok(wallet_ids
                    .iter()
                    .filter(|wallet_id| existing_wallet_ids.contains(&wallet_id.as_ref().as_str()))
                    .map(|wallet_id| (wallet_id.clone(), Uuid::new_v4()))
                    .collect())
            });
        wallet_user_repo
            .expect_revoke_wallet_users()
            .returning(move |_, wallet_user_ids, _, _| {
                Ok(RevokedWalletUsers {
                    revoked_count: revoked_count.unwrap_or(wallet_user_ids.len() as u64),
                    wia_ids: vec![],
                })
            });
        wallet_user_repo
    }

    /// Builds a [`UserState`] around `wallet_user_repo`, with every other parameter fixed to a test default.
    async fn setup_user_state(
        wallet_user_repo: MockTransactionalWalletUserRepository,
    ) -> UserState<
        MockTransactionalWalletUserRepository,
        StubWalletFlags,
        MockPkcs11Client<HsmError>,
        SigningKey,
        MockStatusListService,
    > {
        user_state(
            wallet_user_repo,
            StubWalletFlags::default(),
            setup_hsm().await,
            Kid::try_from("0").unwrap(),
            TrustAnchors::empty(),
            mock_status_list_service(),
        )
    }

    /// Executes task `task_id` as [`REVIEWER`], with `wallet_user_repo` handling the revocation.
    async fn execute(
        repository: MockAdminPortalRepository,
        wallet_user_repo: MockTransactionalWalletUserRepository,
        now: DateTime<Utc>,
        task_id: Uuid,
    ) -> Result<AdminTask, ExecuteTaskError> {
        let mut user_state = setup_user_state(wallet_user_repo).await;
        user_state
            .wia_status_list
            .expect_revoke_attestation_batches()
            .returning(|_| Ok(()));

        service(repository, now, task_id)
            .execute_task(task_id, &reviewer(), &user_state, &MockAuditLog)
            .await
    }

    #[rstest]
    #[tokio::test]
    async fn create_task_rejects_missing_privilege(
        now: DateTime<Utc>,
        task_id: Uuid,
        #[from(unprivileged_reviewer)] user: LoggedInUser,
    ) {
        let user_state = setup_user_state(wallet_user_repository(&[])).await;

        let result = service(MockAdminPortalRepository::new(), now, task_id)
            .create_task(&revoke_wallet_payload(), TASK_REASON, &user, &user_state, &MockAuditLog)
            .await;

        assert_matches!(result, Err(CreateTaskError::MissingPrivilege(TaskType::RevokeWallet)));
    }

    #[rstest]
    #[tokio::test]
    async fn create_task_inserts_task_with_created_status_when_privileged(
        now: DateTime<Utc>,
        task_id: Uuid,
        creator: LoggedInUser,
    ) {
        let mut repository = MockAdminPortalRepository::new();
        repository.expect_insert_task().times(1).return_once(move |create| {
            assert_eq!(create.id, task_id);
            assert_eq!(create.created_at, now);
            assert_eq!(create.created_by_user_id, CREATOR);
            assert_eq!(create.created_by_display_name, "User 1");
            assert_eq!(create.payload, revoke_wallet_payload());
            assert_eq!(create.task_reason, TASK_REASON);
            Ok(AdminTask::from(create))
        });
        let user_state = setup_user_state(wallet_user_repository(&["wallet-1"])).await;

        let task = service(repository, now, task_id)
            .create_task(
                &revoke_wallet_payload(),
                TASK_REASON,
                &creator,
                &user_state,
                &MockAuditLog,
            )
            .await
            .expect("task should be created");

        assert_eq!(task.id, task_id);
        assert_eq!(task.status, TaskStatus::Created);
    }

    #[rstest]
    #[tokio::test]
    async fn create_task_rejects_unknown_wallet_ids(now: DateTime<Utc>, task_id: Uuid, creator: LoggedInUser) {
        let user_state = setup_user_state(wallet_user_repository(&[])).await;

        let result = service(MockAdminPortalRepository::new(), now, task_id)
            .create_task(
                &revoke_wallet_payload(),
                TASK_REASON,
                &creator,
                &user_state,
                &MockAuditLog,
            )
            .await;

        assert_matches!(
            result,
            Err(CreateTaskError::WalletIdsNotFound(ids)) if ids == HashSet::from([WalletId::from("wallet-1".to_string())])
        );
    }

    #[rstest]
    #[tokio::test]
    async fn withdraw_task_stores_withdrawal_of_created_task(now: DateTime<Utc>, task_id: Uuid, creator: LoggedInUser) {
        let created = task_in(State::Created, task_id);
        let withdrawn = AdminTask {
            withdrawn_at: Some(now),
            ..task_in(State::Withdrawn, task_id)
        };
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        expect_find_task(&mut repository, task_id, Some(created));
        let stored = withdrawn.clone();
        repository
            .expect_withdraw_task()
            .times(1)
            .return_once(move |_, withdrawal| {
                assert_eq!(withdrawal.task_id, task_id);
                assert_eq!(withdrawal.withdrawn_at, now);
                assert_eq!(withdrawal.withdrawal_comment, WITHDRAWAL_COMMENT);
                Ok(Some(stored))
            });

        let task = service(repository, now, task_id)
            .withdraw_task(task_id, WITHDRAWAL_COMMENT, &creator, &MockAuditLog)
            .await
            .expect("task should be withdrawn");

        assert_eq!(task, withdrawn);
    }

    #[rstest]
    #[tokio::test]
    async fn withdraw_task_returns_already_withdrawn_task_without_updating_it(
        now: DateTime<Utc>,
        task_id: Uuid,
        creator: LoggedInUser,
    ) {
        let withdrawn = task_in(State::Withdrawn, task_id);
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        expect_find_task(&mut repository, task_id, Some(withdrawn.clone()));
        repository.expect_withdraw_task().never();

        let task = service(repository, now, task_id)
            .withdraw_task(task_id, WITHDRAWAL_COMMENT, &creator, &MockAuditLog)
            .await
            .expect("already withdrawn task should be returned");

        assert_eq!(task, withdrawn);
    }

    #[rstest]
    #[tokio::test]
    async fn withdraw_task_reports_unknown_task(now: DateTime<Utc>, task_id: Uuid, creator: LoggedInUser) {
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        expect_find_task(&mut repository, task_id, None);

        let result = service(repository, now, task_id)
            .withdraw_task(task_id, WITHDRAWAL_COMMENT, &creator, &MockAuditLog)
            .await;

        assert_matches!(result, Err(WithdrawTaskError::NotFound(id)) if id == task_id);
    }

    #[rstest]
    #[tokio::test]
    async fn withdraw_task_rejects_other_user_than_creator(
        now: DateTime<Utc>,
        task_id: Uuid,
        #[from(reviewer)] other_user: LoggedInUser,
    ) {
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        expect_find_task(&mut repository, task_id, Some(task_in(State::Created, task_id)));

        let result = service(repository, now, task_id)
            .withdraw_task(task_id, WITHDRAWAL_COMMENT, &other_user, &MockAuditLog)
            .await;

        assert_matches!(result, Err(WithdrawTaskError::InvalidState));
    }

    #[rstest]
    #[tokio::test]
    async fn withdraw_task_reports_invalid_state_when_task_changed_after_lookup(
        now: DateTime<Utc>,
        task_id: Uuid,
        creator: LoggedInUser,
    ) {
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        expect_find_task(&mut repository, task_id, Some(task_in(State::Created, task_id)));
        repository.expect_withdraw_task().times(1).return_once(|_, _| Ok(None));

        let result = service(repository, now, task_id)
            .withdraw_task(task_id, WITHDRAWAL_COMMENT, &creator, &MockAuditLog)
            .await;

        assert_matches!(result, Err(WithdrawTaskError::InvalidState));
    }

    #[rstest]
    #[tokio::test]
    async fn withdraw_task_reports_update_failure(now: DateTime<Utc>, task_id: Uuid, creator: LoggedInUser) {
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        expect_find_task(&mut repository, task_id, Some(task_in(State::Created, task_id)));
        repository
            .expect_withdraw_task()
            .times(1)
            .return_once(|_, _| Err(PersistenceError::NoRowsUpdated));

        let result = service(repository, now, task_id)
            .withdraw_task(task_id, WITHDRAWAL_COMMENT, &creator, &MockAuditLog)
            .await;

        assert_matches!(result, Err(WithdrawTaskError::Storage(_)));
    }

    #[rstest]
    #[tokio::test]
    async fn withdraw_task_reports_lookup_failure(now: DateTime<Utc>, task_id: Uuid, creator: LoggedInUser) {
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        repository
            .expect_find_task_by_id_for_update()
            .times(1)
            .return_once(|_, _| Err(PersistenceError::NoRowsUpdated));

        let result = service(repository, now, task_id)
            .withdraw_task(task_id, WITHDRAWAL_COMMENT, &creator, &MockAuditLog)
            .await;

        assert_matches!(result, Err(WithdrawTaskError::Storage(_)));
    }

    #[rstest]
    #[tokio::test]
    async fn withdraw_task_reports_transaction_failure(now: DateTime<Utc>, task_id: Uuid, creator: LoggedInUser) {
        let mut repository = MockAdminPortalRepository::new();
        repository
            .expect_begin_transaction()
            .times(1)
            .return_once(|| Err(PersistenceError::NoRowsUpdated));

        let result = service(repository, now, task_id)
            .withdraw_task(task_id, WITHDRAWAL_COMMENT, &creator, &MockAuditLog)
            .await;

        assert_matches!(result, Err(WithdrawTaskError::Storage(_)));
    }

    #[rstest]
    #[tokio::test]
    async fn review_task_approves_for_a_privileged_non_creator(
        now: DateTime<Utc>,
        task_id: Uuid,
        reviewer: LoggedInUser,
    ) {
        let approved = AdminTask {
            reviewed_at: Some(now),
            ..task_in(State::Approved, task_id)
        };
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        expect_find_task(&mut repository, task_id, Some(task_in(State::Created, task_id)));
        let stored = approved.clone();
        repository.expect_review_task().times(1).return_once(move |_, review| {
            assert_eq!(review.task_id, task_id);
            assert_eq!(review.reviewed_at, now);
            assert_eq!(review.reviewed_by_user_id, REVIEWER);
            assert_eq!(review.reviewed_by_display_name, "User 2");
            assert_eq!(review.review_verdict, ReviewVerdict::Approve);
            assert_eq!(review.review_comment, REVIEW_COMMENT);
            Ok(Some(stored))
        });

        let task = service(repository, now, task_id)
            .review_task(
                task_id,
                ReviewVerdict::Approve,
                REVIEW_COMMENT,
                &reviewer,
                &MockAuditLog,
            )
            .await
            .expect("task should be reviewed");

        assert_eq!(task, approved);
    }

    #[rstest]
    #[case::approved(ReviewVerdict::Approve, State::Approved)]
    #[case::rejected(ReviewVerdict::Reject, State::Rejected)]
    #[tokio::test]
    async fn review_task_returns_already_reviewed_task_without_updating_it(
        now: DateTime<Utc>,
        task_id: Uuid,
        reviewer: LoggedInUser,
        #[case] verdict: ReviewVerdict,
        #[case] state: State,
    ) {
        let reviewed = task_in(state, task_id);
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        expect_find_task(&mut repository, task_id, Some(reviewed.clone()));
        repository.expect_review_task().never();

        let task = service(repository, now, task_id)
            .review_task(task_id, verdict, "replacement comment", &reviewer, &MockAuditLog)
            .await
            .expect("already reviewed task should be returned");

        assert_eq!(task, reviewed);
    }

    #[rstest]
    #[tokio::test]
    async fn review_task_reports_unknown_task(now: DateTime<Utc>, task_id: Uuid, reviewer: LoggedInUser) {
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        expect_find_task(&mut repository, task_id, None);

        let result = service(repository, now, task_id)
            .review_task(
                task_id,
                ReviewVerdict::Approve,
                REVIEW_COMMENT,
                &reviewer,
                &MockAuditLog,
            )
            .await;

        assert_matches!(result, Err(ReviewTaskError::NotFound(id)) if id == task_id);
    }

    #[rstest]
    #[tokio::test]
    async fn review_task_rejects_the_tasks_own_creator(now: DateTime<Utc>, task_id: Uuid, creator: LoggedInUser) {
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        expect_find_task(&mut repository, task_id, Some(task_in(State::Created, task_id)));

        let result = service(repository, now, task_id)
            .review_task(task_id, ReviewVerdict::Approve, REVIEW_COMMENT, &creator, &MockAuditLog)
            .await;

        assert_matches!(result, Err(ReviewTaskError::InvalidState));
    }

    #[rstest]
    #[tokio::test]
    async fn review_task_rejects_a_reviewer_without_privilege(
        now: DateTime<Utc>,
        task_id: Uuid,
        unprivileged_reviewer: LoggedInUser,
    ) {
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        expect_find_task(&mut repository, task_id, Some(task_in(State::Created, task_id)));

        let result = service(repository, now, task_id)
            .review_task(
                task_id,
                ReviewVerdict::Approve,
                REVIEW_COMMENT,
                &unprivileged_reviewer,
                &MockAuditLog,
            )
            .await;

        assert_matches!(result, Err(ReviewTaskError::MissingPrivilege(TaskType::RevokeWallet)));
    }

    #[rstest]
    #[tokio::test]
    async fn review_task_reports_transaction_failure(now: DateTime<Utc>, task_id: Uuid, reviewer: LoggedInUser) {
        let mut repository = MockAdminPortalRepository::new();
        repository
            .expect_begin_transaction()
            .times(1)
            .return_once(|| Err(PersistenceError::NoRowsUpdated));

        let result = service(repository, now, task_id)
            .review_task(
                task_id,
                ReviewVerdict::Approve,
                REVIEW_COMMENT,
                &reviewer,
                &MockAuditLog,
            )
            .await;

        assert_matches!(result, Err(ReviewTaskError::Storage(_)));
    }

    #[rstest]
    #[case::open(TaskListViewKind::Open, &[], TaskListView::Open { user_id: CREATOR.to_string() })]
    #[case::history(TaskListViewKind::History, &[], TaskListView::History { user_id: CREATOR.to_string() })]
    #[case::reviewable(
        TaskListViewKind::Reviewable,
        &[TaskType::RevokeWallet],
        TaskListView::Reviewable { user_id: CREATOR.to_string(), authorized_task_types: vec![TaskType::RevokeWallet] },
    )]
    #[case::reviewable_without_privileges(
        TaskListViewKind::Reviewable,
        &[],
        TaskListView::Reviewable { user_id: CREATOR.to_string(), authorized_task_types: vec![] },
    )]
    #[tokio::test]
    async fn list_tasks_queries_view_for_the_caller(
        now: DateTime<Utc>,
        task_id: Uuid,
        #[case] view_kind: TaskListViewKind,
        #[case] privileges: &[TaskType],
        #[case] expected_view: TaskListView,
    ) {
        let mut repository = MockAdminPortalRepository::new();
        repository.expect_list_tasks().times(1).return_once(move |view| {
            assert_eq!(view, expected_view);
            Ok(vec![])
        });

        service(repository, now, task_id)
            .list_tasks(view_kind, &user(CREATOR, privileges), &MockAuditLog)
            .await
            .expect("should list");
    }

    #[rstest]
    #[case::successful(&["wallet-1"], None, true, "revoked 1 wallet(s)")]
    #[case::already_revoked(&["wallet-1"], Some(0), true, "revoked 0 wallet(s)")]
    #[case::unknown_wallet(&[], None, false, "wallet ID not found")]
    #[tokio::test]
    async fn execute_task_stores_revocation_result(
        now: DateTime<Utc>,
        task_id: Uuid,
        #[case] existing_wallet_ids: &'static [&'static str],
        #[case] revoked_count: Option<u64>,
        #[case] expected_successful: bool,
        #[case] expected_details: &str,
    ) {
        let approved = task_in(State::Approved, task_id);
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        expect_find_task(&mut repository, task_id, Some(approved.clone()));
        expect_store_execution_result(&mut repository, approved, now);

        let user_repo = wallet_user_repository_with_count(existing_wallet_ids, revoked_count);
        let executed = execute(repository, user_repo, now, task_id)
            .await
            .expect("task should be executed");

        assert_eq!(executed.executed_at, Some(now));
        assert_eq!(executed.execution_successful, Some(expected_successful));
        assert!(executed.execution_details.unwrap().contains(expected_details));
    }

    #[rstest]
    #[tokio::test]
    async fn execute_task_skips_task_that_is_not_approved_or_already_executed(
        now: DateTime<Utc>,
        task_id: Uuid,
        #[values(State::Created, State::Withdrawn, State::Rejected, State::Executed)] state: State,
    ) {
        let task = task_in(state, task_id);
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        expect_find_task(&mut repository, task_id, Some(task.clone()));

        // The wallet user repository has no expectations, so any revocation attempt panics.
        let result = execute(repository, MockTransactionalWalletUserRepository::new(), now, task_id)
            .await
            .expect("task should be returned");

        assert_eq!(result, task);
    }

    #[rstest]
    #[tokio::test]
    async fn execute_task_reports_unknown_task(now: DateTime<Utc>, task_id: Uuid) {
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        expect_find_task(&mut repository, task_id, None);

        let result = execute(repository, MockTransactionalWalletUserRepository::new(), now, task_id).await;

        assert_matches!(result, Err(ExecuteTaskError::NotFound(id)) if id == task_id);
    }

    #[rstest]
    #[tokio::test]
    async fn execute_task_reports_invalid_state_when_execution_result_is_not_stored(now: DateTime<Utc>, task_id: Uuid) {
        let mut repository = MockAdminPortalRepository::new();
        expect_transaction(&mut repository);
        expect_find_task(&mut repository, task_id, Some(task_in(State::Approved, task_id)));
        repository
            .expect_store_execution_result()
            .times(1)
            .return_once(|_, _| Ok(None));

        let result = execute(repository, wallet_user_repository(&["wallet-1"]), now, task_id).await;

        assert_matches!(result, Err(ExecuteTaskError::InvalidState));
    }

    async fn retrieve_task_with_lookup_result(
        id: Uuid,
        lookup_result: Option<AdminTask>,
        user: &LoggedInUser,
    ) -> Result<AdminTask, RetrieveTaskError> {
        let mut repository = MockAdminPortalRepository::new();
        repository
            .expect_find_task_by_id()
            .withf(move |task_id| *task_id == id)
            .times(1)
            .return_once(move |_| Ok(lookup_result));

        service(repository, Utc::now(), Uuid::new_v4())
            .retrieve_task(id, user, &MockAuditLog)
            .await
    }

    #[rstest]
    #[case::creator(CREATOR, &[], State::Created)]
    #[case::reviewer(REVIEWER, &[], State::Approved)]
    #[case::privileged_user("user-3", &[TaskType::RevokeWallet], State::Created)]
    #[tokio::test]
    async fn retrieve_task_happy_flow(#[case] user_id: &str, #[case] task_types: &[TaskType], #[case] state: State) {
        let id = Uuid::new_v4();
        let task = task_in(state, id);
        let result = retrieve_task_with_lookup_result(id, Some(task.clone()), &user(user_id, task_types))
            .await
            .unwrap();
        assert_eq!(result, task);
    }

    #[tokio::test]
    async fn retrieve_task_returns_not_found() {
        let id = Uuid::new_v4();
        let result = retrieve_task_with_lookup_result(id, None, &user("user-1", &[])).await;

        assert_matches!(result, Err(RetrieveTaskError::NotFound(task_id)) if task_id == id);
    }

    #[tokio::test]
    async fn retrieve_task_rejects_unrelated_user_without_task_privilege() {
        let id = Uuid::new_v4();
        let task = task_in(State::Created, id);
        let result = retrieve_task_with_lookup_result(id, Some(task), &user("user-3", &[TaskType::BlockUser])).await;

        assert_matches!(result, Err(RetrieveTaskError::UnAuthorized));
    }
}
