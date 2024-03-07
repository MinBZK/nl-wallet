use std::env;

use indexmap::IndexMap;
use reqwest::StatusCode;
use tokio::fs;
use url::Url;
use uuid::Uuid;

use nl_wallet_mdoc::{
    holder::{CborHttpClient, DisclosureSession},
    verifier::SessionType,
    ItemsRequest,
};
use openid4vc::{issuance_session::HttpIssuanceSession, oidc::HttpOidcClient};
use platform_support::utils::{software::SoftwareUtilities, PlatformUtilities};
use wallet::{
    mock::{default_configuration, MockStorage},
    wallet_deps::{
        ConfigServerConfiguration, ConfigurationRepository, HttpAccountProviderClient, HttpConfigurationRepository,
        UpdateableConfigurationRepository,
    },
    Wallet,
};
use wallet_common::keys::software::SoftwareEcdsaKey;
use wallet_server::verifier::{StartDisclosureRequest, StartDisclosureResponse};

use crate::common::fake_digid_auth;

pub mod common;

/// This test is meant to be run against an already running (external) environment. It uses the wallet default
/// configuration determined by `wallet/.env` (or by using environment variables).
///
/// In addition, it requires an `test_integration/.env` file containing two keys:
/// - PERF_RELYING_PART_URL; The external URL from where the disclosure flow is started. Is used as return_url.
/// - PERF_WALLET_SERVER_REQUESTER_URL; The internal URL where the disclosure session is started. Normally used by the relying party server.
///
#[cfg_attr(not(feature = "performance_test"), ignore)]
#[tokio::test]
async fn test_complete_flow() {
    let storage_path = SoftwareUtilities::storage_path().await.unwrap();
    let etag_file = storage_path.join("latest-configuration-etag.txt");
    // make sure there are no storage files from previous test runs
    let _ = fs::remove_file(etag_file.as_path()).await;

    dotenvy::dotenv().unwrap();
    let relying_party_url = env::var("PERF_RELYING_PART_URL").expect("should set PERF_RELYING_PART_URL env var");
    let wallet_server_requester_url =
        env::var("PERF_WALLET_SERVER_REQUESTER_URL").expect("should set PERF_WALLET_SERVER_REQUESTER_URL env var");

    let config_server_config = ConfigServerConfiguration::default();
    let wallet_config = default_configuration();

    let config_repository = HttpConfigurationRepository::new(
        config_server_config.base_url,
        config_server_config.trust_anchors,
        (&config_server_config.signing_public_key).into(),
        storage_path,
        wallet_config,
    )
    .await
    .unwrap();
    config_repository.fetch().await.unwrap();
    let digid_base_url = &config_repository.config().pid_issuance.digid_url;

    let mut wallet: Wallet<
        HttpConfigurationRepository,
        MockStorage,
        SoftwareEcdsaKey,
        HttpAccountProviderClient,
        HttpOidcClient,
        HttpIssuanceSession,
        DisclosureSession<CborHttpClient, Uuid>,
    > = Wallet::init_registration(
        config_repository,
        MockStorage::default(),
        HttpAccountProviderClient::default(),
    )
    .await
    .expect("Could not create test wallet");

    let pin = String::from("123344");

    wallet.register(pin.clone()).await.expect("Could not register wallet");

    let authorization_url = wallet
        .create_pid_issuance_auth_url()
        .await
        .expect("Could not create pid issuance auth url");

    let redirect_url = fake_digid_auth(&authorization_url, digid_base_url).await;

    let _unsigned_mdocs = wallet
        .continue_pid_issuance(&redirect_url)
        .await
        .expect("Could not continue pid issuance");
    wallet
        .accept_pid_issuance(pin.clone())
        .await
        .expect("Could not accept pid issuance");

    let client = reqwest::Client::new();

    let start_request = StartDisclosureRequest {
        usecase: "xyz_bank".to_owned(),
        session_type: SessionType::SameDevice,
        items_requests: vec![ItemsRequest {
            doc_type: "com.example.pid".to_owned(),
            request_info: None,
            name_spaces: IndexMap::from([(
                "com.example.pid".to_owned(),
                IndexMap::from_iter(
                    [("given_name", true), ("family_name", false)]
                        .iter()
                        .map(|(name, intent_to_retain)| (name.to_string(), *intent_to_retain)),
                ),
            )]),
        }]
        .into(),
        return_url_template: Some(relying_party_url.parse().unwrap()),
    };

    let mrp_url: Url = wallet_server_requester_url.parse().unwrap();

    let response = client
        .post(
            mrp_url
                .join("/disclosure/sessions")
                .expect("could not join url with endpoint"),
        )
        .json(&start_request)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let StartDisclosureResponse {
        session_url: _session_url,
        engagement_url,
        disclosed_attributes_url: _disclosed_attributes_url,
    } = response.json::<StartDisclosureResponse>().await.unwrap();

    let proposal = wallet
        .start_disclosure(&engagement_url)
        .await
        .expect("Could not start disclosure");
    assert_eq!(proposal.documents.len(), 1);

    let return_url = wallet
        .accept_disclosure(pin)
        .await
        .expect("Could not accept disclosure");

    assert!(return_url.unwrap().to_string().starts_with(&relying_party_url));
}
