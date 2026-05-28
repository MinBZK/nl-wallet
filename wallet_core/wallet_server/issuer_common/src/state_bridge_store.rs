use chrono::DateTime;
use chrono::Utc;
use openid4vc::pkce::PKCE_FLOW_TTL;
use openid4vc::store::MemoryStore;
use openid4vc::store::Store;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
use sea_orm::EntityTrait;
use sea_orm::NotSet;
use sea_orm::QueryFilter;
use sea_orm::Set;
use serde::Serialize;
use serde::de::DeserializeOwned;
use server_utils::store::StoreConnection;
use tracing::info;
use utils::generator::Generator;
use utils::generator::TimeGenerator;

use crate::entity::prelude::*;
use crate::entity::state_bridge;

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
enum StateBridgeStoreBackend {
    Postgres(DatabaseConnection),
    Memory(MemoryStore<String, String>),
}

/// Stores the state-bridge entries that link a downstream wallet authorization request
/// (kept in the entry) to the issuer-generated `issuer_state` sent to the upstream OIDC provider.
/// The configured [`AuthorizationCodeFlow`](openid4vc::authorization_code_flow::AuthorizationCodeFlow)
/// impl writes one entry at `/authorize` and consumes it from its upstream-callback handler.
/// The store is generic over the entry type and serializes it to/from JSON internally; the entry's
/// shape is private to the AF impl.
#[derive(Debug)]
pub struct IssuerStateBridgeStore<T = TimeGenerator> {
    backend: StateBridgeStoreBackend,
    time_generator: T,
}

impl IssuerStateBridgeStore {
    pub fn new(store_connection: StoreConnection) -> Self {
        let backend = match store_connection {
            StoreConnection::Postgres(connection) => StateBridgeStoreBackend::Postgres(connection),
            StoreConnection::Memory => StateBridgeStoreBackend::Memory(MemoryStore::new(PKCE_FLOW_TTL)),
        };

        Self {
            backend,
            time_generator: TimeGenerator,
        }
    }
}

#[cfg(feature = "db_test")]
impl<T> IssuerStateBridgeStore<T>
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

impl<T> IssuerStateBridgeStore<T>
where
    T: Generator<DateTime<Utc>>,
{
    fn now(&self) -> DateTime<Utc> {
        self.time_generator.generate()
    }
}

impl<T, E> Store<String, E> for IssuerStateBridgeStore<T>
where
    T: Generator<DateTime<Utc>> + Send + Sync,
    E: Serialize + DeserializeOwned + Send,
{
    type Error = IssuerStateBridgeStoreError;

    async fn store(&self, issuer_state: String, entry: E) -> Result<(), Self::Error> {
        let entry = serde_json::to_string(&entry).map_err(IssuerStateBridgeStoreError::Serialize)?;

        match &self.backend {
            StateBridgeStoreBackend::Postgres(connection) => {
                let expires_at = self.now() + PKCE_FLOW_TTL;

                state_bridge::ActiveModel {
                    id: NotSet,
                    issuer_state: Set(issuer_state),
                    entry: Set(entry),
                    expires_at: Set(expires_at.into()),
                }
                .insert(connection)
                .await
                .map_err(IssuerStateBridgeStoreError::DbStore)?;

                Ok(())
            }
            StateBridgeStoreBackend::Memory(memory_store) => {
                memory_store.store(issuer_state, entry).await.unwrap();
                Ok(())
            }
        }
    }

    async fn consume(&self, issuer_state: impl Into<String> + Send) -> Result<Option<E>, Self::Error> {
        let issuer_state = issuer_state.into();
        let entry = match &self.backend {
            StateBridgeStoreBackend::Postgres(connection) => {
                let deleted = StateBridge::delete_many()
                    .filter(state_bridge::Column::IssuerState.eq(issuer_state.as_str()))
                    .exec_with_returning(connection)
                    .await
                    .map_err(IssuerStateBridgeStoreError::DbConsume)?;

                match deleted.into_iter().next() {
                    Some(model) if self.now() <= model.expires_at.to_utc() => Some(model.entry),
                    _ => None,
                }
            }
            StateBridgeStoreBackend::Memory(memory_store) => memory_store.consume(issuer_state.as_str()).await.unwrap(),
        };

        entry
            .map(|entry| serde_json::from_str(&entry).map_err(IssuerStateBridgeStoreError::Deserialize))
            .transpose()
    }

    async fn cleanup(&self) -> Result<(), Self::Error> {
        match &self.backend {
            StateBridgeStoreBackend::Postgres(connection) => {
                // TODO (PVW-5911): limit the number of deleted rows by selecting ids (using FOR UPDATE SKIP LOCKED)
                // and then delete
                let result = StateBridge::delete_many()
                    .filter(state_bridge::Column::ExpiresAt.lt(self.now()))
                    .exec(connection)
                    .await
                    .map_err(IssuerStateBridgeStoreError::DbCleanup)?;

                if result.rows_affected > 0 {
                    info!(
                        "Deleted {} expired state bridge entry(s) from storage",
                        result.rows_affected
                    );
                }

                Ok(())
            }
            StateBridgeStoreBackend::Memory(memory_store) => {
                memory_store.cleanup().await.unwrap();
                Ok(())
            }
        }
    }
}
