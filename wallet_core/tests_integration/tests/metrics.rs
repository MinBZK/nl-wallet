use hsm::service::Pkcs11Hsm;
use http_utils::reqwest::trusted_reqwest_client_builder;
use tests_integration::common::start_wallet_provider;
use tests_integration::common::wallet_provider_settings;

#[tokio::test]
async fn test_wallet_provider_metrics() {
    let (settings, root_ca) = wallet_provider_settings();
    let hsm = Pkcs11Hsm::from_settings(settings.hsm.clone()).expect("Could not initialize HSM");
    let port = start_wallet_provider(settings, hsm, root_ca.clone()).await;

    let client = trusted_reqwest_client_builder(std::iter::once(root_ca.into_certificate()))
        .build()
        .unwrap();

    let response = client
        .get(format!("https://localhost:{port}/metrics"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    let body: String = response.text().await.unwrap();
    assert!(body.contains("nlwallet_metrics_endpoint_count"));
}
