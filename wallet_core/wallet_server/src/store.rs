use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use nl_wallet_mdoc::{
    server_state::{MemorySessionStore, SessionState, SessionStore, SessionStoreError, SessionToken},
    verifier::DisclosureData,
};

#[cfg(feature = "issuance")]
use openid4vc::issuer::IssuanceData;

#[cfg(feature = "postgres")]
use crate::store::postgres::PostgresSessionStore;

pub trait SessionDataType {
    const TYPE: &'static str;
}

impl SessionDataType for DisclosureData {
    const TYPE: &'static str = "mdoc_disclosure";
}

#[cfg(feature = "issuance")]
impl SessionDataType for openid4vc::issuer::IssuanceData {
    const TYPE: &'static str = "openid4vci_issuance";
}

pub struct SessionStores {
    pub disclosure: SessionStoreVariant<DisclosureData>,

    #[cfg(feature = "issuance")]
    pub issuance: SessionStoreVariant<IssuanceData>,
}

impl SessionStores {
    pub async fn init(url: Url) -> Result<SessionStores, anyhow::Error> {
        match url.scheme() {
            #[cfg(feature = "postgres")]
            "postgres" => {
                let store = PostgresSessionStore::try_new(url).await?;
                Ok(SessionStores {
                    #[cfg(feature = "issuance")]
                    issuance: SessionStoreVariant::Postgres(store.clone()),
                    disclosure: SessionStoreVariant::Postgres(store),
                })
            }
            "memory" => Ok(SessionStores {
                #[cfg(feature = "issuance")]
                issuance: SessionStoreVariant::Memory(MemorySessionStore::new()),
                disclosure: SessionStoreVariant::Memory(MemorySessionStore::new()),
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
    T: SessionDataType + Clone + Serialize + DeserializeOwned + Send + Sync,
{
    async fn get(&self, token: &SessionToken) -> Result<Option<SessionState<T>>, SessionStoreError> {
        match self {
            #[cfg(feature = "postgres")]
            SessionStoreVariant::Postgres(postgres) => postgres.get(token).await,
            SessionStoreVariant::Memory(memory) => memory.get(token).await,
        }
    }

    async fn write(&self, session: &SessionState<T>) -> Result<(), SessionStoreError> {
        match self {
            #[cfg(feature = "postgres")]
            SessionStoreVariant::Postgres(postgres) => postgres.write(session).await,
            SessionStoreVariant::Memory(memory) => memory.write(session).await,
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

#[cfg(feature = "postgres")]
pub mod postgres {
    use std::time::Duration;

    use chrono::Utc;
    use sea_orm::{
        sea_query::OnConflict, ActiveValue, ColumnTrait, ConnectOptions, Database, DatabaseConnection, DbErr,
        EntityTrait, QueryFilter,
    };
    use serde::{de::DeserializeOwned, Serialize};
    use tracing::log::LevelFilter;
    use url::Url;

    use crate::entity::session_state;
    use nl_wallet_mdoc::server_state::{
        SessionState, SessionStore, SessionStoreError, SessionToken, SESSION_EXPIRY_MINUTES,
    };

    use super::SessionDataType;

    const DB_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

    #[derive(Debug, Clone)]
    pub struct PostgresSessionStore {
        connection: DatabaseConnection,
    }

    impl PostgresSessionStore {
        pub async fn try_new(url: Url) -> Result<Self, DbErr> {
            let mut connection_options = ConnectOptions::new(url);
            connection_options
                .connect_timeout(DB_CONNECT_TIMEOUT)
                .sqlx_logging(true)
                .sqlx_logging_level(LevelFilter::Trace);

            let connection = Database::connect(connection_options).await?;

            Ok(Self { connection })
        }
    }

    impl<T> SessionStore<T> for PostgresSessionStore
    where
        T: SessionDataType + Clone + Serialize + DeserializeOwned + Send + Sync,
    {
        async fn get(&self, token: &SessionToken) -> Result<Option<SessionState<T>>, SessionStoreError> {
            // find value by token, deserialize from JSON if it exists
            let state = session_state::Entity::find_by_id((T::TYPE.to_string(), token.to_string()))
                .one(&self.connection)
                .await
                .map_err(|e| SessionStoreError::Other(e.into()))?;

            state
                .map(|s| serde_json::from_value(s.data))
                .transpose()
                .map_err(|e| SessionStoreError::Deserialize(Box::new(e)))
        }

        async fn write(&self, session: &SessionState<T>) -> Result<(), SessionStoreError> {
            // insert new value (serialized to JSON), update on conflicting session token
            session_state::Entity::insert(session_state::ActiveModel {
                data: ActiveValue::set(
                    serde_json::to_value(session.clone()).map_err(|e| SessionStoreError::Serialize(Box::new(e)))?,
                ),
                r#type: ActiveValue::set(T::TYPE.to_string()),
                token: ActiveValue::set(session.token.to_string()),
                expiration_date_time: ActiveValue::set(
                    (session.last_active + chrono::Duration::minutes(SESSION_EXPIRY_MINUTES as i64)).into(),
                ),
            })
            .on_conflict(
                OnConflict::columns([session_state::PrimaryKey::Type, session_state::PrimaryKey::Token])
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
                .filter(session_state::Column::Type.eq(T::TYPE.to_string()))
                .filter(session_state::Column::ExpirationDateTime.lt(Utc::now()))
                .exec(&self.connection)
                .await
                .map_err(|e| SessionStoreError::Other(e.into()))?;

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::settings::Settings;

        use super::*;
        use serde::Deserialize;

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct TestData {
            id: String,
            data: Vec<u8>,
        }

        impl SessionDataType for TestData {
            const TYPE: &'static str = "testdata";
        }

        #[cfg_attr(not(feature = "db_test"), ignore)]
        #[tokio::test]
        async fn test_write() {
            let settings = Settings::new().unwrap();
            let store = PostgresSessionStore::try_new(settings.store_url).await.unwrap();

            let expected = SessionState::<TestData>::new(
                SessionToken::new(),
                TestData {
                    id: "hello".to_owned(),
                    data: vec![1, 2, 3],
                },
            );

            store.write(&expected).await.unwrap();

            let actual: SessionState<TestData> = store.get(&expected.token).await.unwrap().unwrap();
            assert_eq!(actual.session_data, expected.session_data);
        }
    }
}
