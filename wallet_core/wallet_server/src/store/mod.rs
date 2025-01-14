cfg_if::cfg_if! {
    if #[cfg(feature = "postgres")] {
        pub mod postgres;
        use postgres::PostgresSessionStore;
    }
}

use serde::de::DeserializeOwned;
use serde::Serialize;
use url::Url;

use openid4vc::server_state::Expirable;
use openid4vc::server_state::HasProgress;
use openid4vc::server_state::MemorySessionStore;
use openid4vc::server_state::SessionState;
use openid4vc::server_state::SessionStore;
use openid4vc::server_state::SessionStoreError;
use openid4vc::server_state::SessionStoreTimeouts;
use openid4vc::server_state::SessionToken;

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

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[cfg(feature = "postgres")]
    #[error("database error: {0}")]
    DbError(#[from] sea_orm::DbErr),
}

#[derive(Debug, Clone)]
pub enum DatabaseConnection {
    #[cfg(feature = "postgres")]
    Postgres(sea_orm::DatabaseConnection),
    Memory,
}

impl DatabaseConnection {
    pub async fn try_new(url: Url) -> Result<Self, DatabaseError> {
        match url.scheme() {
            #[cfg(feature = "postgres")]
            "postgres" => Ok(Self::Postgres(postgres::new_connection(url).await?)),
            "memory" => Ok(Self::Memory),
            e => unimplemented!("{}", e),
        }
    }
}

impl<T> SessionStoreVariant<T> {
    pub fn new(database: DatabaseConnection, timeouts: SessionStoreTimeouts) -> SessionStoreVariant<T> {
        match database {
            #[cfg(feature = "postgres")]
            DatabaseConnection::Postgres(connection) => {
                SessionStoreVariant::Postgres(PostgresSessionStore::new(connection, timeouts))
            }
            DatabaseConnection::Memory => SessionStoreVariant::Memory(MemorySessionStore::new(timeouts)),
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

#[cfg(feature = "issuance")]
pub use wte_tracker::WteTrackerVariant;

#[cfg(feature = "issuance")]
mod wte_tracker {
    use openid4vc::server_state::MemoryWteTracker;
    use openid4vc::server_state::WteTracker;
    use wallet_common::jwt::JwtCredentialClaims;
    use wallet_common::jwt::VerifiedJwt;
    use wallet_common::wte::WteClaims;

    use super::DatabaseConnection;
    use super::DatabaseError;

    #[cfg(feature = "postgres")]
    use super::postgres::PostgresWteTracker;

    pub enum WteTrackerVariant {
        #[cfg(feature = "postgres")]
        Postgres(PostgresWteTracker),
        Memory(MemoryWteTracker),
    }

    impl WteTrackerVariant {
        pub fn new(connection: DatabaseConnection) -> Self {
            match connection {
                #[cfg(feature = "postgres")]
                DatabaseConnection::Postgres(connection) => Self::Postgres(PostgresWteTracker::new(connection)),
                DatabaseConnection::Memory => Self::Memory(MemoryWteTracker::new()),
            }
        }
    }

    impl WteTracker for WteTrackerVariant {
        type Error = DatabaseError;

        async fn track_wte(&self, wte: &VerifiedJwt<JwtCredentialClaims<WteClaims>>) -> Result<bool, Self::Error> {
            match self {
                #[cfg(feature = "postgres")]
                WteTrackerVariant::Postgres(postgres_wte_tracker) => Ok(postgres_wte_tracker.track_wte(wte).await?),
                WteTrackerVariant::Memory(memory_wte_tracker) => {
                    Ok(memory_wte_tracker.track_wte(wte).await.unwrap()) // this implementation is infallible
                }
            }
        }

        async fn cleanup(&self) -> Result<(), Self::Error> {
            match self {
                #[cfg(feature = "postgres")]
                WteTrackerVariant::Postgres(postgres_wte_tracker) => Ok(postgres_wte_tracker.cleanup().await?),
                WteTrackerVariant::Memory(memory_wte_tracker) => {
                    memory_wte_tracker.cleanup().await.unwrap(); // this implementation is infallible
                    Ok(())
                }
            }
        }
    }
}
