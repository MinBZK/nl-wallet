use std::error::Error;
use std::future::Future;

use chrono::DateTime;
use chrono::Utc;
use sea_orm::ActiveModelTrait;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
use sea_orm::JsonValue;
use sea_orm::Set;
use sea_orm::prelude::DateTimeWithTimeZone;
use uuid::Uuid;

use crate::entity::audit_log;

pub trait AuditLogRepository {
    type Error: Error + Send + Sync + 'static;

    fn audit_operation_start(
        &self,
        timestamp: DateTime<Utc>,
        operation_name: String,
        parameters: JsonValue,
        user_id: Option<String>,
        correlation_id: Uuid,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn audit_operation_result(
        &self,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
        is_success: bool,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

#[derive(Clone)]
pub struct PostgresAuditLogRepository {
    pub db_connection: DatabaseConnection,
}

impl AuditLogRepository for PostgresAuditLogRepository {
    type Error = DbErr;
    async fn audit_operation_start(
        &self,
        timestamp: DateTime<Utc>,
        operation_name: String,
        parameters: JsonValue,
        user_id: Option<String>,
        correlation_id: Uuid,
    ) -> Result<(), DbErr> {
        let timestamp: DateTimeWithTimeZone = timestamp.into();

        let audit_start = audit_log::ActiveModel {
            correlation_id: Set(correlation_id),
            timestamp: Set(timestamp),
            operation: Set(Some(operation_name)),
            params: Set(Some(parameters.clone())),
            user_id: Set(user_id),
            ..Default::default()
        };

        audit_start.insert(&self.db_connection).await.map_err(|error| {
            // Log the audit error
            tracing::debug!(
                audit_timestamp = %timestamp,
                audit_parameters = %parameters,
                "error while auditing start of operation: {error}",
            );
            error
        })?;

        Ok(())
    }

    async fn audit_operation_result(
        &self,
        timestamp: DateTime<Utc>,
        correlation_id: Uuid,
        is_success: bool,
    ) -> Result<(), DbErr> {
        let timestamp: DateTimeWithTimeZone = timestamp.into();

        let audit_result = audit_log::ActiveModel {
            correlation_id: Set(correlation_id),
            timestamp: Set(timestamp),
            is_success: Set(Some(is_success)),
            ..Default::default()
        };

        audit_result.insert(&self.db_connection).await.map_err(|error| {
            // Log the audit error
            tracing::debug!(
                audit_timestamp = %timestamp,
                audit_operation_result = is_success,
                "error while auditing result of operation: {error}",
            );
            error
        })?;

        Ok(())
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use std::convert::Infallible;

    use chrono::DateTime;
    use chrono::Utc;
    use sea_orm::JsonValue;
    use uuid::Uuid;

    use super::AuditLogRepository;

    pub struct NoopAuditLogRepository;

    impl AuditLogRepository for NoopAuditLogRepository {
        type Error = Infallible;

        async fn audit_operation_start(
            &self,
            _timestamp: DateTime<Utc>,
            _operation_name: String,
            _parameters: JsonValue,
            _user_id: Option<String>,
            _correlation_id: Uuid,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn audit_operation_result(
            &self,
            _timestamp: DateTime<Utc>,
            _correlation_id: Uuid,
            _is_success: bool,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }
}
