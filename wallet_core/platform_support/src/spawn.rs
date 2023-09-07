use std::panic;

use tokio::task;

pub async fn blocking<R, E, FE>(fun: impl FnOnce() -> Result<R, FE> + Send + Sync + 'static) -> Result<R, E>
where
    R: Send + Sync + 'static,
    FE: Send + Sync + 'static,
    E: From<FE>,
{
    let result = task::spawn_blocking(fun)
        .await
        .unwrap_or_else(|e| panic::resume_unwind(e.into_panic()))?;

    Ok(result)
}
