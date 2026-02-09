use std::path::Path;
use std::time::Duration;

use assert_matches::assert_matches;
use audit_log::audited;
use chrono::Utc;
use config::Config;
use config::File;
use rstest::rstest;
use sea_orm::ColumnTrait;
use sea_orm::ConnectOptions;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use utils::generator::Generator;
use utils::generator::mock::MockTimeGenerator;
use utils::path::prefix_local_path;

use audit_log::entity;
use audit_log::model::AuditLog;
use audit_log::model::PostgresAuditLog;
use audit_log::model::PostgresAuditLogError;

#[derive(Debug, Clone, Deserialize)]
struct TestSettings {
    pub storage_url: String,
}

fn test_settings() -> TestSettings {
    Config::builder()
        .add_source(File::from(prefix_local_path(Path::new("test_settings.toml")).as_ref()).required(true))
        .build()
        .expect("cannot build config")
        .try_deserialize()
        .expect("cannot read test settings")
}

struct MockUuid(Uuid);

impl Generator<Uuid> for MockUuid {
    fn generate(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, thiserror::Error)]
enum TestError {
    #[error("audit error: {0}")]
    AuditLog(#[from] PostgresAuditLogError),
    #[error("test error")]
    Test,
}

async fn setup_test_database(
    correlation_id: Uuid,
) -> (DatabaseConnection, PostgresAuditLog<MockUuid, MockTimeGenerator>) {
    let database_url = test_settings().storage_url;

    let mut connection_options = ConnectOptions::new(database_url);
    connection_options.connect_timeout(Duration::from_secs(3));
    connection_options.sqlx_logging(true);

    let connection = Database::connect(connection_options)
        .await
        .expect("Failed to connect to test database");

    let now = Utc::now();
    let time_generator = MockTimeGenerator::new(now);

    let audit_log = PostgresAuditLog {
        db_connection: connection.clone(),
        time_generator,
        uuid_generator: MockUuid(correlation_id),
    };

    (connection, audit_log)
}

#[audited]
async fn operation(
    #[auditer] audit_log: &impl AuditLog<Error = PostgresAuditLogError>,
    is_success: bool,
    #[audit] param1: &'static str,
) -> Result<(), TestError> {
    if is_success { Ok(()) } else { Err(TestError::Test) }
}

#[rstest]
#[tokio::test]
async fn test_audit(#[values(true, false)] is_success: bool) {
    // Setup PostgresAuditLog and database connection
    let correlation_id = Uuid::new_v4();
    let (connection, audit_log) = setup_test_database(correlation_id).await;

    // Perform audited test operation
    let result: Result<(), TestError> = operation(&audit_log, is_success, "input").await;

    assert_eq!(result.is_ok(), is_success);

    // Verify the audit records in the database
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
