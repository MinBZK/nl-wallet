use anyhow::Result;
use clap::Parser;

use wallet_server::{server, settings::Settings, store::SessionStores};

#[cfg(feature = "issuance")]
use wallet_server::pid::{attributes::BrpPidAttributeService, brp::client::HttpBrpClient};

/// wallet_server
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Configuration file.
    #[arg(short, long, default_value = "wallet_server.toml")]
    config_file: String,

    /// Prefix to be used for environment variables. Environment variables will be upper case, so default prefix is:
    /// `WALLET_SERVER`.
    #[arg(short, long, default_value = "wallet_server")]
    env_prefix: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let settings = Settings::new_custom(&args.config_file, &args.env_prefix)?;

    let storage_settings = &settings.storage;
    let sessions = SessionStores::init(storage_settings.url.clone(), storage_settings.into()).await?;

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
