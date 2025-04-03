use anyhow::Result;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::settings::Settings;

mod config;
mod server;
mod settings;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
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

    server::serve(settings).await
}
