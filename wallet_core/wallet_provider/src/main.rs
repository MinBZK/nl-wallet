use std::error::Error;

use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

use wallet_common::account::serialization::DerVerifyingKey;
use wallet_provider::{server, settings::Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = Settings::new()?;

    let builder = tracing_subscriber::fmt().with_env_filter(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy(),
    );
    if settings.structured_logging {
        builder.json().init();
    } else {
        builder.init()
    }

    info!(
        "Account server public key: {}",
        DerVerifyingKey::from(&settings.certificate_private_key)
    );
    info!(
        "Instruction signing public key: {}",
        DerVerifyingKey::from(&settings.instruction_result_private_key)
    );

    server::serve(settings).await?;

    Ok(())
}
