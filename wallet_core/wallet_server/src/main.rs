use anyhow::Result;
use clap::Parser;

use wallet_server::{server, settings::Settings, store::SessionStores};

#[cfg(feature = "issuance")]
use wallet_server::pid::{attributes::BrpPidAttributeService, brp::client::HttpBrpClient};

/// WalletServer
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Configuration file. Will be interpreted
    #[arg(short, long, default_value = "wallet_server.toml")]
    config_file: String,

    ///
    #[arg(short, long, default_value = "wallet_server")]
    env_prefix: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let settings = Settings::new_custom(&args.config_file, &args.env_prefix)?;

    let sessions = SessionStores::init(settings.store_url.clone()).await?;

    // This will block until the server shuts down.
    #[cfg(feature = "issuance")]
    server::serve_full(
        BrpPidAttributeService::new(
            HttpBrpClient::new(settings.issuer.brp_server.clone()),
            settings.issuer.digid.issuer_url.clone(),
            settings.issuer.digid.bsn_privkey.clone(),
            settings.issuer.digid.trust_anchors.clone(),
            settings.issuer.certificates(),
        )?,
        settings,
        sessions,
    )
    .await?;

    #[cfg(not(feature = "issuance"))]
    server::serve_disclosure(settings, sessions).await?;

    Ok(())
}
