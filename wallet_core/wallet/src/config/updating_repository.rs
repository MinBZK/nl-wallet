use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use crypto::PublicKey;
use http_utils::client::TlsPinningConfig;
use parking_lot::Mutex;
use rand::Rng;
use tokio::task::JoinHandle;
use tokio::time;
use tracing::error;
use tracing::info;
use tracing::warn;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use wallet_configuration::config_server_config::ConfigServerConfiguration;
use wallet_configuration::wallet_config::WalletConfiguration;

use super::CONFIG_EXPIRY_LEEWAY;
use super::ConfigurationError;
use super::FileStorageConfigurationRepository;
use super::WalletConfigurationRepository;
use super::is_expired;
use crate::repository::ObservableRepository;
use crate::repository::Repository;
use crate::repository::RepositoryCallback;
use crate::repository::RepositoryUpdateState;
use crate::repository::UpdateableRepository;

pub struct UpdatingConfigurationRepository<T> {
    wrapped: Arc<T>,
    callback: Arc<Mutex<Option<RepositoryCallback<Arc<WalletConfiguration>>>>>,
    expiry: Arc<ExpiryState>,
    updating_task: JoinHandle<()>,
}

/// Upper bound for the delay between attempts to replace an expired configuration.
const EXPIRED_RETRY_MAX_INTERVAL: Duration = Duration::from_secs(60);

/// The fraction by which the delay between attempts is randomized. This prevents wallets having the same `exp` from
/// retrying all at the same time, when their config expires.
const EXPIRED_RETRY_JITTER: f64 = 0.2;

/// The delay before the next attempt to replace an expired configuration, growing linearly with the number of
/// consecutive attempts that failed to produce one, up to [`EXPIRED_RETRY_MAX_INTERVAL`].
fn expired_retry_delay(retry_interval: Duration, failed_attempts: u32) -> Duration {
    let delay = retry_interval
        .saturating_mul(failed_attempts.max(1))
        .min(EXPIRED_RETRY_MAX_INTERVAL);

    let jitter = rand::thread_rng().gen_range(-EXPIRED_RETRY_JITTER..=EXPIRED_RETRY_JITTER);

    delay.mul_f64(1.0 + jitter)
}

/// The delay before the next attempt while the wallet holds a valid configuration, which is whichever comes first:
/// the regular update frequency, or the moment the configuration expires. The latter makes sure the wallet notices
/// its own expiry as it happens, instead of at the next regular update.
fn valid_config_delay(config: &WalletConfiguration, update_frequency: Duration, now: DateTime<Utc>) -> Duration {
    let expires: DateTime<Utc> = config.expires.into();
    let until_expired = (expires + CONFIG_EXPIRY_LEEWAY - now)
        .to_std()
        .unwrap_or(Duration::ZERO);

    // Fetch somewhat before the configuration actually expires, so that wallets sharing an `exp` do not all fetch at
    // the very same instant. Note that this only ever moves the attempt earlier: doing so is free, as it simply
    // replaces a configuration that is still valid, whereas moving it later would leave a window in which the
    // configuration is expired without any attempt having been made to replace it.
    let jitter = rand::thread_rng().gen_range(1.0 - EXPIRED_RETRY_JITTER..=1.0);

    update_frequency.min(until_expired.mul_f64(jitter))
}

/// The outcome of evaluating the `exp` of a wallet configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigurationStatus {
    Valid,
    Expired,
}

impl ConfigurationStatus {
    fn is_expired(self) -> bool {
        matches!(self, Self::Expired)
    }
}

/// Tracks whether the wallet configuration should be reported as expired to the user interface.
///
/// This is only advanced after a fetch attempt, so that the user is blocked when the wallet has no valid
/// configuration and fails to retrieve a fresh one, instead of on every cold start that begins with an expired one.
#[derive(Default)]
struct ExpiryState {
    callback: Mutex<Option<RepositoryCallback<bool>>>,
    /// The most recently evaluated value, used to seed a callback that is registered later.
    expired: AtomicBool,
}

impl ExpiryState {
    /// Re-evaluates the status of the given configuration at `now`, reports it and returns it. Must only be called
    /// once a fetch attempt has resolved.
    fn evaluate(&self, config: &WalletConfiguration, now: DateTime<Utc>) -> ConfigurationStatus {
        let status = if is_expired(config, now) {
            ConfigurationStatus::Expired
        } else {
            ConfigurationStatus::Valid
        };

        self.expired.store(status.is_expired(), Ordering::Relaxed);

        if let Some(callback) = self.callback.lock().as_deref_mut() {
            callback(status.is_expired());
        }

        status
    }

