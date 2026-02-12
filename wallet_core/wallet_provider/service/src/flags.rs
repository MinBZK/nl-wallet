use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use tokio::sync::Mutex;
use tokio::task::AbortHandle;
use tokio::time::MissedTickBehavior;

use wallet_provider_domain::model::wallet_flag::WalletFlag;
use wallet_provider_domain::repository::PersistenceError;
use wallet_provider_domain::repository::WalletFlagRepository;

pub trait WalletFlags {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn set_solution_revoked(&self) -> Result<(), Self::Error>;

    fn solution_is_revoked(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct WalletRepoFlags<R> {
    repository: R,
    refresh_delay: Duration,
    refresh_mutex: Arc<Mutex<()>>,
    values: Arc<RwLock<HashMap<WalletFlag, bool>>>,
}

impl<R> WalletRepoFlags<R> {
    pub fn new(repository: R, refresh_delay: Duration) -> Self {
        Self {
            repository,
            refresh_delay,
            refresh_mutex: Arc::default(),
            values: Arc::default(),
        }
    }
}

impl<R> WalletRepoFlags<R>
where
    R: WalletFlagRepository + Send,
{
    async fn refresh_flags(&self) -> Result<(), PersistenceError> {
        // Use lock to ensure that this method is called sequentially
        let _guard = self.refresh_mutex.lock().await;

        // Fetch the values before the lock to not hold it over async points.
        // This works because the previous async mutex guarantees the rest of
        // the method is called sequentially. This prevents intertwining of the
        // fetching and the writing which can cause old data to overwrite new.
        let new_values = self.repository.fetch_flags().await?.into_iter().collect();
        *self.values.write() = new_values;
        Ok(())
    }
}

impl<R> WalletRepoFlags<R>
where
    R: WalletFlagRepository + Clone + Send + Sync + 'static,
{
    pub fn start_refresh_job(&self) -> AbortHandle {
        let flags = self.clone();
        let mut interval = tokio::time::interval(self.refresh_delay);
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        tokio::spawn(async move {
            tracing::info!("Starting refresh job for flags");
            loop {
                interval.tick().await;

                // Wrap in separate spawn job to catch panics
                let job_flags = flags.clone();
                if let Err(err) = tokio::spawn(async move {
                    if let Err(err) = job_flags.refresh_flags().await {
                        tracing::error!("Could not fetch flags: {:?}", err);
                    }
                })
                .await
                {
                    tracing::error!("Join error on refresh job for flags: {:?}", err);
                };
            }
        })
        .abort_handle()
    }
}

impl<R> WalletFlags for WalletRepoFlags<R>
where
    R: WalletFlagRepository,
{
    type Error = PersistenceError;

    async fn set_solution_revoked(&self) -> Result<(), Self::Error> {
        self.repository.set_flag(WalletFlag::SolutionRevoked).await?;
        self.refresh_flags().await
    }

    fn solution_is_revoked(&self) -> bool {
        self.values
            .read()
            .get(&WalletFlag::SolutionRevoked)
            .copied()
            .unwrap_or(false)
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::convert::Infallible;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;

    use crate::flags::WalletFlags;

    #[derive(Default)]
    pub struct StubWalletFlags {
        solution_revoked: AtomicBool,
    }

    impl StubWalletFlags {
        pub fn set_solution_revoked(&self, value: bool) {
            self.solution_revoked.store(value, Ordering::Relaxed);
        }
    }

    impl WalletFlags for StubWalletFlags {
        type Error = Infallible;

        async fn set_solution_revoked(&self) -> Result<(), Self::Error> {
            self.set_solution_revoked(true);
            Ok(())
        }

        fn solution_is_revoked(&self) -> bool {
            self.solution_revoked.load(Ordering::Relaxed)
        }
    }
}
