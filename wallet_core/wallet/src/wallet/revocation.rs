use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use futures::StreamExt;
use parking_lot::Mutex;
use rand::random;
use rustls_pki_types::TrustAnchor;
use tokio::sync::RwLock;
use tokio::task::AbortHandle;
use tokio::time;
use tokio::time::MissedTickBehavior;
use tracing::error;
use tracing::info;
use tracing::instrument;
use uuid::Uuid;

use error_category::ErrorCategory;
use openid4vc::disclosure_session::DisclosureClient;
use platform_support::attested_key::AttestedKeyHolder;
use token_status_list::verification::client::StatusListClient;
use token_status_list::verification::verifier::RevocationStatus;
use token_status_list::verification::verifier::RevocationVerifier;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::NotificationType;
use crate::ScheduledNotificationsCallback;
use crate::Wallet;
use crate::digid::DigidClient;
use crate::errors::StorageError;
use crate::repository::Repository;
use crate::storage::RevocationInfo;
use crate::storage::Storage;
use crate::storage::StorageState;
use crate::wallet::attestations::AttestationsCallback;
use crate::wallet::attestations::AttestationsError;
use crate::wallet::notifications::DirectNotificationsCallback;
use crate::wallet::notifications::emit_scheduled_notifications;

const STATUS_LIST_TOKEN_CACHE_CAPACITY: u64 = 100;
const STATUS_LIST_TOKEN_CACHE_DEFAULT_TTL: Duration = Duration::from_secs(180);
const STATUS_LIST_TOKEN_CACHE_ERROR_TTL: Duration = Duration::from_secs(10);

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum RevocationError {
    #[error("error emitting attestations: {0}")]
    #[category(unexpected)]
    Attestations(#[from] AttestationsError),

    #[error("storage error: {0}")]
    #[category(unexpected)]
    Storage(#[from] StorageError),
}

impl<CR, UR, S, AKH, APC, DC, IS, DCC, SLC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC, SLC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    DCC: DisclosureClient,
    SLC: StatusListClient,
{
    /// Start background revocation checks
    ///
    /// Spawns a background task that only accesses storage, not the entire wallet.
    /// Stores the ablort handle in the wallet.
    #[instrument(skip_all)]
    pub fn start_background_revocation_checks(&mut self, check_frequency: Duration)
    where
        S: Sync + 'static,
        SLC: Sync + 'static,
        CR: Send + Sync + 'static,
    {
        if self.revocation_status_job_handle.is_some() {
            info!("background revocation checks already running");
            return;
        }

        let abort_handle = Self::spawn_revocation_checks(
            Arc::clone(&self.config_repository),
            Arc::clone(&self.status_list_client),
            Arc::clone(&self.storage),
            Arc::clone(&self.attestations_callback),
            Arc::clone(&self.direct_notifications_callback),
            Arc::clone(&self.scheduled_notifications_callback),
            TimeGenerator,
            check_frequency,
        );

        self.revocation_status_job_handle = Some(abort_handle);
    }

    /// Stop background revocation checks
    #[instrument(skip_all)]
    pub fn stop_background_revocation_checks(&mut self) {
        if let Some(handle) = &self.revocation_status_job_handle.take() {
            handle.abort();
        }
    }

    #[expect(clippy::too_many_arguments)]
    fn spawn_revocation_checks<T>(
        config_repo: Arc<CR>,
        status_list_client: Arc<SLC>,
        storage: Arc<RwLock<S>>,
        attestations_callback: Arc<Mutex<Option<AttestationsCallback>>>,
        direct_notifications_callback: Arc<Mutex<Option<DirectNotificationsCallback>>>,
        scheduled_notifications_callback: Arc<Mutex<Option<ScheduledNotificationsCallback>>>,
        time_generator: T,
        check_interval: Duration,
    ) -> AbortHandle
    where
        S: Sync + 'static,
        SLC: Sync + 'static,
        CR: Send + Sync + 'static,
        T: Generator<DateTime<Utc>> + Clone + Send + Sync + 'static,
    {
        let task = tokio::spawn(async move {
            info!("wallet revocation status background job started");

            let mut interval = time::interval(check_interval);
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

            loop {
                interval.tick().await;

                let config = config_repo.get();

                if !matches!(storage.read().await.state().await, Ok(StorageState::Opened)) {
                    info!("database is not opened, skipping wallet revocation status check");
                    continue;
                }

                info!("wallet revocation status update timer expired, performing revocation check...");

                if let Err(e) = Self::check_revocations(
                    Arc::clone(&config),
                    Arc::clone(&status_list_client),
                    Arc::clone(&storage),
                    Arc::clone(&attestations_callback),
                    Arc::clone(&direct_notifications_callback),
                    Arc::clone(&scheduled_notifications_callback),
                    time_generator.clone(),
                )
                .await
                {
                    error!("background revocation check failed: {}", e);
                }
            }
        });

        task.abort_handle()
    }

    /// Perform revocation checks where all revocation info is fetched from storage
    #[expect(clippy::too_many_arguments)]
    async fn check_revocations<T>(
        config: Arc<WalletConfiguration>,
        status_list_client: Arc<SLC>,
        storage: Arc<RwLock<S>>,
        attestations_callback: Arc<Mutex<Option<AttestationsCallback>>>,
        direct_notifications_callback: Arc<Mutex<Option<DirectNotificationsCallback>>>,
        scheduled_notifications_callback: Arc<Mutex<Option<ScheduledNotificationsCallback>>>,
        time_generator: T,
    ) -> Result<(), RevocationError>
    where
        SLC: StatusListClient,
        S: Storage,
        T: Generator<DateTime<Utc>> + Clone + Send + Sync + 'static,
    {
        let revocation_verifier = RevocationVerifier::new(
            Arc::clone(&status_list_client),
            STATUS_LIST_TOKEN_CACHE_CAPACITY,
            STATUS_LIST_TOKEN_CACHE_DEFAULT_TTL,
            STATUS_LIST_TOKEN_CACHE_ERROR_TTL,
            time_generator.clone(),
        );

        // Fetch revocation info in one storage lock
        let revocation_info = storage.read().await.fetch_all_revocation_info(&time_generator).await?;

        let issuer_trust_anchors = config.issuer_trust_anchors();

        // Verify all revocations without holding any locks
        let updates: Vec<(Uuid, RevocationStatus)> = futures::stream::iter(revocation_info)
            .map(|revocation_info| {
                Self::check_revocation(
                    revocation_info,
                    &revocation_verifier,
                    &issuer_trust_anchors,
                    &time_generator,
                )
            })
            .buffer_unordered(10)
            .collect()
            .await;

        // Write all updates in one storage lock
        storage
            .write()
            .await
            .update_revocation_statuses(updates.clone())
            .await?;

        let attestations = storage.read().await.fetch_unique_attestations().await?;

        // Send direct notifications for all attestations that have been revoked. Since we don't want to hold the lock
        // across await boundaries, clone the callback Arc.
        let maybe_notifications_callback = { direct_notifications_callback.lock().clone() };
        if let Some(callback) = maybe_notifications_callback {
            let revocation_status_by_attestation_id: HashMap<Uuid, RevocationStatus> = updates.into_iter().collect();

            let notifications: Vec<(i32, NotificationType)> = attestations
                .iter()
                .filter(|attestation| {
                    revocation_status_by_attestation_id.contains_key(&attestation.attestation_copy_id())
                })
                .map(|attestation| {
                    (
                        random(),
                        NotificationType::Revoked {
                            attestation: attestation
                                .clone()
                                .into_attestation_presentation(&config.pid_attributes),
                        },
                    )
                })
                .collect();

            if !notifications.is_empty() {
                callback(notifications).await;
            }
        }

        // Schedule revocation notifications for the dashboard
        let _ = emit_scheduled_notifications(scheduled_notifications_callback, storage, &config).await;

        // Callback with the updated attestations
        if let Some(callback) = attestations_callback.lock().as_ref() {
            callback(
                attestations
                    .into_iter()
                    .map(|copy| copy.into_attestation_presentation(&config.pid_attributes))
                    .collect(),
            )
        }

        Ok(())
    }

    /// Perform revocation check using revocation info of a single attestation
    async fn check_revocation(
        revocation_info: RevocationInfo,
        revocation_verifier: &RevocationVerifier<SLC>,
        issuer_trust_anchors: &[TrustAnchor<'_>],
        time_generator: &impl Generator<DateTime<Utc>>,
    ) -> (Uuid, RevocationStatus)
    where
        SLC: StatusListClient,
    {
        let status = revocation_info
            .verify_revocation(issuer_trust_anchors, revocation_verifier, time_generator)
            .await;

        (revocation_info.attestation_copy_id(), status)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use itertools::Itertools;
    use parking_lot::Mutex;
    use tokio::time;

    use attestation_data::validity::ValidityWindow;
    use attestation_types::status_claim::StatusClaim;
    use attestation_types::status_claim::StatusListClaim;
    use crypto::server_keys::generate::Ca;
    use crypto::utils::random_string;
    use token_status_list::status_list_token::mock::create_status_list_token;
    use token_status_list::verification::client::mock::MockStatusListClient;
    use token_status_list::verification::verifier::RevocationStatus;
    use utils::generator::mock::MockTimeGenerator;

    use crate::config::default_wallet_config;
    use crate::storage::MockStorage;
    use crate::storage::StoredAttestation;
    use crate::storage::StoredAttestationCopy;
    use crate::wallet::test::TestWalletMockStorage;
    use crate::wallet::test::create_example_pid_sd_jwt;

    use super::*;

    struct TestConfigRepo(parking_lot::RwLock<WalletConfiguration>);

    impl Repository<Arc<WalletConfiguration>> for TestConfigRepo {
        fn get(&self) -> Arc<WalletConfiguration> {
            Arc::new(self.0.read().clone())
        }
    }

    #[tokio::test]
    async fn should_check_revocation_periodically() {
        // Pause time so we can advance it manually
        time::pause();

        let config_repo = Arc::new(TestConfigRepo(parking_lot::RwLock::new(default_wallet_config())));
        let status_list_client = Arc::new(MockStatusListClient::new());

        let mut storage = MockStorage::new();
        storage.expect_state().returning(|| Ok(StorageState::Opened));
        storage
            .expect_fetch_all_revocation_info()
            .returning(|_: &MockTimeGenerator| Ok(vec![]));
        storage.expect_update_revocation_statuses().returning(|_| Ok(()));
        storage.expect_fetch_unique_attestations().returning(|| Ok(vec![]));
        let storage = Arc::new(RwLock::new(storage));

        let attestations_callback = Arc::new(Mutex::new(None));
        let time_generator = MockTimeGenerator::new(Utc::now());
        let check_interval = Duration::from_millis(100);

        let attestations_counter = Arc::new(AtomicU64::new(0));
        let attestations_callback_counter = Arc::clone(&attestations_counter);
        let attestations_callback_fn: AttestationsCallback = Box::new(move |_| {
            attestations_callback_counter.fetch_add(1, Ordering::SeqCst);
        });

        let direct_notifications_callback = Arc::new(Mutex::new(None));
        let direct_notifications_counter = Arc::new(AtomicU64::new(0));
        let direct_notifications_callback_counter = Arc::clone(&direct_notifications_counter);
        let direct_notifications_callback_fn: DirectNotificationsCallback = Arc::new(move |_| {
            let counter = Arc::clone(&direct_notifications_callback_counter);
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
            })
        });

        let scheduled_notifications_callback = Arc::new(Mutex::new(None));
        let scheduled_notifications_counter = Arc::new(AtomicU64::new(0));
        let scheduled_notifications_callback_counter = Arc::clone(&scheduled_notifications_counter);
        let scheduled_notifications_callback_fn: ScheduledNotificationsCallback = Box::new(move |_| {
            scheduled_notifications_callback_counter.fetch_add(1, Ordering::SeqCst);
        });

        // Register callbacks to track updates
        attestations_callback.lock().replace(attestations_callback_fn);
        direct_notifications_callback
            .lock()
            .replace(direct_notifications_callback_fn);
        scheduled_notifications_callback
            .lock()
            .replace(scheduled_notifications_callback_fn);

        let abort_handle = TestWalletMockStorage::<TestConfigRepo>::spawn_revocation_checks(
            config_repo,
            status_list_client,
            storage,
            attestations_callback,
            direct_notifications_callback,
            scheduled_notifications_callback,
            time_generator,
            check_interval,
        );

        // Initially no checks should have occurred
        assert_eq!(0, attestations_counter.load(Ordering::SeqCst));
        assert_eq!(0, direct_notifications_counter.load(Ordering::SeqCst));
        assert_eq!(0, scheduled_notifications_counter.load(Ordering::SeqCst));

        // Advance time 10 times by 10 ms - should trigger first check
        for _ in 0..10 {
            time::advance(Duration::from_millis(10)).await;
        }

        // Should have performed at least one check
        assert!(attestations_counter.load(Ordering::SeqCst) >= 1);

        // Advance time 20 times by 10 ms - should trigger 2 more checks
        for _ in 0..20 {
            time::advance(Duration::from_millis(10)).await;
        }

        let final_count = attestations_counter.load(Ordering::SeqCst);
        assert!(final_count >= 3, "Expected at least 3 checks, got {}", final_count);

        // Since nothing is revoked, no notifications should have been emitted
        assert_eq!(0, direct_notifications_counter.load(Ordering::SeqCst));

        // Scheduled notifications should have been emitted
        assert!(scheduled_notifications_counter.load(Ordering::SeqCst) >= 3);

        abort_handle.abort();
    }

    #[tokio::test]
    async fn should_stop_checking_after_abort() {
        // Pause time so we can advance it manually
        time::pause();

        let config_repo = Arc::new(TestConfigRepo(parking_lot::RwLock::new(default_wallet_config())));
        let status_list_client = Arc::new(MockStatusListClient::new());

        let mut storage = MockStorage::new();
        storage.expect_state().returning(|| Ok(StorageState::Opened));
        storage
            .expect_fetch_all_revocation_info()
            .returning(|_: &MockTimeGenerator| Ok(vec![]));
        storage.expect_update_revocation_statuses().returning(|_| Ok(()));
        storage.expect_fetch_unique_attestations().returning(|| Ok(vec![]));
        let storage = Arc::new(RwLock::new(storage));

        let callback = Arc::new(Mutex::new(None));
        let time_generator = MockTimeGenerator::new(Utc::now());
        let check_interval = Duration::from_millis(100);

        let counter = Arc::new(AtomicU64::new(0));
        let callback_counter = Arc::clone(&counter);

        let callback_fn: AttestationsCallback = Box::new(move |_| {
            callback_counter.fetch_add(1, Ordering::SeqCst);
        });

        let notifications_callback = Arc::default();

        // Register callback to track updates
        callback.lock().replace(callback_fn);

        let abort_handle = TestWalletMockStorage::spawn_revocation_checks(
            config_repo,
            status_list_client,
            storage,
            callback,
            notifications_callback,
            Arc::default(),
            time_generator,
            check_interval,
        );

        // Advance time to trigger multiple checks
        for _ in 0..30 {
            time::advance(Duration::from_millis(10)).await;
        }

        let count_before_abort = counter.load(Ordering::SeqCst);
        assert!(count_before_abort > 0, "Should have performed checks before abort");

        // Abort the task
        abort_handle.abort();

        // Advance time again
        for _ in 0..30 {
            time::advance(Duration::from_millis(10)).await;
        }

        // Count should not have changed after abort
        assert_eq!(
            count_before_abort,
            counter.load(Ordering::SeqCst),
            "Count should not change after abort"
        );
    }

    #[tokio::test]
    async fn should_update_storage_with_revocation_statuses() {
        // Pause time so we can advance it manually
        time::pause();

        let ca = Ca::generate("test", Default::default()).unwrap();
        let keypair = ca.generate_status_list_mock().unwrap();

        let mut wallet_config = default_wallet_config();
        wallet_config.issuer_trust_anchors = vec![ca.borrowing_trust_anchor().clone()];
        let config_repo = Arc::new(TestConfigRepo(parking_lot::RwLock::new(wallet_config)));

        let (_, _, status_list_token) = create_status_list_token(&keypair, None, None).await;
        let mut mock_status_list_client = MockStatusListClient::new();
        mock_status_list_client
            .expect_fetch()
            .returning(move |_| Ok(status_list_token.clone()));
        let status_list_client = Arc::new(mock_status_list_client);

        let update_count = Arc::new(AtomicU64::new(0));
        let update_counter = Arc::clone(&update_count);

        let mut storage = MockStorage::new();
        storage.expect_state().returning(|| Ok(StorageState::Opened));

        let revocation_info_id_1 = Uuid::new_v4();
        let revocation_info_id_2 = Uuid::new_v4();
        let test_revocation_info = vec![
            RevocationInfo::new(
                revocation_info_id_1,
                StatusClaim::new_mock(),
                keypair.certificate().distinguished_name_canonical().unwrap(),
            ),
            RevocationInfo::new(
                revocation_info_id_2,
                StatusClaim::StatusList(StatusListClaim {
                    idx: 3,
                    uri: "https://example.com/statuslists/1".parse().unwrap(),
                }),
                keypair.certificate().distinguished_name_canonical().unwrap(),
            ),
        ];

        storage
            .expect_fetch_all_revocation_info()
            .returning(move |_: &MockTimeGenerator| Ok(test_revocation_info.clone()));
        storage.expect_update_revocation_statuses().returning(move |updates| {
            update_counter.fetch_add(1, Ordering::SeqCst);
            assert_eq!(
                vec![RevocationStatus::Valid, RevocationStatus::Revoked],
                updates.into_iter().map(|(_, status)| status).collect_vec()
            );
            Ok(())
        });

        let (sd_jwt, sd_jwt_metadata) = create_example_pid_sd_jwt();
        let attestations = vec![
            StoredAttestationCopy::new(
                Uuid::new_v4(),
                revocation_info_id_1,
                ValidityWindow::new_valid_mock(),
                StoredAttestation::SdJwt {
                    key_identifier: random_string(16),
                    sd_jwt: sd_jwt.clone(),
                },
                sd_jwt_metadata.clone(),
                Some(RevocationStatus::Valid),
            ),
            StoredAttestationCopy::new(
                Uuid::new_v4(),
                revocation_info_id_2,
                ValidityWindow::new_valid_mock(),
                StoredAttestation::SdJwt {
                    key_identifier: random_string(16),
                    sd_jwt,
                },
                sd_jwt_metadata,
                Some(RevocationStatus::Revoked),
            ),
        ];
        storage
            .expect_fetch_unique_attestations()
            .returning(move || Ok(attestations.clone()));
        let storage = Arc::new(RwLock::new(storage));

        let time_generator = MockTimeGenerator::new(Utc::now());
        let check_interval = Duration::from_millis(100);

        let attestations_callback = Arc::default();

        let notifications_counter = Arc::new(AtomicU64::new(0));
        let notifications_callback_counter = Arc::clone(&notifications_counter);
        let notifications_callback_fn: DirectNotificationsCallback = Arc::new(move |_| {
            let counter = Arc::clone(&notifications_callback_counter);
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
            })
        });
        let notifications_callback = Arc::new(Mutex::new(Some(notifications_callback_fn)));

        let abort_handle = TestWalletMockStorage::spawn_revocation_checks(
            config_repo,
            status_list_client,
            storage,
            attestations_callback,
            notifications_callback,
            Arc::default(),
            time_generator,
            check_interval,
        );

        // Advance time to trigger checks
        for _ in 0..30 {
            time::advance(Duration::from_millis(10)).await;
        }

        // Should have called update_revocation_statuses
        assert!(
            update_count.load(Ordering::SeqCst) > 0,
            "Should have updated revocation statuses"
        );

        assert!(
            notifications_counter.load(Ordering::SeqCst) >= 1,
            "Should have sent revocation notifications"
        );

        abort_handle.abort();
    }
}
