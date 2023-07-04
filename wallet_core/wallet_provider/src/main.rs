mod app;
mod app_dependencies;
mod settings;

use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD, Engine};
use p256::{ecdsa::SigningKey, pkcs8::DecodePrivateKey};

use crate::{app_dependencies::AppDependencies, settings::Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;
    let (ip, port) = (settings.webserver.ip, settings.webserver.port);

    dbg!(STANDARD.encode(
        SigningKey::from_pkcs8_der(&settings.signing_private_key.0)
            .unwrap()
            .verifying_key()
            .to_encoded_point(false)
            .as_bytes()
    ));

    let app = app::router(Arc::new(AppDependencies::new_from_settings(settings).await?));

    let socket = SocketAddr::new(ip, port);
    tracing::debug!("listening on {}", socket);

    axum::Server::bind(&socket).serve(app.into_make_service()).await?;

    Ok(())
}
