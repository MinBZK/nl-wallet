use anyhow::Result;
use tracing::debug;

use pid_issuer::{digid::OpenIdClient, mock_attributes::MockAttributesLookup, server, settings::Settings};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    debug!("Discovering DigiD issuer...");
    let bsn_lookup = OpenIdClient::new(&settings.digid).await?;

    // TODO: `MockAttributesLookup` issues a configured set of mock attributes. Replace with BRP query.
    let attributes_lookup = MockAttributesLookup::from(settings.mock_data.clone().unwrap_or_default());
    // This will block until the server shuts down.
    server::serve(settings, attributes_lookup, bsn_lookup).await?;

    Ok(())
}
