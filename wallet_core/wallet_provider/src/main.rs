use std::error::Error;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use wallet_provider::{server, settings::Settings};

// Cannot use #[tokio::main], see: https://docs.sentry.io/platforms/rust/#async-main-function
fn main() -> Result<(), Box<dyn Error>> {
    let settings = Settings::new()?;

    // Retain [`ClientInitGuard`]
    let _guard = settings
        .sentry
        .as_ref()
        .map(|sentry| sentry.init(sentry::release_name!()));

    let builder = tracing_subscriber::fmt().with_env_filter(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy(),
    );
    if settings.structured_logging {
        builder.json().init();
    } else {
        builder.init();
    }

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async { server::serve(settings).await })
}
