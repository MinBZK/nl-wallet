use anyhow::Result;

use nl_wallet_mdoc::{server_state::MemorySessionStore, verifier::DisclosureData};
use wallet_server::{server, settings::Settings};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    let sessions = MemorySessionStore::<DisclosureData>::new();
    // This will block until the server shuts down.
    server::serve(&settings, sessions).await?;

    Ok(())
}
