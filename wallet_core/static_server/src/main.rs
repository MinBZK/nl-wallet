use std::error::Error;

use rustls::crypto::CryptoProvider;
use rustls::crypto::ring;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::settings::Settings;

mod server;
mod settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    CryptoProvider::install_default(ring::default_provider()).unwrap();

    let settings = Settings::new()?;

    server::serve(settings).await
}
