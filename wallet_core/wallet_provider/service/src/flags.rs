use std::ops::Index;
use std::sync::Arc;
use std::time::Duration;

use itertools::Itertools;
use parking_lot::RwLock;
use strum::VariantArray;
use tokio::sync::Mutex;
use tokio::task::AbortHandle;
use tokio::time::MissedTickBehavior;

use status_lists::postgres::RevokeAll;
use wallet_provider_domain::model::wallet_flag::WalletFlag;
use wallet_provider_domain::repository::PersistenceError;
use wallet_provider_domain::repository::WalletFlagRepository;

#[derive(Debug)]
struct FlagArray([bool; WalletFlag::VARIANTS.len()]);

impl Index<WalletFlag> for FlagArray {
    type Output = bool;

    fn index(&self, index: WalletFlag) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl FromIterator<(WalletFlag, bool)> for FlagArray {
    fn from_iter<T: IntoIterator<Item = (WalletFlag, bool)>>(iter: T) -> Self {
        let mut flags = [false; WalletFlag::VARIANTS.len()];
        for (flag, value) in iter {
            flags[flag as usize] = value;
        }
        Self(flags)
    }
}

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
    values: Arc<RwLock<FlagArray>>,
}

impl<R> WalletRepoFlags<R>
where
    R: WalletFlagRepository,
{
    pub async fn try_new(repository: R, refresh_delay: Duration) -> Result<Self, PersistenceError> {
        // Initially fetch flags to ensure the flags are never empty
        let values = Self::fetch_flags(&repository).await?;
        Ok(Self {
            repository,
            refresh_delay,
            refresh_mutex: Arc::default(),
            values: Arc::new(RwLock::new(values)),
        })
    }

    async fn fetch_flags(repository: &R) -> Result<FlagArray, PersistenceError> {
        Ok(repository.fetch_flags().await?.into_iter().collect())
    }
}

fn update_diff(old_values: &FlagArray, new_values: &FlagArray) -> Vec<(WalletFlag, bool)> {
    let mut diff = Vec::with_capacity(WalletFlag::VARIANTS.len());
    for flag in WalletFlag::VARIANTS {
        let new_value = new_values[*flag];
        if old_values[*flag] != new_value {
            diff.push((*flag, new_value))
        }
    }
    diff
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
        let new_values = Self::fetch_flags(&self.repository).await?;

        // Check for update
        let diff = update_diff(&self.values.read(), &new_values);

        // Write update
        if !diff.is_empty() {
            let diff = diff
                .into_iter()
                .map(|(flag, value)| format!("{flag}={value}"))
                .join(", ");
            tracing::info!("Setting new flags: {diff}");
            *self.values.write() = new_values;
        }

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
        self.values.read()[WalletFlag::SolutionRevoked]
    }
}

impl<R> RevokeAll for WalletRepoFlags<R>
where
    R: WalletFlagRepository + Sync,
{
    type Error = PersistenceError;

    async fn is_revoked_all(&self) -> Result<bool, Self::Error> {
        // Directly query the database as the republish all is already done and
        // new status lists can get created which should be published with all
        // invalid.
        self.repository.get_flag(WalletFlag::SolutionRevoked).await
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::convert::Infallible;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;

    use status_lists::postgres::RevokeAll;

    use crate::flags::WalletFlags;

    #[derive(Default, Clone)]
    pub struct StubWalletFlags {
        solution_revoked: Arc<AtomicBool>,
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

    impl RevokeAll for StubWalletFlags {
        type Error = Infallible;

        async fn is_revoked_all(&self) -> Result<bool, Infallible> {
            Ok(self.solution_is_revoked())
        }
    }
}
