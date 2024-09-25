use std::{env, path::PathBuf};

use aes_gcm::Aes256Gcm;
use anyhow::{anyhow, bail, Result};
use clap::Parser;
use clio::*;

use gba_hc_converter::{
    gba::{
        client::{GbavClient, HttpGbavClient},
        encryption::{encrypt_bytes_to_dir, name_to_encoded_hash},
    },
    haal_centraal::Bsn,
    settings::{RunMode, Settings},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Directory to store encrypted file and nonce in
    #[clap(long, short, value_parser = clap::value_parser!(ClioPath).exists().is_dir(), default_value = ".")]
    output: ClioPath,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let base_path = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(cli.output.path());

    let bsn = Bsn::try_new(&rpassword::prompt_password("Enter BSN: ")?)?;

    let settings = Settings::new()?;

    let (gbav_settings, preloaded_settings) = match settings.run_mode {
        RunMode::All { gbav, preloaded } => (gbav, preloaded),
        _ => bail!("Only Runmode::All is allowed"),
    };

    let http_client = HttpGbavClient::from_settings(gbav_settings).await?;

    let xml = http_client
        .vraag(&bsn)
        .await?
        .ok_or(anyhow!("No GBA-V results found for the supplied BSN"))?;

    let name = name_to_encoded_hash(&bsn.to_string(), &preloaded_settings.hmac_key);

    encrypt_bytes_to_dir(
        preloaded_settings.encryption_key.key::<Aes256Gcm>(),
        xml.as_bytes(),
        &base_path,
        &name,
    )
    .await?;

    Ok(())
}
