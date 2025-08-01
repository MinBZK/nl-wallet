use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use parking_lot::RwLock;
use serde::Deserialize;
use serde::Serialize;

use serial_test::parallel;
use serial_test::serial;

use openid4vc::server_state::Expirable;
use openid4vc::server_state::HasProgress;
use openid4vc::server_state::Progress;
use openid4vc::server_state::SessionDataType;
use openid4vc::server_state::SessionStoreTimeouts;
use openid4vc::server_state::test;
use openid4vc::server_state::test::RandomData;
use server_utils::settings::ServerSettings;
use server_utils::settings::Storage;
use server_utils::store::postgres;
use server_utils::store::postgres::PostgresSessionStore;
use utils::generator::mock::MockTimeGenerator;
use verification_server::settings::VerifierSettings;

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
            data: crypto::utils::random_bytes(32),
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
        self.is_expired = true;
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
    VerifierSettings::new("verification_server.toml", "verification_server")
        .unwrap()
        .server_settings
        .storage
}

async fn postgres_session_store() -> PostgresSessionStore {
    let storage_settings = storage_settings();
    let timeouts = SessionStoreTimeouts::from(&storage_settings);

    PostgresSessionStore::new(postgres::new_connection(storage_settings.url).await.unwrap(), timeouts)
}

type SessionStoreWithMockTime = (PostgresSessionStore<MockTimeGenerator>, Arc<RwLock<DateTime<Utc>>>);

async fn postgres_session_store_with_mock_time() -> SessionStoreWithMockTime {
    let time_generator = MockTimeGenerator::default();
    let mock_time = Arc::clone(&time_generator.time);

    let storage_settings = storage_settings();
    let timeouts = SessionStoreTimeouts::from(&storage_settings);

    let session_store = PostgresSessionStore::new_with_time(
        postgres::new_connection(storage_settings.url).await.unwrap(),
        timeouts,
        time_generator,
    );

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
