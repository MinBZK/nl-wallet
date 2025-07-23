use std::error::Error;
use std::fs::OpenOptions;

use cfg_if::cfg_if;
use rustls::crypto::CryptoProvider;
use rustls::crypto::ring;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use android_attest::android_crl::GoogleRevocationListClient;
use hsm::service::Pkcs11Hsm;
use http_utils::reqwest::default_reqwest_client_builder;
use wallet_provider::logging::redirect_stdout_stderr_to_log;
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
    let log_redirect = if settings.structured_logging {
        let builder = builder.json();
        match settings.capture_and_redirect_logging.clone() {
            None => {
                builder.init();
                None
            }
            Some(path) => {
                let writer = OpenOptions::new().append(true).open(path)?;
                builder.with_writer(writer).init();
                Some(redirect_stdout_stderr_to_log()?)
            }
        }
    } else {
        builder.init();
        None
    };

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

    server::serve(settings, hsm, google_crl_client, play_integrity_client).await?;

    if let Some(log_redirect) = log_redirect {
        let _ = log_redirect.stop_and_wait();
    }
    Ok(())
}
