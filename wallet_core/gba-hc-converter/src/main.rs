use std::error::Error;

use crate::{
    gba::client::{FileGbavClient, HttpGbavClient, NoopGbavClient},
    settings::{RunMode, Settings},
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

    match settings.run_mode {
        RunMode::Gbav(gbav) => {
            let http_client: HttpGbavClient = gbav.try_into()?;
            server::serve(settings.ip, settings.port, http_client).await?;
        }
        RunMode::Preloaded(preloaded) => {
            let file_client = FileGbavClient::new(preloaded.xml_path.into(), NoopGbavClient {});
            server::serve(settings.ip, settings.port, file_client).await?;
        }
        RunMode::All { gbav, preloaded } => {
            let http_client: HttpGbavClient = gbav.try_into()?;
            let file_client = FileGbavClient::new(preloaded.xml_path.into(), http_client);
            server::serve(settings.ip, settings.port, file_client).await?;
        }
    }

    Ok(())
}
