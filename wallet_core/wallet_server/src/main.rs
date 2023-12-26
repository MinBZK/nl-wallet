use anyhow::Result;

#[cfg(feature = "issuance")]
use openid4vc::issuer::AttributeService;

use wallet_server::{server, settings::Settings, store::new_session_stores};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    let sessions = new_session_stores(settings.store_url.clone()).await?;

    // This will block until the server shuts down.
    #[cfg(feature = "issuance")]
    server::serve_full(
        &settings,
        sessions,
        // TODO: `MockPidAttributeService` issues a configured set of mock attributes. Replace with BRP query.
        wallet_server::pid::attributes::MockPidAttributeService::new(&settings.issuer).await?,
    )
    .await?;
    #[cfg(not(feature = "issuance"))]
    server::serve_disclosure(&settings, sessions).await?;

    Ok(())
}
