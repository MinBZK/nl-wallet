use std::{path::PathBuf, sync::Arc, time::Duration};

use async_trait::async_trait;
use tokio::{
    sync::watch::{channel, Receiver, Sender},
    task::JoinHandle,
    time,
};

use wallet_common::config::wallet_config::WalletConfiguration;

use super::{
    ConfigServerConfiguration, ConfigurationError, ConfigurationRepository, FileStorageConfigurationRepository,
    ObservableConfigurationRepository, UpdateableConfigurationRepository, UpdatingFileHttpConfigurationRepository,
};

pub struct UpdatingConfigurationRepository<T> {
    wrapped: Arc<T>,
    updating_task: JoinHandle<()>,
    callback_sender: Sender<CallbackFunction>,
}

pub type CallbackFunction = Box<dyn Fn(Arc<WalletConfiguration>) + Send + Sync>;

impl UpdatingFileHttpConfigurationRepository {
    pub async fn init(
        storage_path: PathBuf,
        config: ConfigServerConfiguration,
        initial_config: WalletConfiguration,
    ) -> Result<Self, ConfigurationError> {
        let wrapped = FileStorageConfigurationRepository::init(storage_path, config.base_url, initial_config).await?;
        let config = Self::new(wrapped, config.update_frequency).await;
        Ok(config)
    }
}

impl<T> UpdatingConfigurationRepository<T>
where
    T: UpdateableConfigurationRepository + Send + Sync + 'static,
{
    pub async fn new(wrapped: T, update_frequency: Duration) -> UpdatingConfigurationRepository<T> {
        let (tx, rx) = channel::<CallbackFunction>(Box::new(|_| {}));

        let wrapped = Arc::new(wrapped);
        let updating_task = Self::start_update_task(Arc::clone(&wrapped), rx, update_frequency).await;
        Self {
            wrapped,
            updating_task,
            callback_sender: tx,
        }
    }

    // This function is marked as async to force using a Tokio runtime and to prevent runtime panics of used without.
    async fn start_update_task(wrapped: Arc<T>, rx: Receiver<CallbackFunction>, interval: Duration) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = time::interval(interval);
            loop {
                interval.tick().await;
                let _ = wrapped.fetch().await;
                // todo: only call callback if config has actually changed
                let config = wrapped.config();
                let callback = rx.borrow();
                callback(config);
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

#[async_trait]
impl<T> ObservableConfigurationRepository for UpdatingConfigurationRepository<T>
where
    T: ConfigurationRepository,
{
    fn register_callback_on_update<F>(&self, callback: F)
    where
        F: Fn(Arc<WalletConfiguration>) + Send + Sync + 'static,
    {
        let _ = self.callback_sender.send_replace(Box::new(callback));
    }

    fn clear_callback(&self) {
        let _ = self.callback_sender.send_replace(Box::new(|_| {}));
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
            Arc, RwLock,
        },
        time::Duration,
    };

    use async_trait::async_trait;
    use tokio::{sync::Notify, time};

    use wallet_common::config::wallet_config::WalletConfiguration;

    use crate::config::{
        default_configuration, ConfigurationError, ConfigurationRepository, ObservableConfigurationRepository,
        UpdateableConfigurationRepository, UpdatingConfigurationRepository,
    };

    struct TestConfigRepo(RwLock<WalletConfiguration>);

    impl ConfigurationRepository for TestConfigRepo {
        fn config(&self) -> Arc<WalletConfiguration> {
            Arc::new(self.0.read().unwrap().clone())
        }
    }

    #[async_trait]
    impl UpdateableConfigurationRepository for TestConfigRepo {
        async fn fetch(&self) -> Result<(), ConfigurationError> {
            let mut config = self.0.write().unwrap();
            config.lock_timeouts.background_timeout = 900;
            Ok(())
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
        config.register_callback_on_update(move |config| {
            assert_eq!(900, config.lock_timeouts.background_timeout);
            let prev = callback_counter.fetch_add(1, Ordering::SeqCst);
            // when the previous value is 2 (= bigger than 1), the current value is 3 and the notifier is notified.
            if prev > 1 {
                callback_notifier.notify_one();
            }
        });

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
        let counter = Arc::new(AtomicU64::new(1));
        let callback_counter = Arc::clone(&counter);

        {
            let config = UpdatingConfigurationRepository::new(
                TestConfigRepo(RwLock::new(initial_wallet_config)),
                update_frequency,
            )
            .await;

            config.register_callback_on_update(move |_| {
                callback_counter.fetch_add(1, Ordering::SeqCst);
            });

            for _ in 0..10 {
                time::advance(Duration::from_millis(101)).await;
            }

            counted += counter.load(Ordering::SeqCst);
        }
        assert_eq!(10, counted);

        for _ in 0..10 {
            time::advance(Duration::from_millis(101)).await;
        }
        assert_eq!(counted, counter.load(Ordering::SeqCst), "after config is dropped, the update loop should have been aborted and the count should not have been updated");
    }
}
