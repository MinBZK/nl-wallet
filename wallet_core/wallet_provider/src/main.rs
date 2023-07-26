use std::error::Error;

use tracing::info;

use wallet_provider::{server, settings::Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    info!("Account server public key: {}", settings.certificate_private_key);
    info!(
        "Instruction signing public key: {}",
        settings.instruction_result_private_key
    );

    server::serve(settings).await?;

    Ok(())
}
