use std::sync::Arc;

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use serial_test::{parallel, serial};

use nl_wallet_mdoc::utils::mock_time::MockTimeGenerator;
use openid4vc::server_state::{
    test::{self, RandomData},
    Expirable, HasProgress, Progress, SessionStoreTimeouts,
};
use wallet_common::utils;
use wallet_server::{
    settings::{Settings, Storage},
    store::{postgres::PostgresSessionStore, SessionDataType},
};

/// A mock data type that adheres to all the trait bounds necessary for testing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MockSessionData {
    #[serde(with = "ProgressDef")]
    progress: Progress,
    is_expired: bool,
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
            is_expired: false,
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

impl Expirable for MockSessionData {
    fn is_expired(&self) -> bool {
        self.is_expired
    }

    fn expire(&mut self) {
        self.is_expired = true
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

fn storage_settings() -> Storage {
    Settings::new_custom("ws_integration_test.toml", "ws_integration_test")
        .unwrap()
        .storage
}

async fn postgres_session_store() -> PostgresSessionStore {
    let storage_settings = storage_settings();
    let timeouts = SessionStoreTimeouts::from(&storage_settings);

    PostgresSessionStore::try_new(storage_settings.url, timeouts)
        .await
        .unwrap()
}

type SessionStoreWithMockTime = (PostgresSessionStore<MockTimeGenerator>, Arc<RwLock<DateTime<Utc>>>);

async fn postgres_session_store_with_mock_time() -> SessionStoreWithMockTime {
    let time_generator = MockTimeGenerator::default();
    let mock_time = Arc::clone(&time_generator.time);

    let storage_settings = storage_settings();
    let timeouts = SessionStoreTimeouts::from(&storage_settings);

    let session_store = PostgresSessionStore::try_new_with_time(storage_settings.url, timeouts, time_generator)
        .await
        .unwrap();

    (session_store, mock_time)
}

#[tokio::test]
#[parallel(cleanup)]
async fn test_get_write() {
    let session_store = postgres_session_store().await;

    test::test_session_store_get_write::<MockSessionData>(&session_store).await;
}

#[tokio::test]
#[serial(cleanup)]
async fn test_cleanup_expiration() {
    let (session_store, mock_time) = postgres_session_store_with_mock_time().await;

    test::test_session_store_cleanup_expiration::<MockSessionData>(
        &session_store,
        &session_store.timeouts,
        mock_time.as_ref(),
    )
    .await;
}

#[tokio::test]
#[serial(cleanup)]
async fn test_cleanup_successful_deletion() {
    let (session_store, mock_time) = postgres_session_store_with_mock_time().await;

    test::test_session_store_cleanup_successful_deletion::<MockSessionData>(
        &session_store,
        &session_store.timeouts,
        mock_time.as_ref(),
    )
    .await;
}

#[tokio::test]
#[serial(cleanup)]
async fn test_cleanup_failed_deletion() {
    let (session_store, mock_time) = postgres_session_store_with_mock_time().await;

    test::test_session_store_cleanup_failed_deletion::<MockSessionData>(
        &session_store,
        &session_store.timeouts,
        mock_time.as_ref(),
    )
    .await;
}
