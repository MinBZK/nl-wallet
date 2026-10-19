use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

use super::errors::PersistenceError;
use crate::model::admin_task::AdminTask;
use crate::model::admin_task::AdminTaskCreate;
use crate::model::admin_task::AdminTaskReview;
use crate::model::admin_task::TaskListView;

type Result<T> = std::result::Result<T, PersistenceError>;

/// Persists [`AdminTask`]s, the 4-eyes task-proposal/review framework's core data.
#[trait_variant::make(Send)]
pub trait AdminTaskRepository {
    async fn insert_task(&self, task: AdminTaskCreate) -> Result<AdminTask>;

    async fn find_task_by_id(&self, task_id: Uuid) -> Result<Option<AdminTask>>;

    /// Withdraws the task, provided it was still in `Created` status. Returns `None` if no row matched (not found or
    /// wrong status). The caller is expected to enforce business logic rules, like whose may withdraw whose tasks.
    async fn withdraw_task(&self, task_id: Uuid, now: DateTime<Utc>) -> Result<Option<AdminTask>>;

    /// Reviews the task, provided it is still in `Created` status. Returns `None` if no row matched. The caller is
    /// expected to enforce business rules, like a user may not review his/her own tasks.
    async fn review_task(&self, review: AdminTaskReview) -> Result<Option<AdminTask>>;

    /// Returns tasks matching `view`, ordered by `created_at` ascending (oldest first).
    async fn list_tasks(&self, view: TaskListView) -> Result<Vec<AdminTask>>;
}
