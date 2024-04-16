use std::error::Error;

use crate::{
    gba::client::{FileGbavClient, HttpGbavClient},
    settings::Settings,
};

mod error;
mod gba;
mod haal_centraal;
mod server;
mod settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    haal_centraal::initialize_eager();

    let settings = Settings::new()?;

    let http_client = HttpGbavClient::new(
        settings.url,
        settings.username,
        settings.password,
        settings.trust_anchor,
        settings.client_cert,
        settings.client_cert_key,
    )?;

    if let Some(path) = settings.preloaded_xml_path {
        let file_client = FileGbavClient::new(path.into(), http_client);
        server::serve(settings.ip, settings.port, file_client).await?;
    } else {
        server::serve(settings.ip, settings.port, http_client).await?;
    };

    Ok(())
}
