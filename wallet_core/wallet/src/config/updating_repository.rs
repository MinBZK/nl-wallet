use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use chrono::Utc;
use crypto::PublicKey;
use http_utils::client::TlsPinningConfig;
use parking_lot::Mutex;
use tokio::task::JoinHandle;
use tokio::time;
use tokio::time::MissedTickBehavior;
use tracing::error;
use tracing::info;
use wallet_configuration::config_server_config::ConfigServerConfiguration;
use wallet_configuration::wallet_config::WalletConfiguration;

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
    /// Re-evaluates whether the given configuration is expired and reports the outcome. Must only be called once a
    /// fetch attempt has resolved.
    fn update(&self, config: &WalletConfiguration) {
        let expired = is_expired(config, Utc::now());
        self.expired.store(expired, Ordering::Relaxed);

        if let Some(callback) = self.callback.lock().as_deref_mut() {
            callback(expired);
        }
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
            let mut interval = time::interval(config.update_frequency);
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

            loop {
                interval.tick().await;

                info!("Wallet configuration update timer expired, fetching from remote...");

                match wrapped.fetch(&config.http_config).await {
                    Ok(state) => {
                        if let RepositoryUpdateState::Updated { .. } = state {
                            let config = wrapped.get();

                            if let Some(callback) = callback.lock().as_deref_mut() {
                                callback(config);
                            }
                        }
                    }
                    Err(e) => error!("fetch configuration error: {}", e),
                }

                // Re-evaluate after every attempt, whatever its outcome: only a valid configuration can clear the
                // expiry.
                expiry.update(&wrapped.get());
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
    use tokio::sync::Notify;
    use tokio::time;
    use wallet_configuration::wallet_config::WalletConfiguration;

    use super::ExpiryState;
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

            // Advance the clock so that the initial fetch plus 9 additional ones occur.
            for _ in 0..(9 * 101) {
                // The `time::advance()` function does not seem to work if we simply
                // advance the time by 100ms. This probably has something to do with
                // the tokio runtime running in `current_thread` mode.
                time::advance(Duration::from_millis(1)).await;
            }

            counted += counter.load(Ordering::SeqCst);
        }
        assert_eq!(10, counted);

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
    fn expiry_should_not_be_reported_before_the_first_update() {
        assert!(!ExpiryState::default().expired());
    }

    #[test]
    fn expiry_should_be_reported_after_every_update() {
        let state = ExpiryState::default();
        let reported = record_reported(&state);

        let expired_config = config_expiring_in(-TimeDelta::hours(1));
        for _ in 0..2 {
            state.update(&expired_config);
        }

        assert!(state.expired());

        // A subsequent fetch yielding a valid configuration should clear the expiry again.
        state.update(&config_expiring_in(TimeDelta::hours(1)));

        assert!(!state.expired());

        // Every evaluation is reported, including repeated ones. Leaving out identical values is up to the consumer.
        assert_eq!(*reported.lock(), vec![true, true, false]);
    }
}
