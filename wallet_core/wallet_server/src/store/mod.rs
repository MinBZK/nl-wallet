#[cfg(feature = "postgres")]
pub mod postgres;

use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use nl_wallet_mdoc::server_state::{
    Expirable, HasProgress, MemorySessionStore, SessionState, SessionStore, SessionStoreError, SessionStoreTimeouts,
    SessionToken,
};

#[cfg(feature = "disclosure")]
use nl_wallet_mdoc::verifier::DisclosureData;

#[cfg(feature = "issuance")]
use openid4vc::issuer::IssuanceData;

#[cfg(feature = "postgres")]
use crate::store::postgres::PostgresSessionStore;

pub trait SessionDataType {
    const TYPE: &'static str;
}

#[cfg(feature = "disclosure")]
impl SessionDataType for DisclosureData {
    const TYPE: &'static str = "mdoc_disclosure";
}

#[cfg(feature = "issuance")]
impl SessionDataType for openid4vc::issuer::IssuanceData {
    const TYPE: &'static str = "openid4vci_issuance";
}

pub struct SessionStores {
    #[cfg(feature = "disclosure")]
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
                    #[cfg(all(feature = "issuance", feature = "disclosure"))]
                    issuance: SessionStoreVariant::Postgres(store.clone()),
                    #[cfg(all(feature = "issuance", not(feature = "disclosure")))]
                    issuance: SessionStoreVariant::Postgres(store),
                    #[cfg(feature = "disclosure")]
                    disclosure: SessionStoreVariant::Postgres(store),
                })
            }
            "memory" => Ok(SessionStores {
                #[cfg(feature = "issuance")]
                issuance: SessionStoreVariant::Memory(MemorySessionStore::new(timeouts)),
                #[cfg(feature = "disclosure")]
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
