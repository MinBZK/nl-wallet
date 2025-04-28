use std::str::FromStr;
use std::sync::Arc;

use axum::routing::post;
use axum::Json;
use axum::Router;
use serial_test::serial;
use url::Url;

use http_utils::urls::disclosure_based_issuance_base_uri;
use http_utils::urls::DEFAULT_UNIVERSAL_LINK_BASE;
use issuance_server::disclosure::mock::MockAttributesFetcher;
use issuance_server::disclosure::AttributesFetcher;
use issuance_server::disclosure::HttpAttributesFetcher;
use issuance_server::settings::IssuanceServerSettings;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::openid4vp::RequestUriMethod;
use openid4vc::openid4vp::VpRequestUriObject;
use openid4vc::verifier::VerifierUrlParameters;
use tests_integration::common::*;
use wallet::openid4vc::AttributeValue;
use wallet::openid4vc::SessionType;
use wallet::utils::BaseUrl;
use wallet::Attestation;
use wallet::AttestationAttributeValue;
use wallet::DisclosureUriSource;

#[tokio::test]
#[serial(hsm)]
async fn test_pid_ok() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // retain [`MockDigidSession::Context`]
    let _context = setup_digid_context();

    let pin = "112233";
    let mut wallet = setup_wallet_and_default_env(WalletDeviceVendor::Apple).await;
    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    // Emit attestations into this local variable
    let attestations: Arc<std::sync::Mutex<Vec<Attestation>>> = Arc::new(std::sync::Mutex::new(vec![]));
    {
        let attestations = attestations.clone();
        wallet
            .set_attestations_callback(Box::new(move |mut a| {
                let mut attestations = attestations.lock().unwrap();
                attestations.append(&mut a);
            }))
            .await
            .unwrap();
    }

    // Verify that the first mdoc contains the bsn
    let attestations = attestations.lock().unwrap();
    let pid_attestation = attestations.first().unwrap();
    let bsn_attr = pid_attestation.attributes.iter().find(|a| a.key == vec!["bsn"]);

    match bsn_attr {
        Some(bsn_attr) => assert_eq!(
            bsn_attr.value,
            AttestationAttributeValue::Basic(AttributeValue::Text("999991772".to_string()))
        ),
        None => panic!("BSN attribute not found"),
    }

    Ok(())
}

fn universal_link(issuance_server_url: &BaseUrl) -> Url {
    let params = serde_urlencoded::to_string(VerifierUrlParameters {
        session_type: SessionType::SameDevice,
        ephemeral_id_params: None,
    })
    .unwrap();

    let mut issuance_server_url = issuance_server_url
        .join_base_url("/disclosure/disclosure_based_issuance/request_uri")
        .into_inner();
    issuance_server_url.set_query(Some(&params));

    let query = serde_urlencoded::to_string(VpRequestUriObject {
        request_uri: issuance_server_url.try_into().unwrap(),
        request_uri_method: Some(RequestUriMethod::POST),
        client_id: "disclosure_based_issuance.example.com".to_string(),
    })
    .unwrap();

    let mut uri = disclosure_based_issuance_base_uri(&DEFAULT_UNIVERSAL_LINK_BASE.parse().unwrap()).into_inner();
    uri.set_query(Some(&query));

    uri
}

#[tokio::test]
#[serial(hsm)]
async fn test_disclosure_based_issuance_ok_mock_attributes_fetcher() {
    let (settings, attributes_fetcher) = issuance_server_settings();
    test_disclosure_based_issuance_ok(settings, attributes_fetcher).await
}

#[tokio::test]
#[serial(hsm)]
async fn test_disclosure_based_issuance_ok_http_attributes_fetcher() {
    let (settings, _) = issuance_server_settings();

    let url = start_attestation_server(vec![mock_issuable_document()]).await;
    let attributes_fetcher = HttpAttributesFetcher::new(
        settings
            .disclosure_settings
            .keys()
            .map(|id| (id.to_string(), url.clone()))
            .collect(),
    );

    test_disclosure_based_issuance_ok(settings, attributes_fetcher).await
}

pub async fn start_attestation_server(issuable_documents: Vec<IssuableDocument>) -> BaseUrl {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let url = BaseUrl::from_str(format!("http://localhost:{}/", port).as_str()).unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().route("/", post(|| async { Json(issuable_documents) })),
        )
        .await
        .unwrap();
    });

    wait_for_server(url.clone(), vec![]).await;

    url
}

async fn test_disclosure_based_issuance_ok(
    issuance_server_settings: IssuanceServerSettings,
    attributes_fetcher: impl AttributesFetcher + Sync + 'static,
) {
    let _context = setup_digid_context();

    let pin = "112233";
    let (mut wallet, _, issuance_server_url) = setup_wallet_and_env(
        WalletDeviceVendor::Apple,
        config_server_settings(),
        update_policy_server_settings(),
        wallet_provider_settings(),
        verification_server_settings(),
        pid_issuer_settings(),
        (issuance_server_settings, attributes_fetcher),
    )
    .await;

    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    let _proposal = wallet
        .start_disclosure(&universal_link(&issuance_server_url), DisclosureUriSource::Link)
        .await
        .unwrap();
    let _attestation_previews = wallet.continue_disclosure_based_issuance(pin.to_owned()).await.unwrap();

    wallet.accept_issuance(pin.to_owned()).await.unwrap();
}

#[tokio::test]
#[serial(hsm)]
async fn test_disclosure_based_issuance_error_no_attributes() {
    let _context = setup_digid_context();

    let pin = "112233";
    let (mut wallet, _, issuance_server_url) = setup_wallet_and_env(
        WalletDeviceVendor::Apple,
        config_server_settings(),
        update_policy_server_settings(),
        wallet_provider_settings(),
        verification_server_settings(),
        pid_issuer_settings(),
        (issuance_server_settings().0, MockAttributesFetcher(vec![])),
    )
    .await;

    wallet = do_wallet_registration(wallet, pin).await;
    wallet = do_pid_issuance(wallet, pin.to_owned()).await;

    let _proposal = wallet
        .start_disclosure(&universal_link(&issuance_server_url), DisclosureUriSource::Link)
        .await
        .unwrap();

    // If the issuer has no attestations to issue, we receive an empty vec and no error.
    let attestations = wallet.continue_disclosure_based_issuance(pin.to_owned()).await.unwrap();
    assert!(attestations.is_empty());
}
