//! Verifies a wallet configuration JWT in the same way the wallet does when it fetches one from the configuration
//! server.

use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use crypto::PublicKey;
use jwt::JwtDecodingKey;
use jwt::UnverifiedJwt;
use wallet_configuration::config_server_config::ConfigServerConfiguration;
use wallet_configuration::verification::WALLET_CONFIG_VALIDATION;
use wallet_configuration::wallet_config::WalletConfiguration;

#[derive(Parser)]
#[command(about, long_about = None)]
struct Cli {
    /// Configuration server configuration JSON, containing the public key the wallet uses to verify the JWT
    config_server_config: PathBuf,

    /// Wallet configuration JWT, as served by the configuration server
    wallet_config_jwt: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_server_config: ConfigServerConfiguration = serde_json::from_slice(
        &fs::read(&cli.config_server_config)
            .with_context(|| format!("could not read {}", cli.config_server_config.display()))?,
    )
    .context("could not parse configuration server configuration")?;

    let jwt: UnverifiedJwt<WalletConfiguration> = fs::read_to_string(&cli.wallet_config_jwt)
        .with_context(|| format!("could not read {}", cli.wallet_config_jwt.display()))?
        .parse()
        .context("could not parse wallet configuration JWT")?;

    let signing_public_key: JwtDecodingKey =
        PublicKey::from(*config_server_config.signing_public_key.as_inner()).into();

    let (_, wallet_config) = jwt
        .parse_and_verify(&signing_public_key, &*WALLET_CONFIG_VALIDATION)
        .context("wallet configuration JWT would be rejected by the wallet")?;

    println!(
        "Wallet configuration version {} for environment '{}' is valid",
        wallet_config.version, wallet_config.environment
    );

    Ok(())
}
