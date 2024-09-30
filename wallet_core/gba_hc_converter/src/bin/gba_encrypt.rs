use std::{env, io::Read, path::PathBuf};

use aes_gcm::Aes256Gcm;
use anyhow::{bail, Result};
use clap::Parser;
use clio::{ClioPath, Input};

use gba_hc_converter::{
    gba::encryption::{encrypt_bytes_to_dir, HmacSha256},
    settings::{RunMode, Settings},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input to encrypt
    #[clap(value_parser, default_value = "-")]
    input: Input,

    /// Directory to store encrypted file and nonce in
    #[clap(long, short, value_parser = clap::value_parser!(ClioPath).exists().is_dir(), default_value = ".")]
    output: ClioPath,

    /// Name of the encrypted file (without extension)
    #[clap(long, short)]
    basename: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut cli = Cli::parse();

    let mut bytes: Vec<u8> = vec![];
    cli.input.read_to_end(&mut bytes)?;

    let base_path = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(cli.output.path());

    let settings = Settings::new()?;

    let preloaded_settings = match settings.run_mode {
        RunMode::All { preloaded, .. } => preloaded,
        RunMode::Preloaded(preloaded) => preloaded,
        _ => bail!("Only Runmode::All and Runmode::Preloaded are allowed"),
    };

    encrypt_bytes_to_dir(
        preloaded_settings.encryption_key.key::<Aes256Gcm>(),
        preloaded_settings.hmac_key.key::<HmacSha256>(),
        &bytes,
        &base_path,
        &cli.basename,
    )
    .await?;

    Ok(())
}
