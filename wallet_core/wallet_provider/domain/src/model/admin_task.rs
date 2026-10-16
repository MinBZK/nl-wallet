use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value as JsonValue;
use serde_json::json;
use serde_with::serde_as;
use utils::vec_at_least::VecNonEmptyUnique;
use uuid::Uuid;

use super::wallet_user::WalletId;

/// Administrator action that can be proposed and reviewed in the admin_portal, together with its parameters.
///
/// [`TaskType`] is derived from this enum, so every task type has exactly one parameter type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, strum::EnumDiscriminants)]
#[serde(tag = "taskType", content = "taskParameters", rename_all = "snake_case")]
#[strum_discriminants(
    name(TaskType),
    derive(Hash, Serialize, Deserialize, strum::Display, strum::EnumString, strum::EnumIter),
    strum(serialize_all = "snake_case"),
    serde(rename_all = "snake_case")
)]
pub enum TaskPayload {
    RevokeWallet(RevokeWalletParameters),
    BlockUser {},      // TODO PVW-6176
    UnblockUser {},    // TODO PVW-6177
    RevokeSolution {}, // TODO PVW-6178
}

impl TaskPayload {
    pub fn task_type(&self) -> TaskType {
        self.into()
    }

    pub fn parameters_json(&self) -> Result<JsonValue, serde_json::Error> {
        match self {
            Self::RevokeWallet(parameters) => serde_json::to_value(parameters),
            Self::BlockUser {} | Self::UnblockUser {} | Self::RevokeSolution {} => Ok(JsonValue::Object(Map::new())),
        }
    }

    /// Reconstructs a [`TaskPayload`] from a task type and its parameters as JSON, e.g. as stored separately.
    pub fn from_parts(task_type: TaskType, parameters: JsonValue) -> Result<Self, serde_json::Error> {
        let tagged = Map::from_iter([
            ("taskType".to_string(), json!(task_type)),
            ("taskParameters".to_string(), parameters),
        ]);
        serde_json::from_value(JsonValue::Object(tagged))
    }
}

/// Parameters of a [`TaskType::RevokeWallet`] task: the wallets to revoke, at least one, without duplicates.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RevokeWalletParameters {
    #[serde_as(as = "Vec<_>")]
    pub wallet_ids: VecNonEmptyUnique<WalletId>,
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
    pub payload: TaskPayload,
    pub task_reason: String,
    pub status: TaskStatus,
    pub withdrawn_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by_user_id: Option<String>,
    pub reviewed_by_display_name: Option<String>,
    pub review_comment: Option<String>,
    pub executed_at: Option<DateTime<Utc>>,
    pub execution_successful: Option<bool>,
    pub execution_details: Option<String>,
}

/// Request model to create a new [`AdminTask`].
/// Will create it with `status = Created` and all optional fields set to `None`.
#[derive(Debug, Clone)]
pub struct AdminTaskCreate {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub created_by_user_id: String,
    pub created_by_display_name: String,
    pub payload: TaskPayload,
    pub task_reason: String,
}

impl From<AdminTaskCreate> for AdminTask {
    fn from(task: AdminTaskCreate) -> Self {
        AdminTask {
            id: task.id,
            created_at: task.created_at,
            created_by_user_id: task.created_by_user_id,
            created_by_display_name: task.created_by_display_name,
            payload: task.payload,
            task_reason: task.task_reason,
            status: TaskStatus::Created,
            withdrawn_at: None,
            reviewed_at: None,
            reviewed_by_user_id: None,
            reviewed_by_display_name: None,
            review_comment: None,
            executed_at: None,
            execution_successful: None,
            execution_details: None,
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
    use rstest::rstest;

    use super::*;

    fn revoke_wallet_payload(wallet_ids: &[&str]) -> TaskPayload {
        TaskPayload::RevokeWallet(RevokeWalletParameters {
            wallet_ids: wallet_ids
                .iter()
                .map(|wallet_id| WalletId::from(wallet_id.to_string()))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        })
    }

    #[rstest]
    #[case::revoke_wallet(
        json!({"taskType": "revoke_wallet", "taskParameters": {"walletIds": ["wallet-1", "wallet-2"]}}),
        revoke_wallet_payload(&["wallet-1", "wallet-2"]),
    )]
    fn task_payload_round_trips(#[case] value: JsonValue, #[case] payload: TaskPayload) {
        // JSON <-> payload
        assert_eq!(serde_json::from_value::<TaskPayload>(value.clone()).unwrap(), payload);
        assert_eq!(serde_json::to_value(&payload).unwrap(), value);

        // payload <-> (task type, parameters), as stored in the database
        let task_type = payload.task_type();
        let parameters = payload.parameters_json().unwrap();
        assert_eq!(parameters, value["taskParameters"]);
        assert_eq!(TaskPayload::from_parts(task_type, parameters).unwrap(), payload);

        // task type <-> string, as used in privilege names, the database and the API
        let task_type_string = value["taskType"].as_str().unwrap();
        assert_eq!(task_type.to_string(), task_type_string);
        assert_eq!(task_type_string.parse::<TaskType>().unwrap(), task_type);
    }

    #[rstest]
    #[case::empty_wallet_ids(json!({"taskType": "revoke_wallet", "taskParameters": {"walletIds": []}}))]
    #[case::duplicate_wallet_ids(
        json!({"taskType": "revoke_wallet", "taskParameters": {"walletIds": ["wallet-1", "wallet-1"]}})
    )]
    #[case::missing_wallet_ids(json!({"taskType": "revoke_wallet", "taskParameters": {}}))]
    #[case::unknown_parameter(
        json!({"taskType": "revoke_wallet", "taskParameters": {"walletIds": ["wallet-1"], "unknown": true}})
    )]
    #[case::missing_task_parameters(json!({"taskType": "revoke_wallet"}))]
    #[case::unknown_task_type(json!({"taskType": "not_a_real_task_type", "taskParameters": {}}))]
    #[case::missing_task_type(json!({"taskParameters": {}}))]
    fn task_payload_rejects_invalid_json(#[case] value: JsonValue) {
        serde_json::from_value::<TaskPayload>(value).expect_err("should not deserialize");
    }

    #[test]
    fn review_verdict_maps_to_the_correct_resulting_status() {
        assert_eq!(ReviewVerdict::Approve.resulting_status(), TaskStatus::Approved);
        assert_eq!(ReviewVerdict::Reject.resulting_status(), TaskStatus::Rejected);
    }
}
