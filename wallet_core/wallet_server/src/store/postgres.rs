use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use sea_orm::sea_query::Expr;
use sea_orm::sea_query::OnConflict;
use sea_orm::ActiveValue;
use sea_orm::ColumnTrait;
use sea_orm::ConnectOptions;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::SqlErr;
use sea_orm::TransactionTrait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use strum::Display;
use strum::EnumString;
use tracing::log::LevelFilter;
use url::Url;

use openid4vc::server_state::Expirable;
use openid4vc::server_state::HasProgress;
use openid4vc::server_state::Progress;
use openid4vc::server_state::SessionState;
use openid4vc::server_state::SessionStore;
use openid4vc::server_state::SessionStoreError;
use openid4vc::server_state::SessionStoreTimeouts;
use openid4vc::server_state::SessionToken;
use wallet_common::generator::Generator;
use wallet_common::generator::TimeGenerator;

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

pub async fn new_connection(url: Url) -> Result<DatabaseConnection, DbErr> {
    let mut connection_options = ConnectOptions::new(url);
    connection_options
        .connect_timeout(DB_CONNECT_TIMEOUT)
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Trace);

    Database::connect(connection_options).await
}

#[derive(Debug, Clone)]
pub struct PostgresSessionStore<G = TimeGenerator> {
    pub timeouts: SessionStoreTimeouts,
    time: G,
    connection: DatabaseConnection,
}

impl<G> PostgresSessionStore<G> {
    pub fn new_with_time(connection: DatabaseConnection, timeouts: SessionStoreTimeouts, time: G) -> Self {
        Self {
            timeouts,
            time,
            connection,
        }
    }
}

impl PostgresSessionStore {
    pub fn new(connection: DatabaseConnection, timeouts: SessionStoreTimeouts) -> Self {
        Self::new_with_time(connection, timeouts, TimeGenerator)
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

#[cfg(feature = "issuance")]
pub use wte_tracker::PostgresWteTracker;

#[cfg(feature = "issuance")]
mod wte_tracker {
    use chrono::DateTime;
    use chrono::Utc;
    use sea_orm::ActiveValue;
    use sea_orm::ColumnTrait;
    use sea_orm::DatabaseConnection;
    use sea_orm::DbErr;
    use sea_orm::EntityTrait;
    use sea_orm::QueryFilter;
    use sea_orm::SqlErr;

    use wallet_common::generator::Generator;
    use wallet_common::generator::TimeGenerator;
    use wallet_common::jwt::JwtCredentialClaims;
    use wallet_common::jwt::VerifiedJwt;
    use wallet_common::utils::sha256;
    use wallet_common::wte::WteClaims;

    use crate::entity::used_wtes;
    use openid4vc::server_state::WteTracker;

    pub struct PostgresWteTracker<G = TimeGenerator> {
        time: G,
        connection: DatabaseConnection,
    }

    impl<G> PostgresWteTracker<G> {
        pub fn new_with_time(connection: DatabaseConnection, time: G) -> Self {
            Self { time, connection }
        }
    }

    impl PostgresWteTracker {
        pub fn new(connection: DatabaseConnection) -> Self {
            Self::new_with_time(connection, TimeGenerator)
        }
    }

    impl<G> WteTracker for PostgresWteTracker<G>
    where
        G: Generator<DateTime<Utc>> + Send + Sync,
    {
        type Error = DbErr;

        async fn track_wte(&self, wte: &VerifiedJwt<JwtCredentialClaims<WteClaims>>) -> Result<bool, Self::Error> {
            let shasum = sha256(wte.jwt().0.as_bytes());
            let expires = wte.payload().contents.attributes.exp;

            let query_result = used_wtes::Entity::insert(used_wtes::ActiveModel {
                used_wte_hash: ActiveValue::set(shasum),
                expires: ActiveValue::set(expires.into()),
            })
            .exec(&self.connection)
            .await;

            match query_result {
                Ok(_) => Ok(false),
                Err(err) if matches!(err.sql_err(), Some(SqlErr::UniqueConstraintViolation(_))) => Ok(true),
                Err(err) => Err(err),
            }
        }

        async fn cleanup(&self) -> Result<(), Self::Error> {
            let now = self.time.generate();

            match used_wtes::Entity::delete_many()
                .filter(used_wtes::Column::Expires.lte(now))
                .exec(&self.connection)
                .await
            {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        }
    }
}
