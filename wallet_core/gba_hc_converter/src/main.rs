use std::error::Error;

use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

use gba_hc_converter::{app, haal_centraal, settings::Settings};

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

    haal_centraal::initialize_eager();

    info!("Run mode: {}", settings.run_mode);

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async { app::serve_from_settings(settings).await })
}
