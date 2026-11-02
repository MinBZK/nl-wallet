use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

use super::errors::PersistenceError;
use super::transaction::TransactionStarter;
use crate::model::admin_portal_session::AdminPortalLoginAttempt;
use crate::model::admin_portal_session::AdminPortalUserSession;
use crate::model::admin_task::AdminTask;
use crate::model::admin_task::CreateAdminTask;
use crate::model::admin_task::ExecuteAdminTask;
use crate::model::admin_task::ReviewAdminTask;
use crate::model::admin_task::TaskListView;
use crate::model::admin_task::WithdrawAdminTask;

type Result<T> = std::result::Result<T, PersistenceError>;

/// Persists the Admin Portal's pending logins, sessions, and proposed/reviewed tasks.
#[trait_variant::make(Send)]
pub trait AdminPortalRepository: TransactionStarter {
    async fn insert_login_attempt(&self, state: String, login_attempt: AdminPortalLoginAttempt) -> Result<()>;

    /// Removes and returns the login attempt identified by `state`, if present.
    async fn take_login_attempt(&self, state: &str) -> Result<Option<AdminPortalLoginAttempt>>;

    /// Deletes login attempts created before `created_before`.
    async fn cleanup_expired_login_attempts(&self, created_before: DateTime<Utc>) -> Result<()>;

    async fn insert_user_session(&self, session_id: String, session: AdminPortalUserSession) -> Result<()>;

    /// Fetches the users session if not already expired and update if provided with a new expiry time.
    ///
    /// Returns `None` if no valid session was found.
    async fn fetch_user_session(
        &self,
        session_id: &str,
        now: DateTime<Utc>,
        new_expires_at: Option<DateTime<Utc>>,
    ) -> Result<Option<AdminPortalUserSession>>;

    /// Removes and returns the session identified by `session_id`, if present.
    async fn take_user_session(&self, session_id: &str) -> Result<Option<AdminPortalUserSession>>;

    /// Deletes sessions that expired at or before `now`.
    async fn cleanup_expired_user_sessions(&self, now: DateTime<Utc>) -> Result<()>;

    async fn insert_task(&self, task: CreateAdminTask) -> Result<AdminTask>;

    async fn find_task_by_id(&self, task_id: Uuid) -> Result<Option<AdminTask>>;

    /// Locks the task exclusively until the transaction is committed or rolled back.
    /// Use the same transaction for the subsequent withdrawal or review.
    async fn find_task_by_id_for_update(
        &self,
        transaction: &Self::TransactionType,
        task_id: Uuid,
    ) -> Result<Option<AdminTask>>;

    /// Withdraws the task, provided it was still in `Created` status. Returns `None` if no row matched (not found or
    /// wrong status). The caller is expected to enforce business logic rules, like whose may withdraw whose tasks.
    async fn withdraw_task(
        &self,
        transaction: &Self::TransactionType,
        withdrawal: WithdrawAdminTask,
    ) -> Result<Option<AdminTask>>;

    /// Reviews the task, provided it is still in `Created` status. Returns `None` if no row matched. The caller is
    /// expected to enforce business rules, like a user may not review his/her own tasks.
    async fn review_task(
        &self,
        transaction: &Self::TransactionType,
        review: ReviewAdminTask,
    ) -> Result<Option<AdminTask>>;

    /// Stores the execution result of the task, provided it is in `Approved` status and was not executed before.
    /// Returns `None` if no row matched.
    async fn store_execution_result(
        &self,
        transaction: &Self::TransactionType,
        execution: ExecuteAdminTask,
    ) -> Result<Option<AdminTask>>;

    /// Returns tasks matching `view`, ordered by `created_at` ascending (oldest first).
    async fn list_tasks(&self, view: TaskListView) -> Result<Vec<AdminTask>>;
}
