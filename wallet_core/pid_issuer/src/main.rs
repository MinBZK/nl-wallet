use anyhow::Result;
use tracing::debug;

use pid_issuer::{app::mock::MockAttributesLookup, digid::OpenIdClient, server, settings::Settings};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    debug!("Discovering DigiD issuer...");
    let bsn_lookup = OpenIdClient::new(&settings.digid).await?;

    // This will block unil the server shuts down.
    // TODO: `MockAttributesLookup` issues a hardcoded set of mock attributes. Replace with BRP query.
    server::serve(settings, MockAttributesLookup, bsn_lookup).await?;

    Ok(())
}
