use std::time::Duration;

use tokio::task::AbortHandle;
use tokio::time;
use tokio::time::MissedTickBehavior;

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
