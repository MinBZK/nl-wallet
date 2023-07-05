use std::error::Error;
use std::net::{SocketAddr, TcpListener};

use base64::{engine::general_purpose::STANDARD, Engine};
use p256::{ecdsa::SigningKey, pkcs8::DecodePrivateKey};

use wallet_provider::server;
use wallet_provider::settings::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    dbg!(STANDARD.encode(
        SigningKey::from_pkcs8_der(&settings.signing_private_key.0)
            .unwrap()
            .verifying_key()
            .to_encoded_point(false)
            .as_bytes()
    ));

    let (ip, port) = (settings.webserver.ip, settings.webserver.port);
    let listener = TcpListener::bind(SocketAddr::new(ip, port))?;

    server::serve(listener, settings).await?;

    Ok(())
}
