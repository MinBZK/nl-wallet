use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use futures::StreamExt;
use parking_lot::Mutex;
use rand::random;
use rustls_pki_types::TrustAnchor;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time;
use tokio::time::MissedTickBehavior;
use tracing::error;
use tracing::info;
use tracing::instrument;
use tracing::warn;
use uuid::Uuid;

use error_category::ErrorCategory;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::oidc::OidcClient;
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

/// Groups dependencies for revocation checks
struct RevocationTaskContext<S, SLC, CR> {
    config_repo: Arc<CR>,
    status_list_client: Arc<SLC>,
    storage: Arc<RwLock<S>>,
    attestations_callback: Arc<Mutex<Option<AttestationsCallback>>>,
    direct_notifications_callback: Arc<Mutex<Option<DirectNotificationsCallback>>>,
    scheduled_notifications_callback: Arc<Mutex<Option<ScheduledNotificationsCallback>>>,
}

impl<S, SLC, CR> Clone for RevocationTaskContext<S, SLC, CR> {
    fn clone(&self) -> Self {
        Self {
            config_repo: Arc::clone(&self.config_repo),
            status_list_client: Arc::clone(&self.status_list_client),
            storage: Arc::clone(&self.storage),
            attestations_callback: Arc::clone(&self.attestations_callback),
            direct_notifications_callback: Arc::clone(&self.direct_notifications_callback),
            scheduled_notifications_callback: Arc::clone(&self.scheduled_notifications_callback),
        }
    }
}

