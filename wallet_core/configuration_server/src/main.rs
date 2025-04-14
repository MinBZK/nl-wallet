use std::error::Error;

use rustls::crypto::ring;
use rustls::crypto::CryptoProvider;

use crate::settings::Settings;

mod server;
mod settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    CryptoProvider::install_default(ring::default_provider()).unwrap();

    let settings = Settings::new()?;

    server::serve(settings).await
}
