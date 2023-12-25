use std::sync::Arc;

use openid4vc::issuer::IssuanceData;
use url::Url;

#[cfg(feature = "postgres")]
use crate::store::postgres::PostgresSessionStore;
use nl_wallet_mdoc::{
    server_state::{MemorySessionStore, SessionState, SessionStore},
    verifier::DisclosureData,
};

use self::postgres::connect;

pub type BoxedSessionStore<T> = Box<dyn SessionStore<Data = SessionState<T>> + Send + Sync>;

pub struct SessionStores {
    pub disclosure: BoxedSessionStore<DisclosureData>,
    pub issuance: BoxedSessionStore<IssuanceData>,
}

pub async fn new_session_stores(url: Url) -> Result<SessionStores, anyhow::Error> {
    match url.scheme() {
        #[cfg(feature = "postgres")]
        "postgres" => {
            let db = Arc::new(connect(url).await?);
            Ok(SessionStores {
                disclosure: Box::new(PostgresSessionStore::new(db.clone())),
                issuance: Box::new(PostgresSessionStore::new(db)),
            })
        }
        "memory" => Ok(SessionStores {
            disclosure: Box::new(MemorySessionStore::new()),
            issuance: Box::new(MemorySessionStore::new()),
        }),
        e => unimplemented!("{}", e),
    }
}

#[cfg(feature = "postgres")]
pub mod postgres {
    use std::{marker::PhantomData, sync::Arc, time::Duration};

    use axum::async_trait;
    use chrono::Utc;
    use openid4vc::issuer::IssuanceData;
    use sea_orm::{
        sea_query::OnConflict, ActiveValue, ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait,
        QueryFilter,
    };
    use serde::{de::DeserializeOwned, Serialize};
    use tracing::log::LevelFilter;
    use url::Url;

    use crate::entity::session_state;
    use nl_wallet_mdoc::{
        server_state::{SessionState, SessionStore, SessionStoreError, SessionToken, SESSION_EXPIRY_MINUTES},
        verifier::DisclosureData,
    };

    const DB_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

    trait SessionDataType {
        const TYPE: &'static str;
    }

    impl SessionDataType for DisclosureData {
        const TYPE: &'static str = "mdoc_disclosure";
    }

    impl SessionDataType for IssuanceData {
        const TYPE: &'static str = "openid4vci_issuance";
    }

    pub struct PostgresSessionStore<T> {
        connection: Arc<DatabaseConnection>,
        _marker: PhantomData<T>,
    }

    impl<T> PostgresSessionStore<T> {
        pub fn new(db: Arc<DatabaseConnection>) -> Self {
            Self {
                connection: db,
                _marker: PhantomData,
            }
        }
    }

    #[async_trait]
    impl<T> SessionStore for PostgresSessionStore<T>
    where
        T: SessionDataType + Clone + Serialize + DeserializeOwned + Send + Sync,
    {
        type Data = SessionState<T>;

        async fn get(&self, token: &SessionToken) -> Result<Option<Self::Data>, SessionStoreError> {
            // find value by token, deserialize from JSON if it exists
            let state = session_state::Entity::find()
                .filter(session_state::Column::Token.eq(token.to_string()))
                .filter(session_state::Column::Type.eq(T::TYPE.to_string()))
                .one(self.connection.as_ref())
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
                r#type: ActiveValue::set(T::TYPE.to_string()),
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
            .exec(self.connection.as_ref())
            .await
            .map_err(|e| SessionStoreError::Other(e.into()))?;

            Ok(())
        }

        async fn cleanup(&self) -> Result<(), SessionStoreError> {
            // delete expired sessions
            session_state::Entity::delete_many()
                .filter(session_state::Column::ExpirationDateTime.lt(Utc::now()))
                .exec(self.connection.as_ref())
                .await
                .map_err(|e| SessionStoreError::Other(e.into()))?;

            Ok(())
        }
    }

    pub async fn connect(url: Url) -> Result<DatabaseConnection, anyhow::Error> {
        let mut connection_options = ConnectOptions::new(url);
        connection_options
            .connect_timeout(DB_CONNECT_TIMEOUT)
            .sqlx_logging(true)
            .sqlx_logging_level(LevelFilter::Trace);

        let db = Database::connect(connection_options).await?;
        Ok(db)
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
            let db = Arc::new(connect(settings.store_url).await.unwrap());
            let store = PostgresSessionStore::<TestData>::new(db);

            let expected = SessionState::<TestData>::new(
                SessionToken::new(),
                TestData {
                    id: "hello".to_owned(),
                    data: vec![1, 2, 3],
                },
            );

            store.write(&expected).await.unwrap();

            let actual = store.get(&expected.token).await.unwrap().unwrap();
            assert_eq!(actual.session_data, expected.session_data);
        }
    }
}