    fn expired(&self) -> bool {
        self.expired.load(Ordering::Relaxed)
    }
}

/// Observes whether the wallet configuration should be considered expired by the user interface.
pub trait ObservableConfigExpiry {
    fn config_expired(&self) -> bool;

    fn register_config_expiry_callback(&self, callback: RepositoryCallback<bool>) -> Option<RepositoryCallback<bool>>;

    fn clear_config_expiry_callback(&self) -> Option<RepositoryCallback<bool>>;
}

impl<T> ObservableConfigExpiry for UpdatingConfigurationRepository<T> {
    fn config_expired(&self) -> bool {
        self.expiry.expired()
    }

    fn register_config_expiry_callback(&self, callback: RepositoryCallback<bool>) -> Option<RepositoryCallback<bool>> {
        self.expiry.callback.lock().replace(callback)
    }

    fn clear_config_expiry_callback(&self) -> Option<RepositoryCallback<bool>> {
        self.expiry.callback.lock().take()
    }
}

impl WalletConfigurationRepository {
    pub async fn init(
        storage_path: PathBuf,
        config: ConfigServerConfiguration,
        initial_config: WalletConfiguration,
    ) -> Result<Self, ConfigurationError> {
        let wrapped = FileStorageConfigurationRepository::init(
            storage_path,
            PublicKey::from(*config.signing_public_key.as_inner()).into(),
            initial_config,
        )
        .await?;
        let updating_repository = Self::new(wrapped, config).await;
        Ok(updating_repository)
    }
}

impl<T> UpdatingConfigurationRepository<T>
where
    T: UpdateableRepository<Arc<WalletConfiguration>, TlsPinningConfig> + Send + Sync + 'static,
{
    pub async fn new(wrapped: T, config: ConfigServerConfiguration) -> UpdatingConfigurationRepository<T> {
        let wrapped = Arc::new(wrapped);
        let callback = Arc::new(Mutex::new(None));
        let expiry = Arc::new(ExpiryState::default());
        let updating_task =
            Self::start_update_task(Arc::clone(&wrapped), Arc::clone(&callback), Arc::clone(&expiry), config).await;

        Self {
            wrapped,
            callback,
            expiry,
            updating_task,
        }
    }

    // This function is marked as async to force using a Tokio runtime and to prevent runtime panics if used without.
    #[expect(clippy::unused_async)]
    async fn start_update_task(
        wrapped: Arc<T>,
        callback: Arc<Mutex<Option<RepositoryCallback<Arc<WalletConfiguration>>>>>,
        expiry: Arc<ExpiryState>,
        config: ConfigServerConfiguration,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            // The number of consecutive attempts that have left the wallet without a valid configuration.
            let mut failed_attempts: u32 = 0;

            loop {
                info!("Fetching wallet configuration from remote...");

                match wrapped.fetch(&config.http_config).await {
                    Ok(RepositoryUpdateState::Updated { to, .. }) => {
                        if let Some(callback) = callback.lock().as_deref_mut() {
                            callback(to);
                        }
                    }
                    // The configuration did not change, so there is nothing to report.
                    Ok(RepositoryUpdateState::Unmodified(_) | RepositoryUpdateState::Cached(_)) => {}
                    Err(e) => error!("fetch configuration error: {}", e),
                }

                // Read the clock once, so that the expiry and the delay derived from it cannot disagree.
                let now = TimeGenerator.generate();

                // Re-evaluate after every attempt, whatever its outcome: only a valid configuration can clear the
                // expiry.
                let delay = match expiry.evaluate(&wrapped.get(), now) {
                    ConfigurationStatus::Expired => {
                        failed_attempts = failed_attempts.saturating_add(1);

                        let delay = expired_retry_delay(config.expired_retry_interval, failed_attempts);
                        warn!(
                            "Wallet configuration is expired, retrying in {:.1}s",
                            delay.as_secs_f64()
                        );

                        delay
                    }
                    ConfigurationStatus::Valid => {
                        failed_attempts = 0;

                        valid_config_delay(&wrapped.get(), config.update_frequency, now)
                    }
                };

                time::sleep(delay).await;
            }
        })
    }
}

impl<T> Repository<Arc<WalletConfiguration>> for UpdatingConfigurationRepository<T>
where
    T: Repository<Arc<WalletConfiguration>>,
{
    fn get(&self) -> Arc<WalletConfiguration> {
        self.wrapped.get()
    }
}

