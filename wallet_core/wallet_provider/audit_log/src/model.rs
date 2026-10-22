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

/// Provides the user ID for auditing.
pub trait AuditUser {
    fn to_audit_user_id(&self) -> String;
}

impl<T: AuditUser> AuditUser for &T {
    fn to_audit_user_id(&self) -> String {
        T::to_audit_user_id(self)
    }
}

pub trait FromAuditLogError {
    fn from_audit_log_error(audit_log_error: Box<dyn Error + Send + Sync>) -> Self;
}

pub trait AuditLog {
    type Error: Error + Send + Sync;

    async fn audit<F, T, E>(
        &self,
        operation_name: impl Into<String>,
        parameters: JsonValue,
        user_id: Option<String>,
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
        user_id: Option<String>,
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
                user_id,
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
    use std::fmt::Display;

    use sea_orm::JsonValue;

    use super::AuditLog;
    use super::AuditUser;

    pub struct MockAuditLog;

    pub struct MockAuditUser(String);

    impl Display for MockAuditUser {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl From<&str> for MockAuditUser {
        fn from(user: &str) -> Self {
            Self(user.to_string())
        }
    }

    impl AuditUser for MockAuditUser {
        fn to_audit_user_id(&self) -> String {
            self.0.clone()
        }
    }

    impl AuditLog for MockAuditLog {
        type Error = Infallible;

        async fn audit<F, T, E>(
            &self,
            _operation_name: impl Into<String>,
            _parameters: JsonValue,
            _user_id: Option<String>,
            operation: F,
        ) -> Result<T, E>
        where
            F: AsyncFnOnce() -> Result<T, E>,
        {
            operation().await
        }
    }
}
