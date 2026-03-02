use std::error::Error;

use assert_matches::assert_matches;
use audit_log::audited;
use audit_log::model::FromAuditLogError;
use chrono::Utc;
use rstest::rstest;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use serde_json::json;
use uuid::Uuid;

use utils::generator::Generator;
use utils::generator::mock::MockTimeGenerator;

use audit_log::entity;
use audit_log::model::AuditLog;
use audit_log::model::PostgresAuditLog;

use db_test::DbSetup;
use db_test::connection_from_url;

struct MockUuid(Uuid);

impl Generator<Uuid> for MockUuid {
    fn generate(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, thiserror::Error)]
enum TestError {
    #[error("audit error: {0}")]
    AuditLog(#[source] Box<dyn Error + Send + Sync>),
    #[error("test error")]
    Test,
}

impl FromAuditLogError for TestError {
    fn from_audit_log_error(audit_log_error: Box<dyn Error + Send + Sync>) -> Self {
        TestError::AuditLog(audit_log_error)
    }
}

async fn setup_test_database(correlation_id: Uuid) -> (DbSetup, PostgresAuditLog<MockUuid, MockTimeGenerator>) {
    let db_setup = DbSetup::create().await;

    let now = Utc::now();
    let time_generator = MockTimeGenerator::new(now);

    let audit_log = PostgresAuditLog {
        db_connection: connection_from_url(db_setup.audit_log_url()).await,
        time_generator,
        uuid_generator: MockUuid(correlation_id),
    };

    (db_setup, audit_log)
}

#[audited]
async fn operation(
    #[auditor] audit_log: &impl AuditLog,
    is_success: bool,
    #[audit] param1: &'static str,
) -> Result<(), TestError> {
    if is_success { Ok(()) } else { Err(TestError::Test) }
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_audit(#[values(true, false)] is_success: bool) {
    // Setup PostgresAuditLog and database connection
    let correlation_id = Uuid::new_v4();
    let (db_setup, audit_log) = setup_test_database(correlation_id).await;

    // Perform audited test operation
    let result: Result<(), TestError> = operation(&audit_log, is_success, "input").await;

    assert_eq!(result.is_ok(), is_success);

    // Verify the audit records in the database
    let connection = connection_from_url(db_setup.audit_log_url()).await;
    let audit_records = entity::audit_log::Entity::find()
        .filter(entity::audit_log::Column::CorrelationId.eq(correlation_id))
        .order_by_asc(entity::audit_log::Column::Id)
        .all(&connection)
        .await
        .expect("Failed to query audit records");
    assert_eq!(audit_records.len(), 2);

    // Check the first record (operation start)
    let start_record = &audit_records[0];
    assert_eq!(start_record.correlation_id, correlation_id);
    assert_matches!(start_record.operation.as_ref(), Some(operation) if operation == "operation");
    assert_matches!(start_record.params.as_ref(), Some(params) if *params == json!({"param1": "input"}));
    assert!(start_record.is_success.is_none());

    // Check the second record (operation result)
    let result_record = &audit_records[1];
    assert_eq!(result_record.correlation_id, correlation_id);
    assert!(result_record.operation.is_none());
    assert!(result_record.params.is_none());
    assert_eq!(result_record.is_success, Some(is_success));
}
