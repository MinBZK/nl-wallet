use std::fmt::Display;

use chrono::DateTime;
use derive_more::Display;
use sea_orm::ActiveModelTrait;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
use sea_orm::JsonValue;
use sea_orm::Set;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sqlx::types::chrono::Utc;
use tracing::Level;
use tracing::span;
use uuid::Uuid;

use utils::generator::Generator;
use utils::generator::TimeGenerator;

use crate::entity::audit_log;

pub trait AuditLog {
    type Error;

    async fn audit<F, T, E>(
        &self,
        operation_name: impl Into<String>,
        parameters: JsonValue,
        operation: F,
    ) -> Result<T, E>
    where
        F: AsyncFnOnce() -> Result<T, E>,
        E: From<Self::Error> + Display;
}

pub struct PostgresAuditLog<UG, TG = TimeGenerator> {
    pub db_connection: DatabaseConnection,
    pub time_generator: TG,
    pub uuid_generator: UG,
}

#[derive(Debug, Display, thiserror::Error)]
pub struct PostgresAuditLogError(#[from] DbErr);

impl<UG, TG> AuditLog for PostgresAuditLog<UG, TG>
where
    TG: Generator<DateTime<Utc>>,
    UG: Generator<Uuid>,
{
    type Error = PostgresAuditLogError;

    /// Audit operation.
    ///
    /// This adds two records to the audit log:
    /// - before executing the `operation`, it records the operation with the parameters
    /// - after executing the `operation`, it records the result of the operation, i.e. Success or Failure
    async fn audit<F, T, E>(
        &self,
        operation_name: impl Into<String>,
        parameters: JsonValue,
        operation: F,
    ) -> Result<T, E>
    where
        F: AsyncFnOnce() -> Result<T, E>,
        E: From<PostgresAuditLogError> + Display,
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

        self.audit_operation_start(operation_name, parameters, correlation_id)
            .await?;
        let result = operation().await;
        self.audit_operation_result(correlation_id, result).await
    }
}

impl<UG, TG> PostgresAuditLog<UG, TG>
where
    TG: Generator<DateTime<Utc>>,
    UG: Generator<Uuid>,
{
    async fn audit_operation_start<E>(
        &self,
        operation_name: String,
        parameters: JsonValue,
        correlation_id: Uuid,
    ) -> Result<(), E>
    where
        E: From<PostgresAuditLogError>,
    {
        let timestamp: DateTimeWithTimeZone = self.time_generator.generate().into();

        let audit_start = audit_log::ActiveModel {
            correlation_id: Set(correlation_id),
            timestamp: Set(timestamp),
            operation: Set(Some(operation_name)),
            params: Set(Some(parameters.clone())),
            ..Default::default()
        };

        audit_start.insert(&self.db_connection).await.map_err(|error| {
            // Log the audit error
            tracing::debug!(
                audit_timestamp = %timestamp,
                audit_parameters = %parameters,
                "error while auditing start of operation: {error}",
            );
            PostgresAuditLogError(error)
        })?;

        Ok(())
    }

    async fn audit_operation_result<T, E>(&self, correlation_id: Uuid, result: Result<T, E>) -> Result<T, E>
    where
        E: From<PostgresAuditLogError> + Display,
    {
        let timestamp: DateTimeWithTimeZone = self.time_generator.generate().into();
        let is_success = result.is_ok();

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
            PostgresAuditLogError(error)
        })?;

        result
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use std::marker::PhantomData;

    use sea_orm::JsonValue;

    use crate::model::AuditLog;

    pub struct MockAuditLog<E>(PhantomData<E>);

    impl<E> Default for MockAuditLog<E> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }

    impl<EE> AuditLog for MockAuditLog<EE> {
        type Error = EE;

        async fn audit<F, T, E>(
            &self,
            _operation_name: impl Into<String>,
            _parameters: JsonValue,
            operation: F,
        ) -> Result<T, E>
        where
            F: AsyncFnOnce() -> Result<T, E>,
        {
            operation().await
        }
    }
}
