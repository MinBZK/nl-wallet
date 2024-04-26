use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use nl_wallet_mdoc::{
    server_state::{
        HasProgress, MemorySessionStore, SessionState, SessionStore, SessionStoreError, SessionStoreTimeouts,
        SessionToken,
    },
    verifier::DisclosureData,
};

#[cfg(feature = "issuance")]
use openid4vc::issuer::IssuanceData;

#[cfg(feature = "postgres")]
use crate::store::postgres::PostgresSessionStore;

pub trait SessionDataType {
    const TYPE: &'static str;
}

impl SessionDataType for DisclosureData {
    const TYPE: &'static str = "mdoc_disclosure";
}

#[cfg(feature = "issuance")]
impl SessionDataType for openid4vc::issuer::IssuanceData {
    const TYPE: &'static str = "openid4vci_issuance";
}

pub struct SessionStores {
    pub disclosure: SessionStoreVariant<DisclosureData>,

    #[cfg(feature = "issuance")]
    pub issuance: SessionStoreVariant<IssuanceData>,
}

impl SessionStores {
    pub async fn init(url: Url, timeouts: SessionStoreTimeouts) -> Result<SessionStores, anyhow::Error> {
        match url.scheme() {
            #[cfg(feature = "postgres")]
            "postgres" => {
                let store = PostgresSessionStore::try_new(url, timeouts).await?;
                Ok(SessionStores {
                    #[cfg(feature = "issuance")]
                    issuance: SessionStoreVariant::Postgres(store.clone()),
                    disclosure: SessionStoreVariant::Postgres(store),
                })
            }
            "memory" => Ok(SessionStores {
                #[cfg(feature = "issuance")]
                issuance: SessionStoreVariant::Memory(MemorySessionStore::new(timeouts)),
                disclosure: SessionStoreVariant::Memory(MemorySessionStore::new(timeouts)),
            }),
            e => unimplemented!("{}", e),
        }
    }
}

/// This enum effectively switches between the different types that implement `DisclosureSessionStore`,
/// by implementing this trait itself and forwarding the calls to the type contained in the invariant.
pub enum SessionStoreVariant<T> {
    #[cfg(feature = "postgres")]
    Postgres(PostgresSessionStore),
    Memory(MemorySessionStore<T>),
}

impl<T> SessionStore<T> for SessionStoreVariant<T>
where
    T: HasProgress + SessionDataType + Clone + Serialize + DeserializeOwned + Send + Sync,
{
    async fn get(&self, token: &SessionToken) -> Result<SessionState<T>, SessionStoreError> {
        match self {
            #[cfg(feature = "postgres")]
            SessionStoreVariant::Postgres(postgres) => postgres.get(token).await,
            SessionStoreVariant::Memory(memory) => memory.get(token).await,
        }
    }

    async fn write(&self, session: SessionState<T>, is_new: bool) -> Result<(), SessionStoreError> {
        match self {
            #[cfg(feature = "postgres")]
            SessionStoreVariant::Postgres(postgres) => postgres.write(session, is_new).await,
            SessionStoreVariant::Memory(memory) => memory.write(session, is_new).await,
        }
    }

    async fn cleanup(&self) -> Result<(), SessionStoreError> {
        match self {
            #[cfg(feature = "postgres")]
            SessionStoreVariant::Postgres(postgres) => {
                <PostgresSessionStore as SessionStore<T>>::cleanup(postgres).await
            }
            SessionStoreVariant::Memory(memory) => memory.cleanup().await,
        }
    }
}

#[cfg(feature = "postgres")]
pub mod postgres {
    use std::time::Duration;

    use chrono::{DateTime, Utc};
    use sea_orm::{
        sea_query::{Expr, OnConflict},
        ActiveValue, ColumnTrait, ConnectOptions, Database, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
        SqlErr, TransactionTrait,
    };
    use serde::{de::DeserializeOwned, Serialize};
    use strum::{Display, EnumString};
    use tracing::log::LevelFilter;
    use url::Url;

