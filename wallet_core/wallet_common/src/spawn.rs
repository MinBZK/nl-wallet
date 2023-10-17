use std::panic;

use tokio::task;

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
