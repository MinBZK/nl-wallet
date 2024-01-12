use url::Url;

use nl_wallet_mdoc::{
    server_state::{MemorySessionStore, SessionState, SessionStore, SessionStoreError, SessionToken},
    verifier::DisclosureData,
};

#[cfg(feature = "postgres")]
use crate::store::postgres::PostgresSessionStore;

/// This enum effectively switches between the different types that implement `DisclosureSessionStore`,
/// by implementing this trait itself and forwarding the calls to the type contained in the invariant.
pub enum DisclosureSessionStore {
    #[cfg(feature = "postgres")]
    Postgres(PostgresSessionStore<DisclosureData>),
    Memory(MemorySessionStore<DisclosureData>),
}

impl DisclosureSessionStore {
    pub async fn init(url: Url) -> anyhow::Result<Self> {
        let session_store = match url.scheme() {
            #[cfg(feature = "postgres")]
            "postgres" => DisclosureSessionStore::Postgres(PostgresSessionStore::connect(url).await?),
            "memory" => DisclosureSessionStore::Memory(MemorySessionStore::new()),
            e => unimplemented!("{}", e),
        };

        Ok(session_store)
    }
}

impl SessionStore for DisclosureSessionStore {
    type Data = SessionState<DisclosureData>;

    async fn get(&self, id: &SessionToken) -> Result<Option<Self::Data>, SessionStoreError> {
        match self {
            #[cfg(feature = "postgres")]
            DisclosureSessionStore::Postgres(postgres) => postgres.get(id).await,
            DisclosureSessionStore::Memory(memory) => memory.get(id).await,
        }
    }

    async fn write(&self, session: &Self::Data) -> Result<(), SessionStoreError> {
        match self {
            #[cfg(feature = "postgres")]
            DisclosureSessionStore::Postgres(postgres) => postgres.write(session).await,
            DisclosureSessionStore::Memory(memory) => memory.write(session).await,
        }
    }

    async fn cleanup(&self) -> Result<(), SessionStoreError> {
        match self {
            #[cfg(feature = "postgres")]
            DisclosureSessionStore::Postgres(postgres) => postgres.cleanup().await,
            DisclosureSessionStore::Memory(memory) => memory.cleanup().await,
        }
    }
}

#[cfg(feature = "postgres")]
pub mod postgres {
    use std::{marker::PhantomData, time::Duration};

    use chrono::Utc;
    use sea_orm::{
        sea_query::OnConflict, ActiveValue, ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait,
        QueryFilter,
    };
    use serde::{de::DeserializeOwned, Serialize};
    use tracing::log::LevelFilter;
    use url::Url;

    use crate::entity::session_state;
    use nl_wallet_mdoc::server_state::{
        SessionState, SessionStore, SessionStoreError, SessionToken, SESSION_EXPIRY_MINUTES,
    };

    const DB_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

    pub struct PostgresSessionStore<T> {
        connection: DatabaseConnection,
        _marker: PhantomData<T>,
    }

    impl<T> PostgresSessionStore<T> {
        pub async fn connect(url: Url) -> anyhow::Result<Self> {
            let mut connection_options = ConnectOptions::new(url);
            connection_options
                .connect_timeout(DB_CONNECT_TIMEOUT)
                .sqlx_logging(true)
                .sqlx_logging_level(LevelFilter::Trace);

            let db = Database::connect(connection_options).await?;
            Ok(Self {
                connection: db,
                _marker: PhantomData,
            })
        }
    }

    impl<T: Clone + Serialize + DeserializeOwned + Send + Sync> SessionStore for PostgresSessionStore<T> {
        type Data = SessionState<T>;

        async fn get(&self, token: &SessionToken) -> Result<Option<Self::Data>, SessionStoreError> {
            // find value by token, deserialize from JSON if it exists
            let state = session_state::Entity::find()
                .filter(session_state::Column::Token.eq(token.to_string()))
                .one(&self.connection)
                .await
                .map_err(|e| SessionStoreError::Other(e.into()))?;

            state
                .map(|s| serde_json::from_value(s.data))
                .transpose()
                .map_err(|e| SessionStoreError::Deserialize(Box::new(e)))
        }

        async fn write(&self, session: &Self::Data) -> Result<(), SessionStoreError> {
            // insert new value (serialized to JSON), update on conflicting session token
            session_state::Entity::insert(session_state::ActiveModel {
                data: ActiveValue::set(
                    serde_json::to_value(session.clone()).map_err(|e| SessionStoreError::Serialize(Box::new(e)))?,
                ),
                token: ActiveValue::set(session.token.to_string()),
                expiration_date_time: ActiveValue::set(
                    (session.last_active + chrono::Duration::minutes(SESSION_EXPIRY_MINUTES as i64)).into(),
                ),
            })
            .on_conflict(
                OnConflict::column(session_state::Column::Token)
                    .update_columns([session_state::Column::Data, session_state::Column::ExpirationDateTime])
                    .to_owned(),
            )
            .exec(&self.connection)
            .await
            .map_err(|e| SessionStoreError::Other(e.into()))?;

            Ok(())
        }

        async fn cleanup(&self) -> Result<(), SessionStoreError> {
            // delete expired sessions
            session_state::Entity::delete_many()
                .filter(session_state::Column::ExpirationDateTime.lt(Utc::now()))
                .exec(&self.connection)
                .await
                .map_err(|e| SessionStoreError::Other(e.into()))?;

            Ok(())
        }
    }
}
