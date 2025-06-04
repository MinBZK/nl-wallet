use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use tokio::task::JoinHandle;
use tokio::time;
use tokio::time::MissedTickBehavior;
use tracing::error;
use tracing::info;

use http_utils::tls::pinning::TlsPinningConfig;
use wallet_configuration::config_server_config::ConfigServerConfiguration;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::repository::ObservableRepository;
use crate::repository::Repository;
use crate::repository::RepositoryCallback;
use crate::repository::RepositoryUpdateState;
use crate::repository::UpdateableRepository;

use super::ConfigurationError;
use super::FileStorageConfigurationRepository;
use super::WalletConfigurationRepository;

pub struct UpdatingConfigurationRepository<T> {
    wrapped: Arc<T>,
    callback: Arc<Mutex<Option<RepositoryCallback<Arc<WalletConfiguration>>>>>,
    updating_task: JoinHandle<()>,
}

impl WalletConfigurationRepository {
    pub async fn init(
        storage_path: PathBuf,
        config: ConfigServerConfiguration,
        initial_config: WalletConfiguration,
    ) -> Result<Self, ConfigurationError> {
        let wrapped = FileStorageConfigurationRepository::init(
            storage_path,
            config.signing_public_key.as_inner().into(),
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
        let updating_task = Self::start_update_task(Arc::clone(&wrapped), Arc::clone(&callback), config).await;

        Self {
            wrapped,
            callback,
            updating_task,
        }
    }

    // This function is marked as async to force using a Tokio runtime and to prevent runtime panics if used without.
    #[allow(clippy::unused_async)]
    async fn start_update_task(
        wrapped: Arc<T>,
        callback: Arc<Mutex<Option<RepositoryCallback<Arc<WalletConfiguration>>>>>,
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
    use std::sync::atomic::AtomicU64;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::time::Duration;

    use parking_lot::RwLock;
    use tokio::sync::Notify;
    use tokio::time;

    use wallet_configuration::wallet_config::WalletConfiguration;

    use crate::config::default_config_server_config;
    use crate::config::default_wallet_config;
    use crate::config::ConfigurationError;
    use crate::config::UpdatingConfigurationRepository;
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
}
