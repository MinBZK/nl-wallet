cfg_if::cfg_if! {
    if #[cfg(feature = "postgres")] {
        pub mod postgres;
        use postgres::PostgresSessionStore;
    }
}

use postgres::PostgresWteTracker;
use sea_orm::{DatabaseConnection, DbErr};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use openid4vc::server_state::{
    Expirable, HasProgress, MemorySessionStore, MemoryWteTracker, SessionState, SessionStore, SessionStoreError,
    SessionStoreTimeouts, SessionToken, WteTracker,
};
use wallet_common::{
    account::messages::instructions::WteClaims,
    jwt::{Jwt, JwtCredentialClaims},
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

#[derive(Debug, Clone)]
pub enum Database {
    Postgres(DatabaseConnection),
    Memory,
}

impl Database {
    pub async fn try_new(url: Url) -> Result<Self, sea_orm::DbErr> {
        match url.scheme() {
            #[cfg(feature = "postgres")]
            "postgres" => Ok(Self::Postgres(postgres::new_connection(url).await?)),
            "memory" => Ok(Self::Memory),
            e => unimplemented!("{}", e),
        }
    }
}

impl<T> SessionStoreVariant<T> {
    pub fn new(database: Database, timeouts: SessionStoreTimeouts) -> SessionStoreVariant<T> {
        match database {
            #[cfg(feature = "postgres")]
            Database::Postgres(connection) => {
                SessionStoreVariant::Postgres(PostgresSessionStore::new(connection, timeouts))
            }
            Database::Memory => SessionStoreVariant::Memory(MemorySessionStore::new(timeouts)),
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

pub enum WteTrackerVariant {
    #[cfg(feature = "postgres")]
    Postgres(PostgresWteTracker),
    Memory(MemoryWteTracker),
}

impl WteTrackerVariant {
    pub fn new(connection: Database) -> Self {
        match connection {
            Database::Postgres(connection) => Self::Postgres(PostgresWteTracker::new(connection)),
            Database::Memory => Self::Memory(MemoryWteTracker::new()),
        }
    }
}

impl WteTracker for WteTrackerVariant {
    type Error = DbErr;

    async fn previously_seen_wte(&self, wte: &Jwt<JwtCredentialClaims<WteClaims>>) -> Result<bool, Self::Error> {
        match self {
            WteTrackerVariant::Postgres(postgres_wte_tracker) => postgres_wte_tracker.previously_seen_wte(wte).await,
            WteTrackerVariant::Memory(memory_wte_tracker) => {
                Ok(memory_wte_tracker.previously_seen_wte(wte).await.unwrap()) // this implementation is infallible
            }
        }
    }

    async fn cleanup(&self) -> Result<(), Self::Error> {
        match self {
            WteTrackerVariant::Postgres(postgres_wte_tracker) => postgres_wte_tracker.cleanup().await,
            WteTrackerVariant::Memory(memory_wte_tracker) => {
                Ok(memory_wte_tracker.cleanup().await.unwrap()) // this implementation is infallible
            }
        }
    }
}
