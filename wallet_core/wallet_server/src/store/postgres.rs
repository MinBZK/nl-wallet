use std::time::Duration;

use chrono::{DateTime, Utc};
use sea_orm::{
    sea_query::{Expr, OnConflict},
    ActiveValue, ColumnTrait, ConnectOptions, Database, DatabaseConnection, DbErr, EntityTrait, QueryFilter, SqlErr,
    TransactionTrait,
};
use serde::{de::DeserializeOwned, Serialize};
use strum::{Display, EnumString};
use tracing::log::LevelFilter;
use url::Url;

use openid4vc::server_state::{
    Expirable, HasProgress, Progress, SessionState, SessionStore, SessionStoreError, SessionStoreTimeouts, SessionToken,
};
use wallet_common::generator::{Generator, TimeGenerator};

use crate::entity::session_state;

use super::SessionDataType;

const DB_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
enum SessionStatus {
    Active,
    Succeeded,
    Failed,
    Expired,
}

impl From<Progress> for SessionStatus {
    fn from(value: Progress) -> Self {
        match value {
            Progress::Active => Self::Active,
            Progress::Finished { has_succeeded } if has_succeeded => Self::Succeeded,
            Progress::Finished { .. } => Self::Failed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresSessionStore<G = TimeGenerator> {
    pub timeouts: SessionStoreTimeouts,
    time: G,
    connection: DatabaseConnection,
}

impl<G> PostgresSessionStore<G> {
    pub async fn try_new_with_time(url: Url, timeouts: SessionStoreTimeouts, time: G) -> Result<Self, DbErr> {
        let mut connection_options = ConnectOptions::new(url);
        connection_options
            .connect_timeout(DB_CONNECT_TIMEOUT)
            .sqlx_logging(true)
            .sqlx_logging_level(LevelFilter::Trace);

        let connection = Database::connect(connection_options).await?;

        let session_store = Self {
            timeouts,
            time,
            connection,
        };

        Ok(session_store)
    }
}

impl PostgresSessionStore {
    pub async fn try_new(url: Url, timeouts: SessionStoreTimeouts) -> Result<Self, DbErr> {
        Self::try_new_with_time(url, timeouts, TimeGenerator).await
    }
}

impl<T, G> SessionStore<T> for PostgresSessionStore<G>
where
    T: HasProgress + Expirable + SessionDataType + Serialize + DeserializeOwned + Send,
    G: Generator<DateTime<Utc>> + Send + Sync,
{
    async fn get(&self, token: &SessionToken) -> Result<Option<SessionState<T>>, SessionStoreError> {
        // find value by token, deserialize from JSON if it exists
        let state = session_state::Entity::find_by_id((T::TYPE.to_string(), token.to_string()))
            .one(&self.connection)
            .await
            .map_err(|e| SessionStoreError::Other(e.into()))?;

        state
            .map(|state| {
                // Decode both the status and data columns.
                let status = state
                    .status
                    .parse::<SessionStatus>()
                    .map_err(|e| SessionStoreError::Deserialize(e.into()))?;
                let mut data =
                    serde_json::from_value::<T>(state.data).map_err(|e| SessionStoreError::Deserialize(e.into()))?;

                // If the status is expired, expire the data.
                if matches!(status, SessionStatus::Expired) {
                    data.expire();
                }

                // Otherwise, convert the remaining columns and return the session state.
                let state = SessionState {
                    data,
                    token: state.token.into(),
                    last_active: state.last_active_date_time.into(),
                };

                Ok(state)
            })
            .transpose()
    }

    async fn write(&self, session: SessionState<T>, is_new: bool) -> Result<(), SessionStoreError> {
        // Needed for potential SessionStoreError::DuplicateToken.
        let session_token = session.token.clone();

        // Insert new value, with data serialized to JSON.
        let status = SessionStatus::from(session.data.progress()); // This cannot be `Expired`.
        let query = session_state::Entity::insert(session_state::ActiveModel {
            r#type: ActiveValue::set(T::TYPE.to_string()),
            token: ActiveValue::set(session.token.into()),
            data: ActiveValue::set(
                serde_json::to_value(session.data).map_err(|e| SessionStoreError::Serialize(Box::new(e)))?,
            ),
            status: ActiveValue::set(status.to_string()),
            last_active_date_time: ActiveValue::set(session.last_active.into()),
        });

        // If this is an existing session, an update is allowed.
        let final_query = match is_new {
            true => query,
            false => query.on_conflict(
                OnConflict::columns([session_state::PrimaryKey::Type, session_state::PrimaryKey::Token])
                    .update_columns([session_state::Column::Data, session_state::Column::LastActiveDateTime])
                    .to_owned(),
            ),
        };

        // Execute the query and handle a conflicting primary key when updates are not allowed.
        final_query.exec(&self.connection).await.map_err(|e| {
            if matches!(e.sql_err(), Some(SqlErr::UniqueConstraintViolation(_))) {
                return SessionStoreError::DuplicateToken(session_token);
            }

            SessionStoreError::Other(e.into())
        })?;

        Ok(())
    }

    async fn cleanup(&self) -> Result<(), SessionStoreError> {
        let now = self.time.generate();
        let succeeded_cutoff = now - self.timeouts.successful_deletion;
        let failed_cutoff = now - self.timeouts.failed_deletion;
        let expiry_cutoff = now - self.timeouts.expiration;

        self.connection
            .transaction::<_, (), DbErr>(|transaction| {
                Box::pin(async move {
                    // Remove all succeeded sessions that are older than the "successful_deletion" timeout.
                    session_state::Entity::delete_many()
                        .filter(session_state::Column::Type.eq(T::TYPE.to_string()))
                        .filter(session_state::Column::Status.eq(SessionStatus::Succeeded.to_string()))
                        .filter(session_state::Column::LastActiveDateTime.lt(succeeded_cutoff))
                        .exec(transaction)
                        .await?;

                    // Remove all failed and expired sessions that are older than the "failed_deletion" timeout.
                    session_state::Entity::delete_many()
                        .filter(session_state::Column::Type.eq(T::TYPE.to_string()))
                        .filter(
                            session_state::Column::Status
                                .is_in([SessionStatus::Failed.to_string(), SessionStatus::Expired.to_string()]),
                        )
                        .filter(session_state::Column::LastActiveDateTime.lt(failed_cutoff))
                        .exec(transaction)
                        .await?;

                    // For all active sessions that are older than the "expiration" timeout,
                    // update the last active time and set the status to expired.
                    session_state::Entity::update_many()
                        .col_expr(
                            session_state::Column::Status,
                            Expr::value(SessionStatus::Expired.to_string()),
                        )
                        .col_expr(session_state::Column::LastActiveDateTime, Expr::value(now))
                        .filter(session_state::Column::Type.eq(T::TYPE.to_string()))
                        .filter(session_state::Column::Status.eq(SessionStatus::Active.to_string()))
                        .filter(session_state::Column::LastActiveDateTime.lt(expiry_cutoff))
                        .exec(transaction)
                        .await?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| SessionStoreError::Other(e.into()))?;

        Ok(())
    }
}
