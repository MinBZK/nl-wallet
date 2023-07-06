use std::error::Error;

use base64::{engine::general_purpose::STANDARD, Engine};
use p256::{ecdsa::SigningKey, pkcs8::DecodePrivateKey};
use tracing::debug;

use wallet_provider::{server, settings::Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    debug!(
        "Account server public key: {}",
        STANDARD.encode(
            SigningKey::from_pkcs8_der(&settings.signing_private_key.0)?
                .verifying_key()
                .to_encoded_point(false)
                .as_bytes()
        )
    );

    server::serve(settings).await?;

    Ok(())
}