impl<CR, UR, S, AKH, APC, OC, IS, DCC, SLC> Wallet<CR, UR, S, AKH, APC, OC, IS, DCC, SLC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    S: Storage,
    AKH: AttestedKeyHolder,
    OC: OidcClient,
    DCC: DisclosureClient,
    SLC: StatusListClient,
{
    /// Start background revocation checks
    ///
    /// Spawns a background task that only accesses storage, not the entire wallet.
    /// Stores the abort handle in the wallet.
    #[instrument(skip_all)]
    pub fn start_background_revocation_checks(&mut self, check_frequency: Duration)
    where
        S: Sync + 'static,
        SLC: Sync + 'static,
        CR: Send + Sync + 'static,
    {
        if let Some(handle) = &self.revocation_status_job_handle {
            if !handle.is_finished() {
                info!("background revocation checks already running");
                return;
            }
            warn!("previous background revocation task has stopped unexpectedly, restarting");
        }

        let ctx = self.get_revocation_context();
        let config_repo = Arc::clone(&self.config_repository);

        info!("wallet revocation status background job started");

        let handle = spawn_periodic_task(check_frequency, move || {
            let ctx = ctx.clone();
            let config = config_repo.get();
            async move {
                if !matches!(ctx.storage.read().await.state().await, Ok(StorageState::Opened)) {
                    info!("database is not opened, skipping wallet revocation status check");
                    return;
                }
                info!("wallet revocation status update timer expired, performing revocation check...");
                if let Err(e) = Self::check_revocations(&ctx, Arc::clone(&config), TimeGenerator).await {
                    error!("background revocation check failed: {}", e);
                }
            }
        });

        self.revocation_status_job_handle = Some(handle);
    }

    /// Asynchronously performs revocation checks
    #[instrument(skip_all)]
    pub async fn perform_revocation_checks(&mut self) -> Result<(), RevocationError>
    where
        S: Sync + 'static,
        SLC: Sync + 'static,
        CR: Send + Sync + 'static,
    {
        info!("performing wallet revocation check");

        let config = self.config_repository.get();

        Self::check_revocations(&self.get_revocation_context(), Arc::clone(&config), TimeGenerator).await
    }

    /// Stop background revocation checks
    #[instrument(skip_all)]
    pub fn stop_background_revocation_checks(&mut self) {
        if let Some(handle) = &self.revocation_status_job_handle.take() {
            handle.abort();
        }
    }

    fn get_revocation_context(&self) -> RevocationTaskContext<S, SLC, CR> {
        RevocationTaskContext {
            config_repo: Arc::clone(&self.config_repository),
            status_list_client: Arc::clone(&self.status_list_client),
            storage: Arc::clone(&self.storage),
            attestations_callback: Arc::clone(&self.attestations_callback),
            direct_notifications_callback: Arc::clone(&self.direct_notifications_callback),
            scheduled_notifications_callback: Arc::clone(&self.scheduled_notifications_callback),
        }
    }

    /// Perform revocation checks where all revocation info is fetched from storage
    async fn check_revocations<T>(
        ctx: &RevocationTaskContext<S, SLC, CR>,
        config: Arc<WalletConfiguration>,
        time_generator: T,
    ) -> Result<(), RevocationError>
    where
        SLC: StatusListClient,
        S: Storage,
        T: Generator<DateTime<Utc>> + Clone + Send + Sync + 'static,
    {
        let revocation_verifier = RevocationVerifier::new(
            Arc::clone(&ctx.status_list_client),
            STATUS_LIST_TOKEN_CACHE_CAPACITY,
            STATUS_LIST_TOKEN_CACHE_DEFAULT_TTL,
            STATUS_LIST_TOKEN_CACHE_ERROR_TTL,
            time_generator.clone(),
        );

        // Fetch revocation info in one storage lock
        let revocation_info = ctx
            .storage
            .read()
            .await
            .fetch_all_revocation_info(&time_generator)
            .await?;

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
        ctx.storage
            .write()
            .await
            .update_revocation_statuses(updates.clone())
            .await?;

        let attestations = ctx.storage.read().await.fetch_unique_attestations().await?;

        // Send direct notifications for all attestations that have been revoked. Since we don't want to hold the lock
        // across await boundaries, clone the callback Arc.
        let maybe_notifications_callback = { ctx.direct_notifications_callback.lock().clone() };
        if let Some(callback) = maybe_notifications_callback {
            let revocation_status_by_attestation_id: HashMap<Uuid, RevocationStatus> = updates.into_iter().collect();

            let notifications: Vec<(i32, NotificationType)> = attestations
                .iter()
                .filter(|attestation| {
                    revocation_status_by_attestation_id
                        .get(&attestation.attestation_copy_id())
                        .is_some_and(|status| *status == RevocationStatus::Revoked)
                })
                .map(|attestation| {
                    let attestation = attestation
                        .clone()
                        .into_attestation_presentation(&config.pid_attributes);

                    (random(), NotificationType::Revoked { attestation })
                })
                .collect();

            if !notifications.is_empty() {
                callback(notifications).await;
            }
        }

        // Schedule revocation notifications for the dashboard and to cancel existing expiration notifications
        let _ = emit_scheduled_notifications(
            ctx.scheduled_notifications_callback.clone(),
            ctx.storage.clone(),
            &config,
        )
        .await;

        // Callback with the updated attestations
        if let Some(callback) = ctx.attestations_callback.lock().as_ref() {
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

fn spawn_periodic_task<F, Fut>(check_interval: Duration, task: F) -> JoinHandle<()>
where
    F: Fn() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send,
{
    tokio::spawn(async move {
        let mut interval = time::interval(check_interval);
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            task().await;
        }
    })
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

    /// Shared setup for revocation tests
    async fn setup_revocation_test_env() -> (Arc<TestConfigRepo>, Arc<MockStatusListClient>, MockStorage, Uuid, Uuid) {
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

        let mut storage = MockStorage::new();

        let revocation_id_1 = Uuid::new_v4();
        let revocation_id_2 = Uuid::new_v4();

        let test_revocation_info = vec![
            RevocationInfo::new(
                revocation_id_1,
                StatusClaim::new_mock(),
                keypair.certificate().distinguished_name_canonical().unwrap(),
            ),
            RevocationInfo::new(
                revocation_id_2,
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

        let (sd_jwt, sd_jwt_metadata) = create_example_pid_sd_jwt();
        let attestations = vec![
            StoredAttestationCopy::new(
                Uuid::new_v4(),
                revocation_id_1,
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
                revocation_id_2,
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

        (
            config_repo,
            Arc::new(mock_status_list_client),
            storage,
            revocation_id_1,
            revocation_id_2,
        )
    }

    #[tokio::test]
    async fn should_update_revocation_statuses() {
        let (config_repo, status_list_client, mut storage, _, _) = setup_revocation_test_env().await;

        storage.expect_update_revocation_statuses().returning(|updates| {
            assert_eq!(
                vec![RevocationStatus::Valid, RevocationStatus::Revoked],
                updates.into_iter().map(|(_, status)| status).collect_vec()
            );
            Ok(())
        });

        let storage = Arc::new(RwLock::new(storage));
        let context = RevocationTaskContext {
            config_repo: Arc::clone(&config_repo),
            status_list_client,
            storage,
            attestations_callback: Arc::default(),
            direct_notifications_callback: Arc::new(Mutex::new(Some(Arc::new(|_| Box::pin(async {}))))),
            scheduled_notifications_callback: Arc::default(),
        };

        TestWalletMockStorage::<TestConfigRepo>::check_revocations(
            &context,
            config_repo.get(),
            MockTimeGenerator::new(Utc::now()),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn should_send_direct_notification_only_for_revoked_attestations() {
        let (config_repo, status_list_client, mut storage, _, _) = setup_revocation_test_env().await;
        storage.expect_update_revocation_statuses().returning(|_| Ok(()));
        let storage = Arc::new(RwLock::new(storage));

        let notification_count: Arc<Mutex<Option<usize>>> = Arc::default();
        let count_clone = Arc::clone(&notification_count);

        let notifications_callback: DirectNotificationsCallback = Arc::new(move |notifications| {
            let count = Arc::clone(&count_clone);
            Box::pin(async move { *count.lock() = Some(notifications.len()) })
        });

        let context = RevocationTaskContext {
            config_repo: Arc::clone(&config_repo),
            status_list_client,
            storage,
            attestations_callback: Arc::default(),
            direct_notifications_callback: Arc::new(Mutex::new(Some(notifications_callback))),
            scheduled_notifications_callback: Arc::default(),
        };

        TestWalletMockStorage::<TestConfigRepo>::check_revocations(
            &context,
            config_repo.get(),
            MockTimeGenerator::new(Utc::now()),
        )
        .await
        .unwrap();

        assert_eq!(
            Some(1),
            *notification_count.lock(),
            "expected exactly 1 notification for the revoked attestation, non-revoked should not trigger"
        );
    }

    #[tokio::test]
    async fn should_check_revocation_periodically() {
        time::pause();

        let counter = Arc::new(AtomicU64::new(0));
        let counter_clone = Arc::clone(&counter);

        let abort_handle = spawn_periodic_task(Duration::from_millis(100), move || {
            let counter = Arc::clone(&counter_clone);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        assert_eq!(0, counter.load(Ordering::SeqCst));

        for _ in 0..10 {
            time::advance(Duration::from_millis(10)).await;
        }
        assert!(counter.load(Ordering::SeqCst) >= 1);

        for _ in 0..20 {
            time::advance(Duration::from_millis(10)).await;
        }
        assert!(counter.load(Ordering::SeqCst) >= 3, "Expected at least 3 checks");

        abort_handle.abort();
    }

    #[tokio::test]
    async fn should_stop_checking_after_abort() {
        time::pause();

        let counter = Arc::new(AtomicU64::new(0));
        let counter_clone = Arc::clone(&counter);

        let abort_handle = spawn_periodic_task(Duration::from_millis(100), move || {
            let counter = Arc::clone(&counter_clone);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        for _ in 0..30 {
            time::advance(Duration::from_millis(10)).await;
        }
        let count_before_abort = counter.load(Ordering::SeqCst);
        assert!(count_before_abort > 0);

        abort_handle.abort();

        for _ in 0..30 {
            time::advance(Duration::from_millis(10)).await;
        }
        assert_eq!(
            count_before_abort,
            counter.load(Ordering::SeqCst),
            "Count should not change after abort"
        );
    }

    #[tokio::test]
    async fn should_fire_on_first_tick_immediately() {
        time::pause();

        let counter = Arc::new(AtomicU64::new(0));
        let counter_clone = Arc::clone(&counter);

        let abort_handle = spawn_periodic_task(Duration::from_millis(100), move || {
            let counter = Arc::clone(&counter_clone);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        // Advance 5 times by 1ms (5ms total, well under the 100ms interval). tokio's
        // Interval fires its first tick immediately at t=0, so the first advance gives
        // the spawned task a chance to run and increment the counter exactly once.
        for _ in 0..5 {
            time::advance(Duration::from_millis(1)).await;
        }
        assert_eq!(1, counter.load(Ordering::SeqCst));

        abort_handle.abort();
    }

    #[tokio::test]
    async fn should_not_fire_between_ticks() {
        time::pause();

        let counter = Arc::new(AtomicU64::new(0));
        let counter_clone = Arc::clone(&counter);

        let abort_handle = spawn_periodic_task(Duration::from_millis(100), move || {
            let counter = Arc::clone(&counter_clone);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        // Advance to 150ms in small steps: first tick (t=0) and second tick (t=100ms)
        // should both have fired, giving count >= 2.
        for _ in 0..15 {
            time::advance(Duration::from_millis(10)).await;
        }
        let count = counter.load(Ordering::SeqCst);
        assert!(count >= 2);

        // Advance 40ms more (total 190ms, still before the third tick at t=200ms).
        // Count must not increase.
        for _ in 0..4 {
            time::advance(Duration::from_millis(10)).await;
        }
        assert_eq!(
            count,
            counter.load(Ordering::SeqCst),
            "task must not fire between ticks"
        );

        abort_handle.abort();
    }
}
