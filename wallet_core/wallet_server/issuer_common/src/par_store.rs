use chrono::DateTime;
use chrono::Utc;
use openid4vc::authorization::VciAuthorizationRequest;
use openid4vc::par::PAR_TTL;
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

use crate::entity::prelude::*;
use crate::entity::pushed_authorization_request;

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

                if self.now() > model.expires_at.to_utc() {
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
                let result = PushedAuthorizationRequest::delete_many()
                    .filter(pushed_authorization_request::Column::ExpiresAt.lt(self.now()))
                    .exec(connection)
                    .await
                    .map_err(IssuerParStoreError::DbCleanup)?;

                if result.rows_affected > 0 {
                    info!("Deleted {} expired PAR request(s) from storage", result.rows_affected);
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
