use anyhow::Result;

use wallet_server::{server, settings::Settings, store::DisclosureSessionStore};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    let sessions = DisclosureSessionStore::init(settings.store_url.clone()).await?;
    // This will block until the server shuts down.
    server::serve(&settings, sessions).await?;

    Ok(())
}
