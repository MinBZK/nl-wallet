use std::error::Error;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use android_attest::android_crl::GoogleRevocationListClient;
use android_attest::play_integrity::client::PlayIntegrityClient;
use android_attest::play_integrity::client::ServiceAccountAuthenticator;
use wallet_common::reqwest::default_reqwest_client_builder;
use wallet_provider::server;
use wallet_provider::settings::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let builder = tracing_subscriber::fmt().with_env_filter(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy(),
    );

    let settings = Settings::new()?;
    if settings.structured_logging {
        builder.json().init();
    } else {
        builder.init();
    }

    let reqwest_client = default_reqwest_client_builder().build()?;
    let google_crl_client = GoogleRevocationListClient::new(reqwest_client.clone());
    let play_integrity_client = PlayIntegrityClient::new(
        reqwest_client,
        ServiceAccountAuthenticator::new(settings.android.credentials_file_absolute().as_ref()).await?,
        &settings.android.package_name,
    )?;

    server::serve(settings, google_crl_client, play_integrity_client).await
}
