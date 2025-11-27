use std::sync::Arc;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use futures::StreamExt;
use parking_lot::Mutex;
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
use openid4vc::issuance_session::IssuanceSession;
use platform_support::attested_key::AttestedKeyHolder;
use token_status_list::verification::client::StatusListClient;
use token_status_list::verification::verifier::RevocationStatus;
use token_status_list::verification::verifier::RevocationVerifier;
use update_policy_model::update_policy::VersionState;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::Wallet;
use crate::account_provider::AccountProviderClient;
use crate::digid::DigidClient;
use crate::errors::StorageError;
use crate::repository::Repository;
use crate::storage::RevocationInfo;
use crate::storage::Storage;
use crate::wallet::attestations::AttestationsCallback;
use crate::wallet::attestations::AttestationsError;

const CHECK_FREQUENCY_IN_SECONDS: u64 = 60 * 60 * 24;

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
    UR: Repository<VersionState>,
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    IS: IssuanceSession,
    DCC: DisclosureClient,
    APC: AccountProviderClient,
    SLC: StatusListClient,
{
    /// Start background revocation checks
    ///
    /// Spawns a background task that only accesses storage, not the entire wallet.
    /// Stores the ablort handle in the wallet.
    #[instrument(skip_all)]
    pub fn start_background_revocation_checks(&mut self)
    where
        S: Send + Sync + 'static,
        SLC: Send + Sync + 'static,
    {
        if self.revocation_status_job_handle.is_some() {
            info!("background revocation checks already running");
            return;
        }

        // Clone only what is needed for the background task
        let config = Arc::clone(&self.config_repository.get());
        let status_list_client = Arc::clone(&self.status_list_client);
        let storage = Arc::clone(&self.storage);
        let callback = Arc::clone(&self.attestations_callback);

        let abort_handle = Self::spawn_revocation_checks(
            Arc::clone(&config),
            Arc::clone(&status_list_client),
            Arc::clone(&storage),
            Arc::clone(&callback),
            TimeGenerator,
            Duration::from_secs(CHECK_FREQUENCY_IN_SECONDS),
        );

        // Store the abort handle
        self.revocation_status_job_handle = Some(abort_handle);
    }

    fn spawn_revocation_checks<T>(
        config: Arc<WalletConfiguration>,
        status_list_client: Arc<SLC>,
        storage: Arc<RwLock<S>>,
        callback: Arc<Mutex<Option<AttestationsCallback>>>,
        time_generator: T,
        check_interval: Duration,
    ) -> AbortHandle
    where
        S: Send + Sync + 'static,
        SLC: Send + Sync + 'static,
        T: Generator<DateTime<Utc>> + Send + Sync + 'static,
    {
        let task = tokio::spawn(async move {
            let mut interval = time::interval(check_interval);
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

            loop {
                interval.tick().await;

                if let Err(e) = Self::check_revocations(
                    Arc::clone(&config),
                    Arc::clone(&status_list_client),
                    Arc::clone(&storage),
                    Arc::clone(&callback),
                    &time_generator,
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
    async fn check_revocations(
        config: Arc<WalletConfiguration>,
        status_list_client: Arc<SLC>,
        storage: Arc<RwLock<S>>,
        callback: Arc<Mutex<Option<AttestationsCallback>>>,
        time_generator: &impl Generator<DateTime<Utc>>,
    ) -> Result<(), RevocationError>
    where
        SLC: StatusListClient,
        S: Storage,
    {
        let revocation_verifier = RevocationVerifier::new(Arc::clone(&status_list_client));

        // Fetch revocation info in one storage lock
        let revocation_info = storage.read().await.fetch_all_revocation_info().await?;

        let issuer_trust_anchors = config.issuer_trust_anchors();

        // Verify all revocations without holding any locks
        let updates: Vec<(Uuid, RevocationStatus)> = futures::stream::iter(revocation_info)
            .map(|revocation_info| {
                Self::check_revocation(
                    revocation_info,
                    &revocation_verifier,
                    &issuer_trust_anchors,
                    time_generator,
                )
            })
            .buffer_unordered(10)
            .collect()
            .await;

        // Write all updates in one storage lock
        storage.write().await.update_revocation_statuses(updates).await?;

        // Callback with the updated attestations
        if callback.lock().is_some() {
            let attestations = storage.read().await.fetch_unique_attestations().await?;

            if let Some(ref mut callback) = callback.lock().as_deref_mut() {
                callback(
                    attestations
                        .into_iter()
                        .map(|copy| copy.into_attestation_presentation(&config.pid_attributes))
                        .collect(),
                )
            }
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

    use attestation_types::status_claim::StatusClaim;
    use crypto::server_keys::generate::Ca;
    use token_status_list::status_list_token::mock::create_status_list_token;
    use token_status_list::verification::client::mock::MockStatusListClient;
    use token_status_list::verification::verifier::RevocationStatus;
    use utils::generator::mock::MockTimeGenerator;

    use crate::config::default_wallet_config;
    use crate::storage::MockStorage;
    use crate::wallet::test::TestWalletMockStorage;

    use super::*;

    #[tokio::test]
    async fn should_check_revocation_periodically() {
        // Pause time so we can advance it manually
        time::pause();

        let config = Arc::new(default_wallet_config());
        let status_list_client = Arc::new(MockStatusListClient::new());

        let mut storage = MockStorage::new();
        storage.expect_fetch_all_revocation_info().returning(|| Ok(vec![]));
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

        // Register callback to track updates
        callback.lock().replace(callback_fn);

        let abort_handle = TestWalletMockStorage::spawn_revocation_checks(
            config,
            status_list_client,
            storage,
            callback,
            time_generator,
            check_interval,
        );

        // Initially no checks should have occurred
        assert_eq!(0, counter.load(Ordering::SeqCst));

        // Advance time 10 times by 10 ms - should trigger first check
        for _ in 0..10 {
            time::advance(Duration::from_millis(10)).await;
        }

        // Should have performed at least one check
        assert!(counter.load(Ordering::SeqCst) >= 1);

        // Advance time 20 times by 10 ms - should trigger 2 more checks
        for _ in 0..20 {
            time::advance(Duration::from_millis(10)).await;
        }

        let final_count = counter.load(Ordering::SeqCst);
        assert!(final_count >= 3, "Expected at least 3 checks, got {}", final_count);

        abort_handle.abort();
    }

    #[tokio::test]
    async fn should_stop_checking_after_abort() {
        // Pause time so we can advance it manually
        time::pause();

        let config = Arc::new(default_wallet_config());
        let status_list_client = Arc::new(MockStatusListClient::new());

        let mut storage = MockStorage::new();
        storage.expect_fetch_all_revocation_info().returning(|| Ok(vec![]));
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

        // Register callback to track updates
        callback.lock().replace(callback_fn);

        let abort_handle = TestWalletMockStorage::spawn_revocation_checks(
            config,
            status_list_client,
            storage,
            callback,
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
        wallet_config.issuer_trust_anchors = vec![ca.as_borrowing_trust_anchor().clone()];
        let config = Arc::new(wallet_config);

        let (_, _, status_list_token) = create_status_list_token(&keypair, None, None).await;
        let mut mock_status_list_client = MockStatusListClient::new();
        mock_status_list_client
            .expect_fetch()
            .returning(move |_| Ok(status_list_token.clone()));
        let status_list_client = Arc::new(mock_status_list_client);

        let update_count = Arc::new(AtomicU64::new(0));
        let update_counter = Arc::clone(&update_count);

        let mut storage = MockStorage::new();
        let test_revocation_info = vec![
            RevocationInfo::new(
                Uuid::new_v4(),
                StatusClaim::new_mock(),
                keypair.certificate().distinguished_name_canonical().unwrap(),
            ),
            RevocationInfo::new(
                Uuid::new_v4(),
                StatusClaim::new_mock(),
                keypair.certificate().distinguished_name_canonical().unwrap(),
            ),
        ];

        storage
            .expect_fetch_all_revocation_info()
            .returning(move || Ok(test_revocation_info.clone()));
        storage.expect_update_revocation_statuses().returning(move |updates| {
            update_counter.fetch_add(1, Ordering::SeqCst);
            assert_eq!(
                vec![RevocationStatus::Valid, RevocationStatus::Valid],
                updates.into_iter().map(|(_, status)| status).collect_vec()
            );
            Ok(())
        });
        storage.expect_fetch_unique_attestations().returning(|| Ok(vec![]));
        let storage = Arc::new(RwLock::new(storage));

        let time_generator = MockTimeGenerator::new(Utc::now());
        let check_interval = Duration::from_millis(100);
        let callback = Arc::new(Mutex::new(None));

        let abort_handle = TestWalletMockStorage::spawn_revocation_checks(
            config,
            status_list_client,
            storage,
            callback,
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

        abort_handle.abort();
    }
}
