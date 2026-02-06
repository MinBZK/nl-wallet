#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "postgres")]
use postgres::PostgresSessionStore;

use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

use openid4vc::server_state::Expirable;
use openid4vc::server_state::HasProgress;
use openid4vc::server_state::MemorySessionStore;
use openid4vc::server_state::SessionDataType;
use openid4vc::server_state::SessionState;
use openid4vc::server_state::SessionStore;
use openid4vc::server_state::SessionStoreError;
use openid4vc::server_state::SessionStoreTimeouts;
use openid4vc::server_state::SessionToken;

/// This enum effectively switches between the different types that implement `DisclosureSessionStore`,
/// by implementing this trait itself and forwarding the calls to the type contained in the invariant.
pub enum SessionStoreVariant<T> {
    #[cfg(feature = "postgres")]
    Postgres(PostgresSessionStore),
    Memory(MemorySessionStore<T>),
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[cfg(feature = "postgres")]
    #[error("database error: {0}")]
    DbError(#[from] sea_orm::DbErr),
}

#[derive(Debug, Clone)]
pub enum StoreConnection {
    #[cfg(feature = "postgres")]
    Postgres(sea_orm::DatabaseConnection),
    Memory,
}

impl StoreConnection {
    #[cfg_attr(not(feature = "postgres"), expect(clippy::unused_async))]
    pub async fn try_new(url: Url) -> Result<Self, StoreError> {
        match url.scheme() {
            #[cfg(feature = "postgres")]
            "postgres" => Ok(Self::Postgres(postgres::new_connection(url).await?)),
            "memory" => Ok(Self::Memory),
            e => unimplemented!("{}", e),
        }
    }
}

impl<T> SessionStoreVariant<T> {
    #[cfg_attr(not(feature = "postgres"), expect(clippy::needless_pass_by_value))]
    pub fn new(connection: StoreConnection, timeouts: SessionStoreTimeouts) -> SessionStoreVariant<T> {
        match connection {
            #[cfg(feature = "postgres")]
            StoreConnection::Postgres(connection) => {
                SessionStoreVariant::Postgres(PostgresSessionStore::new(connection, timeouts))
            }
            StoreConnection::Memory => SessionStoreVariant::Memory(MemorySessionStore::new(timeouts)),
        }
    }
}

impl<T> SessionStore<T> for SessionStoreVariant<T>
where
    T: HasProgress + Expirable + SessionDataType + Clone + Serialize + DeserializeOwned + Send + Sync,
{
    async fn get(&self, token: &SessionToken) -> Result<Option<SessionState<T>>, SessionStoreError> {
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
