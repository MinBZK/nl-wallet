use std::collections::HashMap;
use std::error::Error;

use assert_matches::assert_matches;
use audit_log::model::FromAuditLogError;
use serde::Serialize;

use audit_log::model::mock::MockAuditLog;
use audit_log_macros::audited;

#[derive(Debug, Serialize)]
struct MyType;

#[derive(Clone, Debug, Serialize)]
struct MyTypeWithReferences<'a, 'b> {
    ref_1: &'a String,
    ref_2: &'b String,
}

#[derive(Debug, thiserror::Error)]
enum MyError {
    #[error("audit error: {0}")]
    Audit(String),
}

impl FromAuditLogError for MyError {
    fn from_audit_log_error(audit_log_error: Box<dyn Error + Send + Sync>) -> Self {
        Self::Audit(audit_log_error.to_string())
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

#[audited]
async fn test_operation_with_references<'a, 'b, 'c>(
    #[audit] one: MyTypeWithReferences<'a, 'b>,
    #[audit] two: MyTypeWithReferences<'b, 'c>,
    #[audit] three: &'b MyTypeWithReferences<'a, 'c>,
    #[audit] four: &'a MyTypeWithReferences<'c, 'b>,
    #[auditor] auditor: &MockAuditLog,
) -> Result<(), MyError> {
    tracing::debug!("performed test operation with referenced types: {one:?}, {two:?}, {three:?}, {four:?}");
    Ok(())
}

type MyMap = HashMap<(String, String), MyType>;

#[audited]
async fn test_operation_invalid_json(#[audit] param: MyMap, #[auditor] auditor: &MockAuditLog) -> Result<(), MyError> {
    tracing::debug!("this should not end up in the logs");
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

#[tokio::test]
#[tracing_test::traced_test]
async fn test_macro_operations_with_references() {
    let audit_log = MockAuditLog;

    let some_string = String::from("some string");
    let another_string = String::from("another string");

    let one = MyTypeWithReferences {
        ref_1: &some_string,
        ref_2: &another_string,
    };

    let two = MyTypeWithReferences {
        ref_1: &another_string,
        ref_2: &some_string,
    };

    test_operation_with_references(one.clone(), two.clone(), &one, &two, &audit_log)
        .await
        .expect("success");

    assert!(logs_contain("performed test operation with referenced types"));
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_macro_operation_invalid_json() {
    let audit_log = MockAuditLog;

    let hashmap = HashMap::from_iter([(("one".to_string(), "two".to_string()), MyType)]);

    let error = test_operation_invalid_json(hashmap, &audit_log)
        .await
        .expect_err("should fail");

    logs_assert(|logs| {
        if !logs.is_empty() {
            Err("logs should be empty".to_string())
        } else {
            Ok(())
        }
    });

    assert_matches!(error, MyError::Audit(error) if error == "key must be a string");
}
