use std::{path::PathBuf, sync::Arc, time::Duration};

use parking_lot::Mutex;
use tokio::{
    task::JoinHandle,
    time::{self, MissedTickBehavior},
};
use tracing::{error, info};

use wallet_common::config::wallet_config::WalletConfiguration;

use super::{
    ConfigCallback, ConfigServerConfiguration, ConfigurationError, ConfigurationRepository, ConfigurationUpdateState,
    FileStorageConfigurationRepository, ObservableConfigurationRepository, UpdateableConfigurationRepository,
    UpdatingFileHttpConfigurationRepository,
};

pub struct UpdatingConfigurationRepository<T> {
    wrapped: Arc<T>,
    callback: Arc<Mutex<Option<ConfigCallback>>>,
    updating_task: JoinHandle<()>,
}

impl UpdatingFileHttpConfigurationRepository {
    pub async fn init(
        storage_path: PathBuf,
        config: ConfigServerConfiguration,
        initial_config: WalletConfiguration,
    ) -> Result<Self, ConfigurationError> {
        let wrapped = FileStorageConfigurationRepository::init(
            storage_path,
            config.base_url,
            config.trust_anchors,
            (&config.signing_public_key).into(),
            initial_config,
        )
        .await?;
        let config = Self::new(wrapped, config.update_frequency).await;
        Ok(config)
    }
}

impl<T> UpdatingConfigurationRepository<T>
where
    T: UpdateableConfigurationRepository + Send + Sync + 'static,
{
    pub async fn new(wrapped: T, update_frequency: Duration) -> UpdatingConfigurationRepository<T> {
        let wrapped = Arc::new(wrapped);
        let callback = Arc::new(Mutex::new(None));
        let updating_task =
            Self::start_update_task(Arc::clone(&wrapped), Arc::clone(&callback), update_frequency).await;

        Self {
            wrapped,
            updating_task,
            callback,
        }
    }

    // This function is marked as async to force using a Tokio runtime and to prevent runtime panics if used without.
    async fn start_update_task(
        wrapped: Arc<T>,
        callback: Arc<Mutex<Option<ConfigCallback>>>,
        interval: Duration,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = time::interval(interval);
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

            loop {
                interval.tick().await;

                info!("Wallet configuration update timer expired, fetching from remote...");

                match wrapped.fetch().await {
                    Ok(state) => {
                        if let ConfigurationUpdateState::Updated = state {
                            let config = wrapped.config();

                            if let Some(callback) = callback.lock().as_deref_mut() {
                                callback(config)
                            }
                        }
                    }
                    Err(e) => error!("fetch configuration error: {}", e),
                }
            }
        })
    }
}

impl<T> ConfigurationRepository for UpdatingConfigurationRepository<T>
where
    T: ConfigurationRepository,
{
    fn config(&self) -> Arc<WalletConfiguration> {
        self.wrapped.config()
    }
}

impl<T> ObservableConfigurationRepository for UpdatingConfigurationRepository<T>
where
    T: ConfigurationRepository,
{
    fn register_callback_on_update(&self, callback: ConfigCallback) -> Option<ConfigCallback> {
        self.callback.lock().replace(callback)
    }

    fn clear_callback(&self) -> Option<ConfigCallback> {
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
    use std::{
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc,
        },
        time::Duration,
    };

    use parking_lot::RwLock;
    use tokio::{sync::Notify, time};

    use wallet_common::config::wallet_config::WalletConfiguration;

    use crate::config::{
        default_configuration, ConfigurationError, ConfigurationRepository, ConfigurationUpdateState,
        ObservableConfigurationRepository, UpdateableConfigurationRepository, UpdatingConfigurationRepository,
    };

    struct TestConfigRepo(RwLock<WalletConfiguration>);

    impl ConfigurationRepository for TestConfigRepo {
        fn config(&self) -> Arc<WalletConfiguration> {
            Arc::new(self.0.read().clone())
        }
    }

    impl UpdateableConfigurationRepository for TestConfigRepo {
        async fn fetch(&self) -> Result<ConfigurationUpdateState, ConfigurationError> {
            let mut config = self.0.write();
            config.lock_timeouts.background_timeout = 900;
            Ok(ConfigurationUpdateState::Updated)
        }
    }

    #[tokio::test]
    async fn should_update_config() {
        let initial_wallet_config = default_configuration();

        // pause time so we can advance it later
        time::pause();
        let update_frequency = Duration::from_millis(1000);

        let config =
            UpdatingConfigurationRepository::new(TestConfigRepo(RwLock::new(initial_wallet_config)), update_frequency)
                .await;

        assert_eq!(300, config.config().lock_timeouts.background_timeout);

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

        assert_eq!(900, config.config().lock_timeouts.background_timeout);
        assert_eq!(3, counter.load(Ordering::SeqCst));

        time::advance(Duration::from_millis(3000)).await;
        assert_eq!(3, counter.load(Ordering::SeqCst), "should not update after clear");
    }

    #[tokio::test]
    async fn drop_should_abort_updating_task() {
        let initial_wallet_config = default_configuration();

        // pause time so we can advance it later
        time::pause();
        let update_frequency = Duration::from_millis(100);

        let mut counted = 0;
        let counter = Arc::new(AtomicU64::new(0));
        let callback_counter = Arc::clone(&counter);

        {
            let config = UpdatingConfigurationRepository::new(
                TestConfigRepo(RwLock::new(initial_wallet_config)),
                update_frequency,
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
