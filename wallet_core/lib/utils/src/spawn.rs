use std::panic;
use std::time::Duration;

use tokio::task;
use tokio::task::AbortHandle;
use tokio::time;
use tokio::time::MissedTickBehavior;

pub async fn blocking<F, R, E>(fun: F) -> Result<R, E>
where
    F: FnOnce() -> Result<R, E> + Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
{
    task::spawn_blocking(fun)
        .await
        .unwrap_or_else(|e| panic::resume_unwind(e.into_panic()))
}

/// Spawn a background task that awaits `task` once every `interval`, starting immediately.
///
/// Missed ticks are delayed rather than bursting (so a slow run does not cause back-to-back
/// executions). The loop never returns, so the returned [`AbortHandle`] is the only way to stop it:
/// drop is not enough, the caller must `abort()` (or hold the handle for the desired lifetime).
pub fn start_recurring_task<F, FR>(interval: Duration, mut task: F) -> AbortHandle
where
    F: FnMut() -> FR + Send + 'static,
    FR: Future<Output = ()> + Send,
{
    let mut interval = time::interval(interval);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    tokio::spawn(async move {
        loop {
            interval.tick().await;

            task().await;
        }
    })
    .abort_handle()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use tokio::time;

    use super::start_recurring_task;

    #[tokio::test(start_paused = true)]
    async fn start_recurring_task_runs_each_interval_until_aborted() {
        let interval = Duration::from_secs(120);
        let counter = Arc::new(AtomicUsize::new(0));

        let task_counter = Arc::clone(&counter);
        let handle = start_recurring_task(interval, move || {
            let task_counter = Arc::clone(&task_counter);
            async move {
                task_counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        // Advance the (paused) clock, then yield so the woken background task gets to run before we
        // observe the counter.
        async fn advance(duration: Duration) {
            time::advance(duration).await;
            tokio::task::yield_now().await;
        }

        // `tokio::time::interval` fires its first tick immediately, so the task runs exactly once
        // before any interval has elapsed.
        advance(Duration::from_millis(1)).await;
        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "task should fire exactly once on the first tick"
        );

        // It does not fire again until a full interval has elapsed.
        advance(interval - Duration::from_millis(2)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1, "task must not fire between ticks");

        // Crossing each interval boundary triggers exactly one more run.
        advance(Duration::from_millis(2)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 2, "task should fire once per interval");
        advance(interval).await;
        assert_eq!(counter.load(Ordering::SeqCst), 3, "task should fire once per interval");

        // Aborting stops further runs.
        handle.abort();
        advance(interval * 3).await;
        assert_eq!(counter.load(Ordering::SeqCst), 3, "no runs should occur after abort");
    }
}
