use std::{env, io::Read, path::PathBuf};

use anyhow::{bail, Result};
use clap::Parser;
use clio::*;

use gba_hc_converter::{
    gba::encryption::encrypt_bytes_to_dir,
    settings::{RunMode, Settings},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input to encrypt
    #[clap(value_parser, default_value = "-")]
    input: Input,

    /// Directory to store encrypted files in
    #[clap(long, short, value_parser = clap::value_parser!(ClioPath).exists().is_dir(), default_value = ".")]
    output: ClioPath,

    /// Name of the encrypted files (without extension)
    #[clap(long, short)]
    basename: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut cli = Cli::parse();

    let basename = cli.basename;

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

    encrypt_bytes_to_dir(preloaded_settings.symmetric_key.key(), &bytes, &base_path, &basename).await?;

    Ok(())
}
