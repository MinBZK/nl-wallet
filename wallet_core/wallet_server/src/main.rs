use anyhow::Result;

use wallet_server::{server, settings::Settings, store::SessionStores};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    let sessions = SessionStores::init(settings.store_url.clone()).await?;

    // This will block until the server shuts down.
    #[cfg(feature = "issuance")]
    server::serve_full(
        // TODO: `MockPidAttributeService` issues a configured set of mock attributes. Replace with BRP query.
        wallet_server::pid::attributes::MockPidAttributeService::new(
            settings.issuer.digid.issuer_url.clone(),
            settings.issuer.digid.bsn_privkey.clone(),
            settings.issuer.digid.client_id.clone(),
            settings.issuer.mock_data.clone(),
        )?,
        settings,
        sessions,
    )
    .await?;
    #[cfg(not(feature = "issuance"))]
    server::serve_disclosure(settings, sessions).await?;

    Ok(())
}