    use crate::entity::session_state;
    use nl_wallet_mdoc::server_state::{
        HasProgress, Progress, SessionState, SessionStore, SessionStoreError, SessionStoreTimeouts, SessionToken,
    };

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
    pub struct PostgresSessionStore {
        pub timeouts: SessionStoreTimeouts,
        connection: DatabaseConnection,
    }

    #[cfg(any(test, feature = "mock_time"))]
    pub static POSTGRES_SESSION_STORE_NOW: once_cell::sync::Lazy<parking_lot::RwLock<Option<DateTime<Utc>>>> =
        once_cell::sync::Lazy::new(|| None.into());

    impl PostgresSessionStore {
        pub async fn try_new(url: Url, timeouts: SessionStoreTimeouts) -> Result<Self, DbErr> {
            let mut connection_options = ConnectOptions::new(url);
            connection_options
                .connect_timeout(DB_CONNECT_TIMEOUT)
                .sqlx_logging(true)
                .sqlx_logging_level(LevelFilter::Trace);

            let connection = Database::connect(connection_options).await?;

            let session_store = Self { timeouts, connection };

            Ok(session_store)
        }

        fn now() -> DateTime<Utc> {
            #[cfg(not(any(test, feature = "mock_time")))]
            return Utc::now();

            #[cfg(any(test, feature = "mock_time"))]
            POSTGRES_SESSION_STORE_NOW.read().unwrap_or_else(Utc::now)
        }
    }

    impl<T> SessionStore<T> for PostgresSessionStore
    where
        T: HasProgress + SessionDataType + Serialize + DeserializeOwned + Send,
    {
        async fn get(&self, token: &SessionToken) -> Result<SessionState<T>, SessionStoreError> {
            // find value by token, deserialize from JSON if it exists
            let state = session_state::Entity::find_by_id((T::TYPE.to_string(), token.to_string()))
                .one(&self.connection)
                .await
                .map_err(|e| SessionStoreError::Other(e.into()))?;

            state
                .ok_or_else(|| SessionStoreError::NotFound(token.clone()))
                .and_then(|state| {
                    // Decode both the status and data columns.
                    let status = state
                        .status
                        .parse::<SessionStatus>()
                        .map_err(|e| SessionStoreError::Deserialize(e.into()))?;
                    let data =
                        serde_json::from_value(state.data).map_err(|e| SessionStoreError::Deserialize(e.into()))?;

                    // If the status is expired, return a error.
                    if matches!(status, SessionStatus::Expired) {
                        return Err(SessionStoreError::Expired(token.clone()));
                    }

                    // Otherwise, convert the remaining columns and return the session state.
                    let state = SessionState {
                        data,
                        token: state.token.into(),
                        last_active: state.last_active_date_time.into(),
                    };

                    Ok(state)
                })
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
            let now = Self::now();
            let succeeded_cutoff = now - self.timeouts.successful_deletion;
            let failed_cutoff = now - self.timeouts.failed_deletion;
            let expiry_cutoff = now - self.timeouts.expiration;

            self.connection
                .transaction::<_, (), DbErr>(|transaction| {
                    Box::pin(async move {
                        // Remove all succeeded sessions that are older than SUCCESSFUL_SESSION_DELETION_MINUTES.
                        session_state::Entity::delete_many()
                            .filter(session_state::Column::Type.eq(T::TYPE.to_string()))
                            .filter(session_state::Column::Status.eq(SessionStatus::Succeeded.to_string()))
                            .filter(session_state::Column::LastActiveDateTime.lt(succeeded_cutoff))
                            .exec(transaction)
                            .await?;

                        // Remove all failed and expired sessions that are older than FAILED_SESSION_DELETION_MINUTES.
                        session_state::Entity::delete_many()
                            .filter(session_state::Column::Type.eq(T::TYPE.to_string()))
                            .filter(
                                session_state::Column::Status
                                    .is_in([SessionStatus::Failed.to_string(), SessionStatus::Expired.to_string()]),
                            )
                            .filter(session_state::Column::LastActiveDateTime.lt(failed_cutoff))
                            .exec(transaction)
                            .await?;

                        // For all active sessions that are older than SESSION_EXPIRY_MINUTES,
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
}
