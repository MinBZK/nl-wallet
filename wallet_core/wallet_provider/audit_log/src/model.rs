use std::error::Error;

use chrono::DateTime;
use chrono::Utc;
use sea_orm::JsonValue;
use tracing::Level;
use tracing::span;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use uuid::Uuid;

use crate::repository::AuditLogRepository;

pub trait FromAuditLogError {
    fn from_audit_log_error(audit_log_error: Box<dyn Error + Send + Sync>) -> Self;
}

pub trait AuditLog {
    type Error: Error + Send + Sync;

    async fn audit<F, T, E>(
        &self,
        operation_name: impl Into<String>,
        parameters: JsonValue,
        user_id: Option<impl Into<String>>,
        operation: F,
    ) -> Result<T, E>
    where
        F: AsyncFnOnce() -> Result<T, E>,
        E: FromAuditLogError;
}

#[derive(Clone)]
pub struct DbAuditLog<R, UG, TG = TimeGenerator> {
    pub repository: R,
    pub time_generator: TG,
    pub uuid_generator: UG,
}

impl<R, UG, TG> AuditLog for DbAuditLog<R, UG, TG>
where
    R: AuditLogRepository,
    TG: Generator<DateTime<Utc>>,
    UG: Generator<Uuid>,
{
    type Error = R::Error;

    /// Audit operation.
    ///
    /// This adds two records to the audit log:
    /// - before executing the `operation`, it records the operation with the parameters and the optional `user_id`
    /// - after executing the `operation`, it records the result of the operation, i.e. Success or Failure
    async fn audit<F, T, E>(
        &self,
        operation_name: impl Into<String>,
        parameters: JsonValue,
        user_id: Option<impl Into<String>>,
        operation: F,
    ) -> Result<T, E>
    where
        F: AsyncFnOnce() -> Result<T, E>,
        E: FromAuditLogError,
    {
        let correlation_id: Uuid = self.uuid_generator.generate();
        let operation_name = operation_name.into();

        // Retain span so it is active during the rest of this function
        let _span = span!(
            Level::DEBUG,
            "audit",
            audit_operation = operation_name,
            audit_correlation_id = correlation_id.to_string()
        );

        self.repository
            .audit_operation_start(
                self.time_generator.generate(),
                operation_name,
                parameters,
                user_id.map(Into::into),
                correlation_id,
            )
            .await
            .map_err(|e| E::from_audit_log_error(Box::new(e)))?;

        let result = operation().await;

        self.repository
            .audit_operation_result(self.time_generator.generate(), correlation_id, result.is_ok())
            .await
            .map_err(|e| E::from_audit_log_error(Box::new(e)))?;

        result
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::convert::Infallible;

    use sea_orm::JsonValue;

    use super::AuditLog;

    pub struct MockAuditLog;

    impl AuditLog for MockAuditLog {
        type Error = Infallible;

        async fn audit<F, T, E>(
            &self,
            _operation_name: impl Into<String>,
            _parameters: JsonValue,
            _user_id: Option<impl Into<String>>,
            operation: F,
        ) -> Result<T, E>
        where
            F: AsyncFnOnce() -> Result<T, E>,
        {
            operation().await
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::error::Error;
    use std::sync::Mutex;

    use chrono::DateTime;
    use chrono::Utc;
    use sea_orm::JsonValue;
    use serde_json::json;
    use utils::generator::TimeGenerator;
    use utils::generator::UuidV4AndTimeGenerator;
    use uuid::Uuid;

    use super::AuditLog;
    use super::DbAuditLog;
    use super::FromAuditLogError;
    use crate::repository::AuditLogRepository;

    #[derive(Debug, thiserror::Error)]
    #[error("test error")]
    pub struct MockError;

    impl FromAuditLogError for MockError {
        fn from_audit_log_error(_: Box<dyn Error + Send + Sync>) -> Self {
            Self
        }
    }

    #[derive(Default)]
    pub struct MockAuditLogRepository {
        pub events: Mutex<Vec<(Uuid, Option<bool>)>>,
        pub fail_start: bool,
        pub fail_result: bool,
    }

    impl AuditLogRepository for MockAuditLogRepository {
        type Error = MockError;

        async fn audit_operation_start(
            &self,
            _: DateTime<Utc>,
            operation_name: String,
            parameters: JsonValue,
            user_id: Option<String>,
            correlation_id: Uuid,
        ) -> Result<(), Self::Error> {
            assert_eq!(operation_name, "test");
            assert_eq!(parameters, json!({"input": 42}));
            assert_eq!(user_id.as_deref(), Some("user"));
            self.events.lock().unwrap().push((correlation_id, None));
            if self.fail_start { Err(MockError) } else { Ok(()) }
        }

        async fn audit_operation_result(
            &self,
            _: DateTime<Utc>,
            correlation_id: Uuid,
            is_success: bool,
        ) -> Result<(), Self::Error> {
            self.events.lock().unwrap().push((correlation_id, Some(is_success)));
            if self.fail_result { Err(MockError) } else { Ok(()) }
        }
    }

    #[rstest::rstest]
    #[tokio::test]
    async fn test_repository_audit(
        #[values(true, false)] operation_success: bool,
        #[values(true, false)] fail_start: bool,
        #[values(true, false)] fail_result: bool,
    ) {
        let audit_log = DbAuditLog {
            repository: MockAuditLogRepository {
                events: Default::default(),
                fail_start,
                fail_result,
            },
            time_generator: TimeGenerator,
            uuid_generator: UuidV4AndTimeGenerator,
        };
        let called = RefCell::new(false);
        let result = audit_log
            .audit("test", json!({"input": 42}), Some("user"), async || {
                assert_eq!(audit_log.repository.events.lock().unwrap().len(), 1);
                *called.borrow_mut() = true;
                if operation_success { Ok(42) } else { Err(MockError) }
            })
            .await;

        assert_eq!(
            result.ok(),
            (operation_success && !fail_start && !fail_result).then_some(42)
        );
        assert_eq!(*called.borrow(), !fail_start);
        let events = audit_log.repository.events.lock().unwrap();
        assert_eq!(events.len(), if fail_start { 1 } else { 2 });
        if !fail_start {
            assert_eq!(events[0].0, events[1].0);
            assert_eq!(events[1].1, Some(operation_success));
        }
    }
}
