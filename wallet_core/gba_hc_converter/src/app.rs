use tokio::net::TcpListener;

use crate::gba::client::FileGbavClient;
use crate::gba::client::HttpGbavClient;
use crate::gba::client::NoopGbavClient;
use crate::server;
use crate::settings::RunMode;
use crate::settings::Settings;

pub async fn serve_from_settings(settings: Settings) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind((settings.ip, settings.port)).await?;
    match settings.run_mode {
        RunMode::Gbav(gbav) => {
            let http_client = HttpGbavClient::from_settings(gbav).await?;
            server::serve(listener, http_client).await
        }
        RunMode::Preloaded(preloaded) => {
            let file_client = FileGbavClient::try_from_settings(preloaded, NoopGbavClient {})?;
            server::serve(listener, file_client).await
        }
        RunMode::All { gbav, preloaded } => {
            let http_client = HttpGbavClient::from_settings(gbav).await?;
            let file_client = FileGbavClient::try_from_settings(preloaded, http_client)?;
            server::serve(listener, file_client).await
        }
    }
}
