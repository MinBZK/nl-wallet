use std::io::Write;

use anyhow::{anyhow, bail, Result};
use clap::Parser;
use clio::*;

use gba_hc_converter::{
    gba::client::{GbavClient, HttpGbavClient},
    haal_centraal::Bsn,
    settings::{RunMode, Settings},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Output file, use '-' for stdout
    #[clap(long, short, value_parser, default_value = "-")]
    output_file: Output,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let bsn = Bsn::try_new(&rpassword::prompt_password("Enter BSN: ")?)?;

    let settings = Settings::new()?;

    let gbav_settings = match settings.run_mode {
        RunMode::All { gbav, .. } => gbav,
        RunMode::Gbav(gbav) => gbav,
        _ => bail!("Only Runmode::All and Runmode::Gbav are allowed"),
    };

    let http_client = HttpGbavClient::from_settings(gbav_settings).await?;

    let xml = http_client
        .vraag(&bsn)
        .await?
        .ok_or(anyhow!("No GBA-V results found for the supplied BSN"))?;

    let mut out = cli.output_file;
    out.write_all(xml.as_bytes())?;

    Ok(())
}
