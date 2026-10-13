use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Administrator action that can be proposed and reviewed in the admin_portal.
/// For each of these task types there exists a corresponding privilege. These are named the same as the task types but
/// underscored with privilege_.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString, strum::EnumIter,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    RevokeWallet,
    BlockUser,
    UnblockUser,
    RevokeSolution,
}

/// The lifecycle status of an [`AdminTask`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Created,
    Withdrawn,
    Approved,
    Rejected,
}

/// The outcome a reviewer chooses when reviewing a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdict {
    Approve,
    Reject,
}

impl ReviewVerdict {
    pub fn resulting_status(self) -> TaskStatus {
        match self {
            ReviewVerdict::Approve => TaskStatus::Approved,
            ReviewVerdict::Reject => TaskStatus::Rejected,
        }
    }
}

/// A full admin task row, as stored and returned by the repository.
#[derive(Debug, Clone, PartialEq)]
pub struct AdminTask {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub created_by_user_id: String,
    pub created_by_display_name: String,
    pub task_type: TaskType,
    pub task_parameters: JsonValue,
    pub task_reason: String,
    pub status: TaskStatus,
    pub withdrawn_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by_user_id: Option<String>,
    pub reviewed_by_display_name: Option<String>,
    pub review_comment: Option<String>,
    pub executed_at: Option<DateTime<Utc>>,
    pub execution_result: Option<String>,
    pub execution_result_details: Option<JsonValue>,
}

/// Request model to create a new [`AdminTask`].
/// Will create it with `status = Created` and all optional fields set to `None`.
#[derive(Debug, Clone)]
pub struct AdminTaskCreate {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub created_by_user_id: String,
    pub created_by_display_name: String,
    pub task_type: TaskType,
    pub task_parameters: JsonValue,
    pub task_reason: String,
}

impl From<AdminTaskCreate> for AdminTask {
    fn from(task: AdminTaskCreate) -> Self {
        AdminTask {
            id: task.id,
            created_at: task.created_at,
            created_by_user_id: task.created_by_user_id,
            created_by_display_name: task.created_by_display_name,
            task_type: task.task_type,
            task_parameters: task.task_parameters,
            task_reason: task.task_reason,
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
}

/// Request model for a review verdict, updates [`AdminTask`]
#[derive(Debug, Clone)]
pub struct AdminTaskReview {
    pub id: Uuid,
    pub reviewed_at: DateTime<Utc>,
    pub reviewed_by_user_id: String,
    pub reviewed_by_display_name: String,
    pub review_comment: String,
    pub review_verdict: ReviewVerdict,
}

/// Request model for task lists
#[derive(Debug, Clone)]
pub enum TaskListView {
    Open {
        user_id: String,
    },
    History {
        user_id: String,
    },
    Reviewable {
        user_id: String,
        authorized_task_types: Vec<TaskType>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_verdict_maps_to_the_correct_resulting_status() {
        assert_eq!(ReviewVerdict::Approve.resulting_status(), TaskStatus::Approved);
        assert_eq!(ReviewVerdict::Reject.resulting_status(), TaskStatus::Rejected);
    }
}
