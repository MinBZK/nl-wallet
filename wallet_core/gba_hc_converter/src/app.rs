use crate::{
    gba::client::{FileGbavClient, HttpGbavClient, NoopGbavClient},
    server,
    settings::{RunMode, Settings},
};

pub async fn serve_from_settings(settings: Settings) -> Result<(), Box<dyn std::error::Error>> {
    match settings.run_mode {
        RunMode::Gbav(gbav) => {
            let http_client = HttpGbavClient::from_settings(gbav).await?;
            server::serve(settings.ip, settings.port, http_client).await
        }
        RunMode::Preloaded(preloaded) => {
            let file_client = FileGbavClient::try_from_settings(preloaded, NoopGbavClient {})?;
            server::serve(settings.ip, settings.port, file_client).await
        }
        RunMode::All { gbav, preloaded } => {
            let http_client = HttpGbavClient::from_settings(gbav).await?;
            let file_client = FileGbavClient::try_from_settings(preloaded, http_client)?;
            server::serve(settings.ip, settings.port, file_client).await
        }
    }
}