impl<T> ObservableRepository<Arc<WalletConfiguration>> for UpdatingConfigurationRepository<T>
where
    T: Repository<Arc<WalletConfiguration>>,
{
    fn register_callback_on_update(
        &self,
        callback: RepositoryCallback<Arc<WalletConfiguration>>,
    ) -> Option<RepositoryCallback<Arc<WalletConfiguration>>> {
        self.callback.lock().replace(callback)
    }

    fn clear_callback(&self) -> Option<RepositoryCallback<Arc<WalletConfiguration>>> {
        self.callback.lock().take()
    }
}

impl<T> Drop for UpdatingConfigurationRepository<T> {
    fn drop(&mut self) {
        self.updating_task.abort();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use chrono::TimeDelta;
    use chrono::Utc;
    use parking_lot::Mutex;
    use parking_lot::RwLock;
    use rstest::rstest;
    use tokio::sync::Notify;
    use tokio::time;
    use wallet_configuration::wallet_config::WalletConfiguration;

    use super::CONFIG_EXPIRY_LEEWAY;
    use super::ConfigurationStatus;
    use super::EXPIRED_RETRY_JITTER;
    use super::EXPIRED_RETRY_MAX_INTERVAL;
    use super::ExpiryState;
    use super::expired_retry_delay;
    use super::valid_config_delay;
    use crate::config::ConfigurationError;
    use crate::config::UpdatingConfigurationRepository;
    use crate::config::default_config_server_config;
    use crate::config::default_wallet_config;
    use crate::repository::ObservableRepository;
    use crate::repository::Repository;
    use crate::repository::RepositoryUpdateState;
    use crate::repository::UpdateableRepository;

    struct TestConfigRepo(RwLock<WalletConfiguration>);

    impl Repository<Arc<WalletConfiguration>> for TestConfigRepo {
        fn get(&self) -> Arc<WalletConfiguration> {
            Arc::new(self.0.read().clone())
        }
    }

    impl<B> UpdateableRepository<Arc<WalletConfiguration>, B> for TestConfigRepo
    where
        B: Send + Sync,
    {
        type Error = ConfigurationError;

        async fn fetch(&self, _: &B) -> Result<RepositoryUpdateState<Arc<WalletConfiguration>>, ConfigurationError> {
            let mut config = self.0.write();
            let from = config.clone();
            config.lock_timeouts.background_timeout = 900;
            Ok(RepositoryUpdateState::Updated {
                from: Arc::new(from),
                to: Arc::new(config.clone()),
            })
        }
    }

    #[tokio::test]
    async fn should_update_config() {
        let mut config_server_config = default_config_server_config();
        let initial_wallet_config = default_wallet_config();

        // pause time so we can advance it later
        time::pause();
        config_server_config.update_frequency = Duration::from_millis(1000);

        let config = UpdatingConfigurationRepository::new(
            TestConfigRepo(RwLock::new(initial_wallet_config)),
            config_server_config,
        )
        .await;

        assert_eq!(300, config.get().lock_timeouts.background_timeout);

        let notifier = Arc::new(Notify::new());
        let callback_notifier = notifier.clone();

        let counter = Arc::new(AtomicU64::new(0));
        let callback_counter = Arc::clone(&counter);
        config.register_callback_on_update(Box::new(move |config| {
            assert_eq!(900, config.lock_timeouts.background_timeout);
            let prev = callback_counter.fetch_add(1, Ordering::SeqCst);
            // when the previous value is 2 (= bigger than 1), the current value is 3 and the notifier is notified.
            if prev > 1 {
                callback_notifier.notify_one();
            }
        }));

        time::advance(Duration::from_millis(3000)).await;
        notifier.notified().await;

        config.clear_callback();

        assert_eq!(900, config.get().lock_timeouts.background_timeout);
        assert_eq!(3, counter.load(Ordering::SeqCst));

        time::advance(Duration::from_millis(3000)).await;
        assert_eq!(3, counter.load(Ordering::SeqCst), "should not update after clear");
    }

    #[tokio::test]
    async fn drop_should_abort_updating_task() {
        let mut config_server_config = default_config_server_config();
        let initial_wallet_config = default_wallet_config();

        // pause time so we can advance it later
        time::pause();
        config_server_config.update_frequency = Duration::from_millis(100);

        let mut counted = 0;
        let counter = Arc::new(AtomicU64::new(0));
        let callback_counter = Arc::clone(&counter);

        {
            let config = UpdatingConfigurationRepository::new(
                TestConfigRepo(RwLock::new(initial_wallet_config)),
                config_server_config,
            )
            .await;

            config.register_callback_on_update(Box::new(move |_| {
                callback_counter.fetch_add(1, Ordering::SeqCst);
            }));

            // Advance the clock so that the initial fetch plus 8 additional ones occur. Note that the delay is
            // applied after each fetch completes, so a full cycle takes slightly longer than the update frequency.
            for _ in 0..(9 * 101) {
                // The `time::advance()` function does not seem to work if we simply
                // advance the time by 100ms. This probably has something to do with
                // the tokio runtime running in `current_thread` mode.
                time::advance(Duration::from_millis(1)).await;
            }

            counted += counter.load(Ordering::SeqCst);
        }
        assert_eq!(9, counted);

        for _ in 0..(9 * 101) {
            time::advance(Duration::from_millis(1)).await;
        }
        assert_eq!(
            counted,
            counter.load(Ordering::SeqCst),
            "after config is dropped, the update loop should have been aborted and the count should not have been \
             updated"
        );
    }

    const RETRY_INTERVAL: Duration = Duration::from_secs(10);

    #[rstest]
    #[case(1, RETRY_INTERVAL)]
    #[case(2, 2 * RETRY_INTERVAL)]
    #[case(5, 5 * RETRY_INTERVAL)]
    // Beyond this the delay is capped.
    #[case(6, EXPIRED_RETRY_MAX_INTERVAL)]
    #[case(7, EXPIRED_RETRY_MAX_INTERVAL)]
    #[case(u32::MAX, EXPIRED_RETRY_MAX_INTERVAL)]
    fn retry_delay_should_grow_linearly_up_to_the_maximum(#[case] failed_attempts: u32, #[case] expected: Duration) {
        // Repeat, since the applied jitter is random.
        for _ in 0..100 {
            let delay = expired_retry_delay(RETRY_INTERVAL, failed_attempts);

            assert!(
                delay >= expected.mul_f64(1.0 - EXPIRED_RETRY_JITTER)
                    && delay <= expected.mul_f64(1.0 + EXPIRED_RETRY_JITTER),
                "delay of {delay:?} for {failed_attempts} failed attempts is not within jitter of {expected:?}"
            );
        }
    }

    #[test]
    fn valid_config_delay_should_not_outlast_the_configuration() {
        let update_frequency = Duration::from_secs(3600);

        // Repeat, since the applied jitter is random.
        for _ in 0..100 {
            // A configuration that outlives the update frequency should simply be fetched at the regular interval.
            let delay = valid_config_delay(&config_expiring_in(TimeDelta::hours(2)), update_frequency, Utc::now());
            assert_eq!(delay, update_frequency);

            // One that expires sooner should be fetched again before it does, so that its expiry does not go
            // unnoticed until the next regular update. Jitter should only ever make that happen earlier.
            let window = Duration::from_secs(5 * 60) + CONFIG_EXPIRY_LEEWAY;
            let delay = valid_config_delay(&config_expiring_in(TimeDelta::minutes(5)), update_frequency, Utc::now());

            // Since `exp` has seconds precision and time passes between constructing the configuration and
            // evaluating it, the actual window can be slightly shorter than the nominal one.
            let tolerance = Duration::from_secs(2);

            assert!(delay < update_frequency);
            assert!(
                delay <= window && delay >= window.mul_f64(1.0 - EXPIRED_RETRY_JITTER) - tolerance,
                "delay of {delay:?} should fall within the jitter applied to {window:?}"
            );
        }
    }

    fn config_expiring_in(expires_in: TimeDelta) -> WalletConfiguration {
        WalletConfiguration {
            expires: (Utc::now() + expires_in).into(),
            ..default_wallet_config()
        }
    }

    fn record_reported(state: &ExpiryState) -> Arc<Mutex<Vec<bool>>> {
        let reported = Arc::new(Mutex::new(Vec::new()));
        let callback_reported = Arc::clone(&reported);

        state
            .callback
            .lock()
            .replace(Box::new(move |expired| callback_reported.lock().push(expired)));

        reported
    }

    #[test]
    fn expiry_should_not_be_reported_before_the_first_evaluation() {
        assert!(!ExpiryState::default().expired());
    }

    #[test]
    fn expiry_should_be_reported_after_every_evaluation() {
        let state = ExpiryState::default();
        let reported = record_reported(&state);

        let expired_config = config_expiring_in(-TimeDelta::hours(1));
        for _ in 0..2 {
            assert_eq!(
                state.evaluate(&expired_config, Utc::now()),
                ConfigurationStatus::Expired
            );
        }

        assert!(state.expired());

        // A subsequent fetch yielding a valid configuration should clear the expiry again.
        assert_eq!(
            state.evaluate(&config_expiring_in(TimeDelta::hours(1)), Utc::now()),
            ConfigurationStatus::Valid
        );

        assert!(!state.expired());

        // Every evaluation is reported, including repeated ones. Leaving out identical values is up to the consumer.
        assert_eq!(*reported.lock(), vec![true, true, false]);
    }
}
