use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use tokio::task::AbortHandle;
use tokio::time::MissedTickBehavior;

use wallet_provider_domain::model::wallet_flag::WalletFlag;
use wallet_provider_domain::repository::WalletFlagRepository;

pub trait WalletFlags {
    fn solution_is_revoked(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct WalletRepoFlags<R> {
    repository: R,
    refresh_delay: Duration,
    values: Arc<RwLock<HashMap<WalletFlag, bool>>>,
}

impl<R> WalletRepoFlags<R> {
    pub fn new(repository: R, refresh_delay: Duration) -> Self {
        Self {
            repository,
            refresh_delay,
            values: Arc::default(),
        }
    }
}

impl<R> WalletRepoFlags<R>
where
    R: WalletFlagRepository + Clone + Send + Sync + 'static,
{
    async fn refresh_flags(&self) {
        // Fetch the values before the lock to not hold it over async points.
        // This works only because there is a single background task calling
        // this method sequentially. Otherwise, intertwining of the fetching and
        // the writing can happen, which can cause old data to overwrite new.
        let new_values = match self.repository.fetch_flags().await {
            Ok(values) => values.into_iter().collect(),
            Err(err) => {
                tracing::error!("Could not fetch flags: {:?}", err);
                return;
            }
        };
        *self.values.write() = new_values;
    }

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
                if let Err(err) = tokio::spawn(async move { job_flags.refresh_flags().await }).await {
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
        fn solution_is_revoked(&self) -> bool {
            self.solution_revoked.load(Ordering::Relaxed)
        }
    }
}
