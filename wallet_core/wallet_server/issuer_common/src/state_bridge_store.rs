use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use openid4vc::store::MemoryStore;
use openid4vc::store::Store;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::DatabaseConnection;
use sea_orm::DbBackend;
use sea_orm::DbErr;
use sea_orm::EntityTrait;
use sea_orm::NotSet;
use sea_orm::QueryFilter;
use sea_orm::Set;
use sea_orm::Statement;
use serde::Serialize;
use serde::de::DeserializeOwned;
use server_utils::store::StoreConnection;
use tracing::info;
use utils::generator::Generator;
use utils::generator::TimeGenerator;

use crate::entity::prelude::*;
use crate::entity::state_bridge;

/// TTL for state bridge store entries. Bounds how long an entry may live between being
/// stored and consumed before it is treated as expired.
const STATE_BRIDGE_ENTRY_TTL: Duration = Duration::minutes(30);

/// Maximum rows deleted per statement during cleanup, to bound lock duration and DB load.
const CLEANUP_BATCH_SIZE: u64 = 1_000;

#[derive(Debug, thiserror::Error)]
pub enum IssuerStateBridgeStoreError {
    #[error("could not store state bridge entry in database: {0}")]
    DbStore(#[source] DbErr),

    #[error("could not consume state bridge entry from database: {0}")]
    DbConsume(#[source] DbErr),

    #[error("could not delete expired state bridge entries from database: {0}")]
    DbCleanup(#[source] DbErr),

    #[error("could not serialize state bridge entry: {0}")]
    Serialize(#[source] serde_json::Error),

    #[error("could not deserialize state bridge entry: {0}")]
    Deserialize(#[source] serde_json::Error),
}

#[derive(Debug)]
enum StateBridgeStoreBackend<E> {
    Postgres(DatabaseConnection),
    Memory(MemoryStore<String, E>),
}

/// Stores the state-bridge entries that link a downstream wallet authorization request
/// (kept in the entry) to a flow-generated `bridge_key` used to correlate the two sides of
/// the authorization flow. The configured
/// [`AuthorizationCodeFlow`](openid4vc::authorization_code_flow::AuthorizationCodeFlow) impl writes one entry at
/// `/authorize` and consumes it from its callback handler. The store is generic over the entry type and serializes it
/// to/from JSON internally; the entry's shape is private to the AF impl.
#[derive(Debug)]
pub struct IssuerStateBridgeStore<E, T = TimeGenerator> {
    backend: StateBridgeStoreBackend<E>,
    time_generator: T,
}

impl<E> IssuerStateBridgeStore<E> {
    pub fn new(store_connection: StoreConnection) -> Self {
        let backend = match store_connection {
            StoreConnection::Postgres(connection) => StateBridgeStoreBackend::Postgres(connection),
            StoreConnection::Memory => StateBridgeStoreBackend::Memory(MemoryStore::new(STATE_BRIDGE_ENTRY_TTL)),
        };

        Self {
            backend,
            time_generator: TimeGenerator,
        }
    }
}

#[cfg(feature = "db_test")]
impl<E, T> IssuerStateBridgeStore<E, T>
where
    T: Clone,
{
    pub fn new_postgres_with_time_generator(database_connection: DatabaseConnection, time_generator: T) -> Self {
        Self {
            backend: StateBridgeStoreBackend::Postgres(database_connection),
            time_generator,
        }
    }
}

impl<E, T> IssuerStateBridgeStore<E, T>
where
    T: Generator<DateTime<Utc>>,
{
    fn now(&self) -> DateTime<Utc> {
        self.time_generator.generate()
    }
}

impl<E, T> Store<String, E> for IssuerStateBridgeStore<E, T>
where
    E: Serialize + DeserializeOwned + Send,
    T: Generator<DateTime<Utc>> + Send + Sync,
{
    type Error = IssuerStateBridgeStoreError;

    async fn store(&self, bridge_key: String, entry: E) -> Result<(), Self::Error> {
        match &self.backend {
            StateBridgeStoreBackend::Postgres(connection) => {
                let expires_at = self.now() + STATE_BRIDGE_ENTRY_TTL;

                state_bridge::ActiveModel {
                    id: NotSet,
                    bridge_key: Set(bridge_key),
                    entry: Set(serde_json::to_value(entry).map_err(IssuerStateBridgeStoreError::Serialize)?),
                    expires_at: Set(expires_at.into()),
                }
                .insert(connection)
                .await
                .map_err(IssuerStateBridgeStoreError::DbStore)?;

                Ok(())
            }
            StateBridgeStoreBackend::Memory(memory_store) => {
                memory_store.store_inner(bridge_key, entry);
                Ok(())
            }
        }
    }

    async fn consume(&self, bridge_key: impl Into<String> + Send) -> Result<Option<E>, Self::Error> {
        let bridge_key = bridge_key.into();
        match &self.backend {
            StateBridgeStoreBackend::Postgres(connection) => {
                let deleted = StateBridge::delete_many()
                    .filter(state_bridge::Column::BridgeKey.eq(bridge_key.as_str()))
                    .exec_with_returning(connection)
                    .await
                    .map_err(IssuerStateBridgeStoreError::DbConsume)?;

                match deleted.into_iter().next() {
                    Some(model) if self.now() < model.expires_at.to_utc() => {
                        Some(serde_json::from_value(model.entry).map_err(IssuerStateBridgeStoreError::Deserialize))
                            .transpose()
                    }
                    _ => Ok(None),
                }
            }
            StateBridgeStoreBackend::Memory(memory_store) => Ok(memory_store.consume_inner(&bridge_key)),
        }
    }

    async fn cleanup(&self) -> Result<(), Self::Error> {
        match &self.backend {
            StateBridgeStoreBackend::Postgres(connection) => {
                let now = self.now();
                let mut total_deleted: u64 = 0;

                // Delete in bounded batches so a single statement never holds a large lock or scans
                // the whole backlog. `FOR UPDATE SKIP LOCKED` skips rows currently locked by a
                // concurrent `consume`; the loop drains the rest, stopping once a batch removes
                // fewer rows than the limit (nothing left to delete).
                //
                // We currently do not sleep between batches: with these short TTLs and the
                // periodic cleanup cadence, each run only clears a small backlog, so throttling WAL
                // generation to bound replication lag is not needed. If a deployment ever observes
                // replication lag during cleanup, add a small delay between batches here.
                loop {
                    let result = connection
                        .execute(Statement::from_sql_and_values(
                            DbBackend::Postgres,
                            r#"
                            WITH rows_to_delete AS (
                                SELECT id
                                FROM state_bridge
                                WHERE expires_at <= $1
                                ORDER BY expires_at
                                LIMIT $2
                                FOR UPDATE SKIP LOCKED
                            )
                            DELETE FROM state_bridge sb
                            USING rows_to_delete
                            WHERE sb.id = rows_to_delete.id
                            "#,
                            [now.into(), (CLEANUP_BATCH_SIZE as i64).into()],
                        ))
                        .await
                        .map_err(IssuerStateBridgeStoreError::DbCleanup)?;

                    total_deleted += result.rows_affected();
                    if result.rows_affected() < CLEANUP_BATCH_SIZE {
                        break;
                    }
                }

                if total_deleted > 0 {
                    info!("Deleted {total_deleted} expired state bridge entry(s) from storage");
                }

                Ok(())
            }
            StateBridgeStoreBackend::Memory(memory_store) => {
                memory_store.cleanup_inner();
                Ok(())
            }
        }
    }
}
