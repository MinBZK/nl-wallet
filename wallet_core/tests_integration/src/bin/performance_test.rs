use ctor::ctor;
use reqwest::StatusCode;
use tracing::instrument;
use url::Url;

use dcql::CredentialQueryFormat;
use dcql::normalized::AttributeRequest;
use dcql::normalized::NormalizedCredentialRequest;
use http_utils::reqwest::default_reqwest_client_builder;
use http_utils::tls::pinning::TlsPinningConfig;
use openid4vc::disclosure_session::VpDisclosureClient;
use openid4vc::issuance_session::HttpIssuanceSession;
use openid4vc::verifier::SessionType;
use openid4vc::verifier::StatusResponse;
use openid4vc_server::verifier::StartDisclosureRequest;
use openid4vc_server::verifier::StartDisclosureResponse;
use openid4vc_server::verifier::StatusParams;
use platform_support::attested_key::mock::MockHardwareAttestedKeyHolder;
use tests_integration::default;
use tests_integration::fake_digid::fake_digid_auth;
use tests_integration::logging::init_logging;
use wallet::DisclosureUriSource;
use wallet::Wallet;
use wallet::mock::StorageStub;
use wallet::wallet_deps::HttpAccountProviderClient;
use wallet::wallet_deps::HttpConfigurationRepository;
use wallet::wallet_deps::HttpDigidSession;
use wallet::wallet_deps::Repository;
use wallet::wallet_deps::UpdatePolicyRepository;
use wallet::wallet_deps::UpdateableRepository;
use wallet::wallet_deps::WpWteIssuanceClient;
use wallet::wallet_deps::default_config_server_config;
use wallet::wallet_deps::default_wallet_config;

#[ctor]
fn init() {
    init_logging();
}

type PerformanceTestWallet = Wallet<
    HttpConfigurationRepository<TlsPinningConfig>,
    UpdatePolicyRepository,
    StorageStub,
    MockHardwareAttestedKeyHolder,
    HttpAccountProviderClient,
    HttpDigidSession,
    HttpIssuanceSession,
    VpDisclosureClient,
    WpWteIssuanceClient,
>;

#[instrument(name = "", fields(pid = std::process::id()))]
#[tokio::main]
async fn main() {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    let relying_party_url = option_env!("RELYING_PARTY_URL").unwrap_or("http://localhost:3004/");
    let public_verification_server_url =
        option_env!("PUBLIC_VERIFICATION_SERVER_URL").unwrap_or("http://localhost:3005/");
    let internal_verification_server_url =
        option_env!("INTERNAL_VERIFICATION_SERVER_URL").unwrap_or("http://localhost:3006/");

    let config_server_config = default_config_server_config();
    let wallet_config = default_wallet_config();

    let config_repository = HttpConfigurationRepository::new(
        config_server_config.signing_public_key.as_inner().into(),
        temp_path.to_path_buf(),
        wallet_config,
    )
    .await
    .unwrap();
    config_repository
        .fetch(&config_server_config.http_config)
        .await
        .unwrap();
    let pid_issuance_config = &config_repository.get().pid_issuance;
    let update_policy_repository = UpdatePolicyRepository::init();

    let mut wallet: PerformanceTestWallet = Wallet::init_registration(
        config_repository,
        update_policy_repository,
        StorageStub::default(),
        MockHardwareAttestedKeyHolder::new_apple_mock(default::attestation_environment(), default::app_identifier()),
        HttpAccountProviderClient::default(),
        VpDisclosureClient::new_http(default_reqwest_client_builder()).unwrap(),
    )
    .await
    .expect("Could not create test wallet");

    let pin = "123344";

    wallet.register(pin).await.expect("Could not register wallet");

    let authorization_url = wallet
        .create_pid_issuance_auth_url()
        .await
        .expect("Could not create pid issuance auth url");

    let redirect_url = fake_digid_auth(
        authorization_url,
        pid_issuance_config.digid_http_config.clone(),
        "999991772",
    )
    .await;

    let _attestations = wallet
        .continue_pid_issuance(redirect_url)
        .await
        .expect("Could not continue pid issuance");

    wallet
        .accept_issuance(pin.to_owned())
        .await
        .expect("Could not accept pid issuance");

    let client = reqwest::Client::new();

    let start_request = StartDisclosureRequest {
        usecase: "xyz_bank".to_owned(),
        credential_requests: Some(
            vec![NormalizedCredentialRequest {
                format: CredentialQueryFormat::MsoMdoc {
                    doctype_value: "urn:eudi:pid:nl:1".to_string(),
                },
                claims: vec![
                    AttributeRequest::new_with_keys(
                        vec!["urn:eudi:pid:nl:1".to_string(), "given_name".to_string()],
                        true,
                    ),
                    AttributeRequest::new_with_keys(
                        vec!["urn:eudi:pid:nl:1".to_string(), "family_name".to_string()],
                        false,
                    ),
                ],
            }]
            .try_into()
            .unwrap(),
        ),
        return_url_template: Some(relying_party_url.parse().unwrap()),
    };

    let internal_demo_rp_url: Url = internal_verification_server_url.parse().unwrap();
    let public_demo_rp_url: Url = public_verification_server_url.parse().unwrap();

    let response = client
        .post(
            internal_demo_rp_url
                .join("/disclosure/sessions")
                .expect("could not join url with endpoint"),
        )
        .json(&start_request)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let StartDisclosureResponse { session_token } = response.json::<StartDisclosureResponse>().await.unwrap();

    let mut status_url = public_demo_rp_url
        .join(&format!("disclosure/sessions/{session_token}"))
        .unwrap();
    let status_query = serde_urlencoded::to_string(StatusParams {
        session_type: Some(SessionType::SameDevice),
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
    assert_eq!(proposal.attestations.len(), 1);

    let return_url = wallet
        .accept_disclosure(pin.to_owned())
        .await
        .expect("Could not accept disclosure");

    assert!(return_url.unwrap().to_string().starts_with(relying_party_url));

    // Explicit drop to ensure temp dir is not moved earlier
    drop(temp_dir)
}
