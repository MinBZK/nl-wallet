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

#[cfg(feature = "postgres")]
pub mod postgres {
    use std::time::Duration;

    use chrono::Utc;
    use sea_orm::{
        sea_query::OnConflict, ActiveValue, ColumnTrait, ConnectOptions, Database, DatabaseConnection, DbErr,
        EntityTrait, QueryFilter, SqlErr,
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
        T: SessionDataType + Serialize + DeserializeOwned + Send,
    {
        async fn get(&self, token: &SessionToken) -> Result<Option<SessionState<T>>, SessionStoreError> {
            // find value by token, deserialize from JSON if it exists
            let state = session_state::Entity::find_by_id((T::TYPE.to_string(), token.to_string()))
                .one(&self.connection)
                .await
                .map_err(|e| SessionStoreError::Other(e.into()))?;

            state
                .map(|state| {
                    let state = SessionState {
                        data: serde_json::from_value(state.data)?,
                        token: state.token.into(),
                        last_active: state.last_active_date_time.into(),
                    };

                    Result::<_, serde_json::Error>::Ok(state)
                })
                .transpose()
                .map_err(|e| SessionStoreError::Deserialize(Box::new(e)))
        }

        async fn write(&self, session: SessionState<T>, is_new: bool) -> Result<(), SessionStoreError> {
            // Needed for potential SessionStoreError::DuplicateToken.
            let session_token = session.token.clone();

            // Insert new value, with data serialized to JSON.
            let query = session_state::Entity::insert(session_state::ActiveModel {
                data: ActiveValue::set(
                    serde_json::to_value(session.data).map_err(|e| SessionStoreError::Serialize(Box::new(e)))?,
                ),
                r#type: ActiveValue::set(T::TYPE.to_string()),
                token: ActiveValue::set(session.token.into()),
                last_active_date_time: ActiveValue::set(session.last_active.into()),
            });

            // If this is an existing session, an update is allowed.
            let final_query = match is_new {
                true => query,
                false => query.on_conflict(
                    OnConflict::columns([session_state::PrimaryKey::Type, session_state::PrimaryKey::Token])
                        .update_columns([session_state::Column::Data, session_state::Column::LastActiveDateTime])
                        .to_owned(),
                ),
            };

            // Execute the query and handle a conflicting primary key when updates are not allowed.
            final_query.exec(&self.connection).await.map_err(|e| {
                if matches!(e.sql_err(), Some(SqlErr::UniqueConstraintViolation(_))) {
                    return SessionStoreError::DuplicateToken(session_token);
                }

                SessionStoreError::Other(e.into())
            })?;

            Ok(())
        }

        async fn cleanup(&self) -> Result<(), SessionStoreError> {
            // delete expired sessions
            let threshold = Utc::now() - chrono::Duration::minutes(SESSION_EXPIRY_MINUTES.try_into().unwrap());
            session_state::Entity::delete_many()
                .filter(session_state::Column::Type.eq(T::TYPE.to_string()))
                .filter(session_state::Column::LastActiveDateTime.lt(threshold))
                .exec(&self.connection)
                .await
                .map_err(|e| SessionStoreError::Other(e.into()))?;

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use serde::Deserialize;

        use crate::settings::Settings;

        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
                SessionToken::new_random(),
                TestData {
                    id: "hello".to_owned(),
                    data: vec![1, 2, 3],
                },
            );

            store.write(expected.clone(), true).await.unwrap();

            let actual: SessionState<TestData> = store.get(&expected.token).await.unwrap().unwrap();
            assert_eq!(actual.data, expected.data);
        }
    }
}
