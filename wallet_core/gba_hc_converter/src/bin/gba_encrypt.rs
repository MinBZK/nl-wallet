use std::io::Read;

use aes_gcm::Aes256Gcm;
use anyhow::Result;
use anyhow::bail;
use clap::Parser;
use clio::ClioPath;
use clio::Input;

use utils::built_info::version_string;
use utils::path::prefix_local_path;

use gba_hc_converter::gba::encryption::HmacSha256;
use gba_hc_converter::gba::encryption::encrypt_bytes_to_dir;
use gba_hc_converter::settings::RunMode;
use gba_hc_converter::settings::Settings;

#[derive(Parser)]
#[command(version=version_string(), about, long_about = None)]
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

    let base_path = prefix_local_path(cli.output.path());

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
        base_path.as_ref(),
        &cli.basename,
    )
    .await?;

    Ok(())
}
