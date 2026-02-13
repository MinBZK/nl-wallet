use std::error::Error;

use audit_log::model::FromAuditLogError;
use serde::Serialize;

use audit_log::model::mock::MockAuditLog;
use audit_log_macros::audited;

#[derive(Debug, Serialize)]
struct MyType;

#[derive(Debug, thiserror::Error)]
enum MyError {
    #[error("audit error: {0}")]
    Audit(Box<dyn Error + Send + Sync>),
}

impl FromAuditLogError for MyError {
    fn from_audit_log_error(audit_log_error: Box<dyn Error + Send + Sync>) -> Self {
        Self::Audit(audit_log_error)
    }
}

#[audited]
async fn test_operation<'a, 'b>(
    #[audit] string_input: String,
    #[audit] str_input: &'a str,
    #[audit] my_type_input: MyType,
    #[audit] my_type_ref_input: &'b MyType,
    #[auditor] auditor: &MockAuditLog,
    _ignored: (),
) -> Result<(), MyError> {
    tracing::debug!(
        "performed test operation with input: {string_input}, {str_input}, {my_type_input:?}, {my_type_ref_input:?}"
    );
    Ok(())
}

#[audited]
async fn test_no_audit_params(#[auditor] auditor: &MockAuditLog) -> Result<(), MyError> {
    tracing::debug!("performed operation without audit params");
    Ok(())
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_macro_no_audit_params() {
    let audit_log = MockAuditLog;

    test_no_audit_params(&audit_log).await.expect("success");

    assert!(logs_contain("performed operation without audit params"));
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_macro() {
    let audit_log = MockAuditLog;

    test_operation("string_input".to_string(), "str_input", MyType, &MyType, &audit_log, ())
        .await
        .expect("success");

    assert!(logs_contain(
        "performed test operation with input: string_input, str_input, MyType, MyType"
    ));
}
