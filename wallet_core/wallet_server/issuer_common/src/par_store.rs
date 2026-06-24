use chrono::DateTime;
use chrono::Utc;
use openid4vc::authorization::VciAuthorizationRequest;
use openid4vc::par::PAR_TTL;
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
use server_utils::store::StoreConnection;
use tracing::info;
use utils::generator::Generator;
use utils::generator::TimeGenerator;

use crate::entity::prelude::*;
use crate::entity::pushed_authorization_request;

/// Maximum rows deleted per statement during cleanup, to bound lock duration and DB load.
const CLEANUP_BATCH_SIZE: u64 = 1_000;

#[derive(Debug, thiserror::Error)]
pub enum IssuerParStoreError {
    #[error("could not store PAR request in database: {0}")]
    DbStore(#[source] DbErr),

    #[error("could not consume PAR request from database: {0}")]
    DbConsume(#[source] DbErr),

    #[error("could not delete expired PAR requests from database: {0}")]
    DbCleanup(#[source] DbErr),

    #[error("could not serialize PAR request data: {0}")]
    Serialize(#[source] serde_json::Error),

    #[error("could not deserialize PAR request data: {0}")]
    Deserialize(#[source] serde_json::Error),
}

#[derive(Debug)]
enum ParStoreBackend {
    Postgres(DatabaseConnection),
    Memory(MemoryStore<String, VciAuthorizationRequest>),
}

/// Stores pushed authorization requests either in PostgreSQL or in memory.
#[derive(Debug)]
pub struct IssuerParStore<T = TimeGenerator> {
    backend: ParStoreBackend,
    time_generator: T,
}

impl IssuerParStore {
    pub fn new(store_connection: StoreConnection) -> Self {
        let backend = match store_connection {
            StoreConnection::Postgres(connection) => ParStoreBackend::Postgres(connection),
            StoreConnection::Memory => ParStoreBackend::Memory(MemoryStore::new(PAR_TTL)),
        };

        Self {
            backend,
            time_generator: TimeGenerator,
        }
    }
}

#[cfg(feature = "db_test")]
impl<T> IssuerParStore<T>
where
    T: Clone,
{
    pub fn new_postgres_with_time_generator(database_connection: DatabaseConnection, time_generator: T) -> Self {
        Self {
            backend: ParStoreBackend::Postgres(database_connection),
            time_generator,
        }
    }
}

impl<T> IssuerParStore<T>
where
    T: Generator<DateTime<Utc>>,
{
    fn now(&self) -> DateTime<Utc> {
        self.time_generator.generate()
    }
}

impl<T> Store<String, VciAuthorizationRequest> for IssuerParStore<T>
where
    T: Generator<DateTime<Utc>> + Send + Sync,
{
    type Error = IssuerParStoreError;

    async fn store(&self, request_uri: String, data: VciAuthorizationRequest) -> Result<(), Self::Error> {
        match &self.backend {
            ParStoreBackend::Postgres(connection) => {
                let data = serde_json::to_value(&data).map_err(IssuerParStoreError::Serialize)?;
                let expires_at = self.now() + PAR_TTL;

                pushed_authorization_request::ActiveModel {
                    id: NotSet,
                    request_uri: Set(request_uri),
                    data: Set(data),
                    expires_at: Set(expires_at.into()),
                }
                .insert(connection)
                .await
                .map_err(IssuerParStoreError::DbStore)?;

                Ok(())
            }
            ParStoreBackend::Memory(memory_store) => {
                memory_store.store(request_uri, data).await.unwrap();
                Ok(())
            }
        }
    }

    async fn consume(
        &self,
        request_uri: impl Into<String> + Send,
    ) -> Result<Option<VciAuthorizationRequest>, Self::Error> {
        let request_uri = request_uri.into();
        match &self.backend {
            ParStoreBackend::Postgres(connection) => {
                let deleted = PushedAuthorizationRequest::delete_many()
                    .filter(pushed_authorization_request::Column::RequestUri.eq(request_uri.as_str()))
                    .exec_with_returning(connection)
                    .await
                    .map_err(IssuerParStoreError::DbConsume)?;

                let model = match deleted.into_iter().next() {
                    Some(model) => model,
                    None => return Ok(None),
                };

                if self.now() >= model.expires_at.to_utc() {
                    return Ok(None);
                }

                let data = serde_json::from_value(model.data).map_err(IssuerParStoreError::Deserialize)?;
                Ok(Some(data))
            }
            ParStoreBackend::Memory(memory_store) => Ok(memory_store.consume(request_uri.as_str()).await.unwrap()),
        }
    }

    async fn cleanup(&self) -> Result<(), Self::Error> {
        match &self.backend {
            ParStoreBackend::Postgres(connection) => {
                let now = self.now();
                let mut total_deleted: u64 = 0;

                // Delete in bounded batches so a single statement never holds a large lock or scans
                // the whole backlog. `FOR UPDATE SKIP LOCKED` skips rows currently locked by a
                // concurrent `consume`; the loop drains the rest, stopping once a batch removes
                // fewer rows than the limit (nothing left to delete).
                loop {
                    let result = connection
                        .execute(Statement::from_sql_and_values(
                            DbBackend::Postgres,
                            r#"
                            WITH rows_to_delete AS (
                                SELECT id
                                FROM pushed_authorization_request
                                WHERE expires_at <= $1
                                ORDER BY expires_at
                                LIMIT $2
                                FOR UPDATE SKIP LOCKED
                            )
                            DELETE FROM pushed_authorization_request par
                            USING rows_to_delete
                            WHERE par.id = rows_to_delete.id
                            "#,
                            [now.into(), (CLEANUP_BATCH_SIZE as i64).into()],
                        ))
                        .await
                        .map_err(IssuerParStoreError::DbCleanup)?;

                    total_deleted += result.rows_affected();
                    if result.rows_affected() < CLEANUP_BATCH_SIZE {
                        break;
                    }
                }

                if total_deleted > 0 {
                    info!("Deleted {total_deleted} expired PAR request(s) from storage");
                }

                Ok(())
            }
            ParStoreBackend::Memory(memory_store) => {
                memory_store.cleanup().await.unwrap();
                Ok(())
            }
        }
    }
}
