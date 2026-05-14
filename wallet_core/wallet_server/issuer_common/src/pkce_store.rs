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
use server_utils::store::StoreConnection;
use tracing::info;
use utils::generator::Generator;
use utils::generator::TimeGenerator;

use crate::entity::pkce_flow;
use crate::entity::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum IssuerPkceStoreError {
    #[error("could not store PKCE flow entry in database: {0}")]
    DbStore(#[source] DbErr),

    #[error("could not consume PKCE flow entry from database: {0}")]
    DbConsume(#[source] DbErr),

    #[error("could not delete expired PKCE flow entries from database: {0}")]
    DbCleanup(#[source] DbErr),
}

#[derive(Debug)]
enum PkceStoreBackend {
    Postgres(DatabaseConnection),
    Memory(MemoryStore<String, String>),
}

/// Stores PKCE flow entries either in PostgreSQL or in memory.
#[derive(Debug)]
pub struct IssuerPkceStore<T = TimeGenerator> {
    backend: PkceStoreBackend,
    time_generator: T,
}

impl IssuerPkceStore {
    pub fn new(store_connection: StoreConnection) -> Self {
        let backend = match store_connection {
            StoreConnection::Postgres(connection) => PkceStoreBackend::Postgres(connection),
            StoreConnection::Memory => PkceStoreBackend::Memory(MemoryStore::new(PKCE_FLOW_TTL)),
        };

        Self {
            backend,
            time_generator: TimeGenerator,
        }
    }
}

#[cfg(feature = "db_test")]
impl<T> IssuerPkceStore<T>
where
    T: Clone,
{
    pub fn new_postgres_with_time_generator(database_connection: DatabaseConnection, time_generator: T) -> Self {
        Self {
            backend: PkceStoreBackend::Postgres(database_connection),
            time_generator,
        }
    }
}

impl<T> IssuerPkceStore<T>
where
    T: Generator<DateTime<Utc>>,
{
    fn now(&self) -> DateTime<Utc> {
        self.time_generator.generate()
    }
}

impl<T> Store<String, String> for IssuerPkceStore<T>
where
    T: Generator<DateTime<Utc>> + Send + Sync,
{
    type Error = IssuerPkceStoreError;

    async fn store(&self, wallet_code_challenge: String, upstream_code_verifier: String) -> Result<(), Self::Error> {
        match &self.backend {
            PkceStoreBackend::Postgres(connection) => {
                let expires_at = self.now() + PKCE_FLOW_TTL;

                pkce_flow::ActiveModel {
                    id: NotSet,
                    wallet_code_challenge: Set(wallet_code_challenge),
                    upstream_code_verifier: Set(upstream_code_verifier),
                    expires_at: Set(expires_at.into()),
                }
                .insert(connection)
                .await
                .map_err(IssuerPkceStoreError::DbStore)?;

                Ok(())
            }
            PkceStoreBackend::Memory(memory_store) => {
                memory_store
                    .store(wallet_code_challenge, upstream_code_verifier)
                    .await
                    .unwrap();
                Ok(())
            }
        }
    }

    async fn consume(&self, wallet_code_challenge: impl Into<String> + Send) -> Result<Option<String>, Self::Error> {
        let wallet_code_challenge = wallet_code_challenge.into();
        match &self.backend {
            PkceStoreBackend::Postgres(connection) => {
                let deleted = PkceFlow::delete_many()
                    .filter(pkce_flow::Column::WalletCodeChallenge.eq(wallet_code_challenge.as_str()))
                    .exec_with_returning(connection)
                    .await
                    .map_err(IssuerPkceStoreError::DbConsume)?;

                let model = match deleted.into_iter().next() {
                    Some(model) => model,
                    None => return Ok(None),
                };

                if self.now() > model.expires_at.to_utc() {
                    return Ok(None);
                }

                Ok(Some(model.upstream_code_verifier))
            }
            PkceStoreBackend::Memory(memory_store) => {
                Ok(memory_store.consume(wallet_code_challenge.as_str()).await.unwrap())
            }
        }
    }

    async fn cleanup(&self) -> Result<(), Self::Error> {
        match &self.backend {
            PkceStoreBackend::Postgres(connection) => {
                let result = PkceFlow::delete_many()
                    .filter(pkce_flow::Column::ExpiresAt.lt(self.now()))
                    .exec(connection)
                    .await
                    .map_err(IssuerPkceStoreError::DbCleanup)?;

                if result.rows_affected > 0 {
                    info!(
                        "Deleted {} expired PKCE flow entry(s) from storage",
                        result.rows_affected
                    );
                }

                Ok(())
            }
            PkceStoreBackend::Memory(memory_store) => {
                memory_store.cleanup().await.unwrap();
                Ok(())
            }
        }
    }
}
