use reqwest::Client;
use serde_json::Value;
use serde_json::json;

use hsm::service::Pkcs11Hsm;
use http_utils::reqwest::trusted_reqwest_client_builder;
use http_utils::urls::BaseUrl;
use tests_integration::common::start_wallet_provider;
use tests_integration::common::wallet_provider_settings;

async fn setup_wallet_provider() -> (Client, BaseUrl) {
    let (settings, root_ca) = wallet_provider_settings();
    let hsm = Pkcs11Hsm::from_settings(settings.hsm.clone()).expect("Could not initialize HSM");
    let port = start_wallet_provider(settings, hsm, root_ca.clone()).await;

    let client = trusted_reqwest_client_builder(std::iter::once(root_ca.into_certificate()))
        .build()
        .unwrap();
    let url = format!("https://localhost:{}", port).parse().unwrap();
    (client, url)
}

#[tokio::test]
async fn test_wallet_provider_health() {
    let (client, url) = setup_wallet_provider().await;

    let response = client.get(url.join("health/live")).send().await.unwrap();
    assert!(response.status().is_success());

    let body = serde_json::from_slice::<Value>(response.bytes().await.unwrap().as_ref()).unwrap();
    assert_eq!(body, json!({"status": "UP", "checks": []}));

    for path in ["health", "health/started", "health/ready"] {
        let response = client.get(url.join(path)).send().await.unwrap();
        assert!(response.status().is_success());

        let body = serde_json::from_slice::<Value>(response.bytes().await.unwrap().as_ref()).unwrap();
        assert_eq!(
            body,
            json!({
                "status": "UP",
                "checks": [{"name": "db", "status": "UP" }, {"name": "hsm", "status": "UP" }]
            })
        );
    }
}

#[tokio::test]
async fn test_wallet_provider_metrics() {
    let (client, url) = setup_wallet_provider().await;

    let response = client.get(url.join("metrics")).send().await.unwrap();
    assert!(response.status().is_success());

    let body: String = response.text().await.unwrap();
    assert!(body.contains("nlwallet_metrics_endpoint_count"));
}
