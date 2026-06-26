use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use tokio::task::AbortHandle;
use tracing::error;
use tracing::warn;
use utils::spawn::start_recurring_task;

/// Interval between [`PeriodicCleanup`] tasks removing expired/stale data.
pub const CLEANUP_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// Runs a single cleanup, logging any error in a uniform format.
///
/// Lets [`PeriodicCleanup`] implementations compose heterogeneous, fallible cleanups as infallible `()` futures: a
/// single store's failure is logged but never aborts the others.
pub async fn log_cleanup_error<E>(what: &str, cleanup: impl Future<Output = Result<(), E>>)
where
    E: Display,
{
    if let Err(error) = cleanup.await {
        warn!("error during {what} cleanup: {error}");
    }
}

/// Removes stale entries from the expiring storage a type owns.
///
/// Separates *what* to clean (the implementation) from *when* to clean (the scheduler below): an outer type can clean
/// its own stores plus those of any inner types it wraps. Implementations are responsible for logging their own errors,
/// since the scheduler only drives the cadence.
#[trait_variant::make(Send)]
pub trait PeriodicCleanup {
    async fn cleanup(&self);
}

/// Aborts the spawned cleanup task when dropped, so the task lives exactly as long as its owner.
pub struct CleanupTaskHandle(AbortHandle);

impl Drop for CleanupTaskHandle {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Spawns a background task that calls [`PeriodicCleanup::cleanup`] on `target` every `interval`.
///
/// The task is generic over the cleanup target, so scheduling is decoupled from the concrete issuer types. Each tick
/// runs in a child task so a panic in `cleanup` is contained: it is logged and the recurring task survives to the next
/// tick instead of silently dying. The returned [`CleanupTaskHandle`] must be retained for as long as cleanup should
/// keep running; dropping it aborts the task.
pub fn start_cleanup_task<C>(interval: Duration, target: Arc<C>) -> CleanupTaskHandle
where
    C: PeriodicCleanup + Send + Sync + 'static,
{
    CleanupTaskHandle(start_recurring_task(interval, move || {
        let target = Arc::clone(&target);
        async move {
            if let Err(join_error) = tokio::spawn(async move { target.cleanup().await }).await {
                error!("cleanup task panicked: {join_error}");
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use tokio::time;

    use super::PeriodicCleanup;
    use super::start_cleanup_task;

    struct Counter(AtomicUsize);

    impl PeriodicCleanup for Counter {
        async fn cleanup(&self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[tokio::test(start_paused = true)]
    async fn cleanup_task_runs_each_interval_and_stops_when_dropped() {
        let interval = Duration::from_secs(120);
        let counter = Arc::new(Counter(AtomicUsize::new(0)));

        let handle = start_cleanup_task(interval, Arc::clone(&counter));

        // Advance the (paused) clock, then yield so the woken background task runs before we observe
        // the counter. Avoids the race of sleeping a fixed wall-clock duration and hoping the task
        // was scheduled in time.
        async fn advance(duration: Duration) {
            time::advance(duration).await;
            tokio::task::yield_now().await;
        }

        // `tokio::time::interval` fires its first tick immediately.
        advance(Duration::from_millis(1)).await;
        assert_eq!(counter.0.load(Ordering::SeqCst), 1, "cleanup should run once on start");

        // Each elapsed interval triggers another cleanup.
        advance(interval).await;
        assert_eq!(
            counter.0.load(Ordering::SeqCst),
            2,
            "cleanup should run again per interval"
        );

        // Dropping the handle aborts the task: no further cleanups occur.
        drop(handle);
        advance(interval * 3).await;
        assert_eq!(
            counter.0.load(Ordering::SeqCst),
            2,
            "no cleanups should run after the handle is dropped"
        );
    }

    /// Panics on the first cleanup only; later ticks just count.
    struct PanicOnFirstTick(AtomicUsize);

    impl PeriodicCleanup for PanicOnFirstTick {
        async fn cleanup(&self) {
            if self.0.fetch_add(1, Ordering::SeqCst) == 0 {
                panic!("cleanup boom");
            }
        }
    }

    #[tokio::test(start_paused = true)]
    async fn cleanup_task_survives_a_panicking_cleanup() {
        let interval = Duration::from_secs(120);
        let target = Arc::new(PanicOnFirstTick(AtomicUsize::new(0)));

        let _handle = start_cleanup_task(interval, Arc::clone(&target));

        async fn advance(duration: Duration) {
            time::advance(duration).await;
            tokio::task::yield_now().await;
        }

        // First tick panics (contained by the per-tick child task); the recurring task must keep
        // running and clean up again on subsequent ticks.
        advance(Duration::from_millis(1)).await;
        advance(interval).await;
        advance(interval).await;

        assert!(
            target.0.load(Ordering::SeqCst) >= 2,
            "cleanup task should survive a panic and keep ticking"
        );
    }
}
