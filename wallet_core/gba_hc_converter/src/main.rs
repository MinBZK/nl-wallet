use std::error::Error;

use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

use crate::settings::Settings;

mod app;
mod error;
mod gba;
mod haal_centraal;
mod server;
mod settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = Settings::new()?;

    let builder = tracing_subscriber::fmt().with_env_filter(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy(),
    );
    if settings.structured_logging {
        builder.json().init();
    } else {
        builder.init()
    }

    haal_centraal::initialize_eager();

    info!("Run mode: {}", settings.run_mode);

    app::serve_from_settings(settings).await?;

    Ok(())
}
