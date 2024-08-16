cfg_if::cfg_if! {
    if #[cfg(feature = "postgres")] {
        pub mod postgres;
        use postgres::PostgresSessionStore;
    }
}

use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use openid4vc::server_state::{
    Expirable, HasProgress, MemorySessionStore, SessionState, SessionStore, SessionStoreError, SessionStoreTimeouts,
    SessionToken,
};

pub trait SessionDataType {
    const TYPE: &'static str;
}

cfg_if::cfg_if! {
    if #[cfg(feature = "disclosure")] {
        use openid4vc::verifier::DisclosureData;
        impl SessionDataType for DisclosureData {
            const TYPE: &'static str = "mdoc_disclosure";
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "issuance")] {
        use openid4vc::issuer::IssuanceData;
        impl SessionDataType for IssuanceData {
            const TYPE: &'static str = "openid4vci_issuance";
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

impl<T> SessionStoreVariant<T> {
    pub async fn new(url: Url, timeouts: SessionStoreTimeouts) -> Result<SessionStoreVariant<T>, anyhow::Error> {
        match url.scheme() {
            #[cfg(feature = "postgres")]
            "postgres" => {
                let store = PostgresSessionStore::try_new(url, timeouts).await?;
                Ok(SessionStoreVariant::Postgres(store))
            }
            "memory" => Ok(SessionStoreVariant::Memory(MemorySessionStore::new(timeouts))),
            e => unimplemented!("{}", e),
        }
    }

    /// Clone this [SessionStoreVariant] into a SessionStoreVariant with a different generic type, reusing the
    /// underlying implementation. This function is provided so that the same connection pool is used for PostgreSQL
    /// connections.
    pub fn clone_into<R>(&self) -> SessionStoreVariant<R> {
        match self {
            #[cfg(feature = "postgres")]
            SessionStoreVariant::Postgres(store) => SessionStoreVariant::Postgres(store.clone()),
            SessionStoreVariant::Memory(MemorySessionStore { timeouts, .. }) => {
                SessionStoreVariant::Memory(MemorySessionStore::new(*timeouts))
            }
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
