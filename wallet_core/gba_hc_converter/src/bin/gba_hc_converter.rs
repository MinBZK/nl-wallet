use std::error::Error;

use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

use gba_hc_converter::{app, haal_centraal, settings::Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let builder = tracing_subscriber::fmt().with_env_filter(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy(),
    );

    let settings = Settings::new()?;
    if settings.structured_logging {
        builder.json().init();
    } else {
        builder.init();
    }

    haal_centraal::initialize_eager();

    info!("Run mode: {}", settings.run_mode);

    app::serve_from_settings(settings).await
}
