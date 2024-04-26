use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use nl_wallet_mdoc::server_state::{
    test::{self, RandomData},
    HasProgress, Progress,
};

use wallet_common::utils;
use wallet_server::{
    settings::Settings,
    store::{
        postgres::{PostgresSessionStore, POSTGRES_SESSION_STORE_NOW},
        SessionDataType,
    },
};

/// A mock data type that adheres to all the trait bounds necessary for testing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MockSessionData {
    #[serde(with = "ProgressDef")]
    progress: Progress,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Progress")]
pub enum ProgressDef {
    Active,
    Finished { has_succeeded: bool },
}

impl MockSessionData {
    fn new(progress: Progress) -> Self {
        Self {
            progress,
            data: utils::random_bytes(32),
        }
    }
}

impl From<Progress> for MockSessionData {
    fn from(value: Progress) -> Self {
        Self::new(value)
    }
}

impl HasProgress for MockSessionData {
    fn progress(&self) -> Progress {
        self.progress
    }
}

impl RandomData for MockSessionData {
    fn new_random() -> Self {
        Self::new(Progress::Active)
    }
}

impl SessionDataType for MockSessionData {
    const TYPE: &'static str = "mockdata";
}

async fn postgres_session_store() -> PostgresSessionStore {
    let storage_settings = Settings::new().unwrap().storage;
    let timeouts = (&storage_settings).into();

    PostgresSessionStore::try_new(storage_settings.url, timeouts)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_get_write() {
    let session_store = postgres_session_store().await;

    test::test_session_store_get_write::<MockSessionData>(&session_store).await;
}

#[tokio::test]
async fn test_cleanup() {
    let session_store = postgres_session_store().await;
    let mock_now = Lazy::force(&POSTGRES_SESSION_STORE_NOW);

    test::test_session_store_cleanup_expiration::<MockSessionData>(&session_store, &session_store.timeouts, mock_now)
        .await;
    test::test_session_store_cleanup_successful_deletion::<MockSessionData>(
        &session_store,
        &session_store.timeouts,
        mock_now,
    )
    .await;
    test::test_session_store_cleanup_failed_deletion::<MockSessionData>(
        &session_store,
        &session_store.timeouts,
        mock_now,
    )
    .await;
}
