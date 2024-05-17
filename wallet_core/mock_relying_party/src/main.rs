use anyhow::Result;

use mock_relying_party::{server, settings::Settings};
use wallet_common::try_init_sentry;

// Cannot use #[tokio::main], see: https://docs.sentry.io/platforms/rust/#async-main-function
fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    // Retain [`ClientInitGuard`]
    let _guard = try_init_sentry!(settings.sentry);

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async { server::serve(settings).await })
}
