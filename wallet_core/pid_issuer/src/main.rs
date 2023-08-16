use anyhow::Result;

use pid_issuer::{application::mock::MockAttributesLookup, openid::OpenIdClient, server, settings::Settings};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    // This will block unil the server shuts down.
    // TODO: `MockAttributesLookup` issues a hardcoded set of mock attributes. Replace with BRP query.
    server::serve::<MockAttributesLookup, OpenIdClient>(settings).await?;

    Ok(())
}
