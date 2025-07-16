use std::error::Error;

use cfg_if::cfg_if;
use rustls::crypto::CryptoProvider;
use rustls::crypto::ring;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use android_attest::android_crl::GoogleRevocationListClient;
use hsm::service::Pkcs11Hsm;
use http_utils::reqwest::default_reqwest_client_builder;
use wallet_provider::server;
use wallet_provider::settings::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    CryptoProvider::install_default(ring::default_provider()).unwrap();

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

    let hsm = Pkcs11Hsm::from_settings(settings.hsm.clone())?;

    let reqwest_client = default_reqwest_client_builder().build()?;
    let google_crl_client = GoogleRevocationListClient::new(reqwest_client.clone());

    cfg_if! {
        if #[cfg(feature = "mock_android_integrity_verdict")] {
            use wallet_provider_service::account_server::mock_play_integrity::MockPlayIntegrityClient;

            tracing::warn!("DANGEROUS - Android integrity verdicts are mocked. This should NOT be used in production!");

            let play_integrity_client = MockPlayIntegrityClient::new(
                settings.android.package_name.clone(),
                settings.android.play_store_certificate_hashes.clone()
            );
        } else {
            use android_attest::play_integrity::client::PlayIntegrityClient;
            use android_attest::play_integrity::client::ServiceAccountAuthenticator;
            use utils::path::prefix_local_path;

            let credentials_file_path = prefix_local_path(&settings.android.credentials_file);
            let play_integrity_client = PlayIntegrityClient::new(
                reqwest_client,
                ServiceAccountAuthenticator::new(credentials_file_path.as_ref()).await?,
                &settings.android.package_name,
            )?;
        }
    }

    server::serve(settings, hsm, google_crl_client, play_integrity_client).await
}
