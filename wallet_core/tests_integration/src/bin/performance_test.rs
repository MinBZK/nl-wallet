use ctor::ctor;
use indexmap::IndexMap;
use reqwest::StatusCode;
use tracing::instrument;
use url::Url;
use uuid::Uuid;

use nl_wallet_mdoc::ItemsRequest;
use openid4vc::disclosure_session::DisclosureSession;
use openid4vc::disclosure_session::HttpVpMessageClient;
use openid4vc::issuance_session::HttpIssuanceSession;
use openid4vc::verifier::SessionType;
use openid4vc::verifier::StatusResponse;
use tests_integration::fake_digid::fake_digid_auth;
use tests_integration::logging::init_logging;
use wallet::mock::default_configuration;
use wallet::mock::MockStorage;
use wallet::wallet_deps::ConfigServerConfiguration;
use wallet::wallet_deps::ConfigurationRepository;
use wallet::wallet_deps::HttpAccountProviderClient;
use wallet::wallet_deps::HttpConfigurationRepository;
use wallet::wallet_deps::HttpDigidSession;
use wallet::wallet_deps::UpdateableConfigurationRepository;
use wallet::DisclosureUriSource;
use wallet::Wallet;
use wallet_common::keys::software::SoftwareEcdsaKey;
use wallet_server::verifier::StartDisclosureRequest;
use wallet_server::verifier::StartDisclosureResponse;
use wallet_server::verifier::StatusParams;

#[ctor]
fn init() {
    init_logging();
}

#[instrument(name = "", fields(pid = std::process::id()))]
#[tokio::main]
async fn main() {
    let temp_path = tempfile::tempdir().unwrap();

    let relying_party_url = option_env!("RELYING_PARTY_URL").unwrap_or("http://localhost:3004/");
    let internal_wallet_server_url = option_env!("INTERNAL_WALLET_SERVER_URL").unwrap_or("http://localhost:3006/");
    let public_wallet_server_url = option_env!("PUBLIC_WALLET_SERVER_URL").unwrap_or("http://localhost:3005/");

    let config_server_config = ConfigServerConfiguration::default();
    let wallet_config = default_configuration();

    let config_repository = HttpConfigurationRepository::new(
        config_server_config.base_url,
        config_server_config.trust_anchors,
        (&config_server_config.signing_public_key).into(),
        temp_path.into_path(),
        wallet_config,
    )
    .await
    .unwrap();
    config_repository.fetch().await.unwrap();
    let pid_issuance_config = &config_repository.config().pid_issuance;

    let mut wallet: Wallet<
        HttpConfigurationRepository,
        MockStorage,
        SoftwareEcdsaKey,
        HttpAccountProviderClient,
        HttpDigidSession,
        HttpIssuanceSession,
        DisclosureSession<HttpVpMessageClient, Uuid>,
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

    let redirect_url = fake_digid_auth(
        &authorization_url,
        &pid_issuance_config.digid_url,
        pid_issuance_config.digid_trust_anchors(),
        "999991772",
    )
    .await;

    let _unsigned_mdocs = wallet
        .continue_pid_issuance(redirect_url)
        .await
        .expect("Could not continue pid issuance");

    wallet
        .accept_pid_issuance(pin.clone())
        .await
        .expect("Could not accept pid issuance");

    let client = reqwest::Client::new();

    let start_request = StartDisclosureRequest {
        usecase: "xyz_bank".to_owned(),
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

    let internal_mrp_url: Url = internal_wallet_server_url.parse().unwrap();
    let public_mrp_url: Url = public_wallet_server_url.parse().unwrap();

    let response = client
        .post(
            internal_mrp_url
                .join("/disclosure/sessions")
                .expect("could not join url with endpoint"),
        )
        .json(&start_request)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let StartDisclosureResponse { session_token } = response.json::<StartDisclosureResponse>().await.unwrap();

    let mut status_url = public_mrp_url
        .join(&format!("disclosure/sessions/{session_token}"))
        .unwrap();
    let status_query = serde_urlencoded::to_string(StatusParams {
        session_type: SessionType::SameDevice,
    })
    .unwrap();
    status_url.set_query(status_query.as_str().into());

    // obtain engagement_url
    let response = client.get(status_url).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let status = response.json::<StatusResponse>().await.unwrap();
    let ul = match status {
        StatusResponse::Created { ul: Some(ul), .. } => ul,
        _ => panic!("should match StatusResponse::Created"),
    };

    let proposal = wallet
        .start_disclosure(&ul.into_inner(), DisclosureUriSource::Link)
        .await
        .expect("Could not start disclosure");
    assert_eq!(proposal.documents.len(), 1);

    let return_url = wallet
        .accept_disclosure(pin)
        .await
        .expect("Could not accept disclosure");

    assert!(return_url.unwrap().to_string().starts_with(relying_party_url));
}
