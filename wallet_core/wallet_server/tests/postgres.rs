use serde::{Deserialize, Serialize};

use nl_wallet_mdoc::server_state::{SessionState, SessionStore, SessionToken};

use wallet_server::{
    settings::Settings,
    store::{postgres::PostgresSessionStore, SessionDataType},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct TestData {
    id: String,
    data: Vec<u8>,
}

impl SessionDataType for TestData {
    const TYPE: &'static str = "testdata";
}

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
