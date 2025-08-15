use std::collections::HashMap;
use std::future;
use std::future::Future;
use std::net::IpAddr;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;

use assert_matches::assert_matches;
use chrono::DateTime;
use chrono::Utc;
use http::StatusCode;
use itertools::Itertools;
use parking_lot::RwLock;
use reqwest::Client;
use reqwest::Response;
use rstest::rstest;
use rustls_pki_types::TrustAnchor;
use tokio::net::TcpListener;
use tokio::time;
use url::Url;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::auth::reader_auth::ReaderRegistration;
use attestation_data::credential_payload::mock::pid_example_payload;
use attestation_data::disclosure::DisclosedAttestation;
use attestation_data::x509::generate::mock::generate_issuer_mock;
use attestation_data::x509::generate::mock::generate_reader_mock;
use crypto::mock_remote::MockRemoteEcdsaKey;
use crypto::server_keys::generate::Ca;
use dcql::Query;
use hsm::service::Pkcs11Hsm;
use http_utils::error::HttpJsonErrorBody;
use http_utils::reqwest::default_reqwest_client_builder;
use http_utils::urls::BaseUrl;
use mdoc::examples::EXAMPLE_ATTR_NAME;
use mdoc::holder::Mdoc;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::disclosure_session::DisclosureSession;
use openid4vc::disclosure_session::DisclosureUriSource;
use openid4vc::disclosure_session::VpDisclosureClient;
use openid4vc::mock::MOCK_WALLET_CLIENT_ID;
use openid4vc::server_state::CLEANUP_INTERVAL_SECONDS;
use openid4vc::server_state::MemorySessionStore;
use openid4vc::server_state::SessionStore;
use openid4vc::server_state::SessionStoreTimeouts;
use openid4vc::server_state::SessionToken;
use openid4vc::verifier::DisclosureData;
use openid4vc::verifier::SessionType;
use openid4vc::verifier::SessionTypeReturnUrl;
use openid4vc::verifier::StatusResponse;
use openid4vc_server::verifier::StartDisclosureRequest;
use openid4vc_server::verifier::StartDisclosureResponse;
use openid4vc_server::verifier::StatusParams;
use sd_jwt_vc_metadata::TypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use sd_jwt_vc_metadata::UncheckedTypeMetadata;
use server_utils::settings::Authentication;
use server_utils::settings::RequesterAuth;
use server_utils::settings::Server;
use server_utils::settings::Settings;
use server_utils::settings::Storage;
use utils::generator::mock::MockTimeGenerator;
use verification_server::server;
use verification_server::settings::UseCaseSettings;
use verification_server::settings::VerifierSettings;
use wscd::mock_remote::MockRemoteWscd;

const PID_ATTESTATION_TYPE: &str = "urn:eudi:pid:nl:1";
const USECASE_NAME: &str = "usecase";

static EXAMPLE_START_DISCLOSURE_REQUEST: LazyLock<StartDisclosureRequest> = LazyLock::new(|| StartDisclosureRequest {
    usecase: USECASE_NAME.to_string(),
    dcql_query: Some(Query::new_example()),
    return_url_template: Some("https://return.url/{session_token}".parse().unwrap()),
});

static EXAMPLE_PID_START_DISCLOSURE_REQUEST: LazyLock<StartDisclosureRequest> =
    LazyLock::new(|| StartDisclosureRequest {
        usecase: USECASE_NAME.to_string(),
        dcql_query: Some(Query::pid_family_name()),
        return_url_template: Some("https://return.url/{session_token}".parse().unwrap()),
    });

fn memory_storage_settings() -> Storage {
    // Set up the default storage timeouts.
    let default_store_timeouts = SessionStoreTimeouts::default();

    Storage {
        url: "memory://".parse().unwrap(),
        expiration_minutes: (default_store_timeouts.expiration.as_secs() / 60).try_into().unwrap(),
        successful_deletion_minutes: (default_store_timeouts.successful_deletion.as_secs() / 60)
            .try_into()
            .unwrap(),
        failed_deletion_minutes: (default_store_timeouts.failed_deletion.as_secs() / 60)
            .try_into()
            .unwrap(),
    }
}

async fn request_server_settings_and_listener() -> (RequesterAuth, Option<TcpListener>) {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    (
        RequesterAuth::InternalEndpoint(Server {
            ip: addr.ip(),
            port: addr.port(),
        }),
        Some(listener),
    )
}

async fn wallet_server_settings_and_listener(
    requester_server: RequesterAuth,
    request: &StartDisclosureRequest,
) -> (VerifierSettings, TcpListener, Ca, TrustAnchor<'static>) {
    // Set up the listener.
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = Server {
        ip: addr.ip(),
        port: addr.port(),
    };

    // Create the issuer CA and derive the trust anchors from it.
    let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
    let issuer_trust_anchors = vec![issuer_ca.as_borrowing_trust_anchor().clone()];

    // Create the RP CA, derive the trust anchor from it and generate
    // a reader registration, based on the example items request.
    let rp_ca = Ca::generate_reader_mock_ca().unwrap();
    let reader_trust_anchors = vec![rp_ca.as_borrowing_trust_anchor().clone()];
    let rp_trust_anchor = rp_ca.to_trust_anchor().to_owned();
    let reader_registration = request
        .dcql_query
        .as_ref()
        .map(ReaderRegistration::mock_from_dcql_query);

    // Set up the use case, based on RP CA and reader registration.
    let usecase_keypair = generate_reader_mock(&rp_ca, reader_registration).unwrap();
    let usecases = HashMap::from([(
        USECASE_NAME.to_string(),
        UseCaseSettings {
            session_type_return_url: SessionTypeReturnUrl::SameDevice,
            key_pair: usecase_keypair.into(),
            dcql_query: None,
            return_url_template: None,
        },
    )])
    .into();

    // Generate a complete configuration for the verifier, including
    // a section for the issuer if that feature is enabled.
    let ws_port = server.port;
    let settings = Settings {
        wallet_server: server,

        public_url: format!("http://localhost:{ws_port}/").parse().unwrap(),

        log_requests: true,
        structured_logging: false,
        storage: memory_storage_settings(),
        issuer_trust_anchors,

        hsm: None,
    };

    let settings = VerifierSettings {
        usecases,
        ephemeral_id_secret: crypto::utils::random_bytes(64).try_into().unwrap(),

        allow_origins: None,
        reader_trust_anchors,
        requester_server,

        universal_link_base_url: "http://universal.link/".parse().unwrap(),

        server_settings: settings,

        wallet_client_ids: vec![MOCK_WALLET_CLIENT_ID.to_string()],
    };

    (settings, listener, issuer_ca, rp_trust_anchor)
}

async fn start_wallet_server<S>(
    wallet_listener: TcpListener,
    requester_listener: Option<TcpListener>,
    settings: VerifierSettings,
    hsm: Option<Pkcs11Hsm>,
    disclosure_sessions: S,
) where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let public_url = settings.server_settings.public_url.clone();

    tokio::spawn(async move {
        if let Err(error) = server::serve_with_listeners(
            wallet_listener,
            requester_listener,
            settings,
            hsm,
            Arc::new(disclosure_sessions),
        )
        .await
        {
            println!("Could not start wallet_server: {error:?}");

            process::exit(1);
        }
    });

    wait_for_server(public_url).await;
}

async fn wait_for_server(base_url: BaseUrl) {
    let client = default_reqwest_client_builder().build().unwrap();

    time::timeout(Duration::from_secs(3), async {
        let mut interval = time::interval(Duration::from_millis(10));
        loop {
            match client.get(base_url.join("health")).send().await {
                Ok(_) => break,
                Err(error) => {
                    println!("Server not yet up: {error}");
                    interval.tick().await;
                }
            }
        }
    })
    .await
    .unwrap();
}

fn internal_url(settings: &VerifierSettings) -> BaseUrl {
    match settings.requester_server {
        RequesterAuth::ProtectedInternalEndpoint {
            server: Server { port, .. },
            ..
        }
        | RequesterAuth::InternalEndpoint(Server { port, .. }) => format!("http://localhost:{port}/").parse().unwrap(),
        RequesterAuth::Authentication(_) => settings.server_settings.public_url.clone(),
    }
}

#[rstest]
#[case(RequesterAuth::Authentication(Authentication::ApiKey(String::from("secret_key"))))]
#[case(RequesterAuth::ProtectedInternalEndpoint {
    authentication: Authentication::ApiKey(String::from("secret_key")),
    server: Server {
        ip: IpAddr::from_str("127.0.0.1").unwrap(),
        port: 0,
    }
})]
#[case(RequesterAuth::InternalEndpoint(Server {
    ip: IpAddr::from_str("127.0.0.1").unwrap(),
    port: 0,
}))]
#[tokio::test]
async fn test_requester_authentication(#[case] mut auth: RequesterAuth) {
    let requester_listener = match &mut auth {
        RequesterAuth::Authentication(_) => None,
        RequesterAuth::ProtectedInternalEndpoint { server, .. } | RequesterAuth::InternalEndpoint(server) => {
            let listener = TcpListener::bind(("localhost", 0)).await.unwrap();
            server.port = listener.local_addr().unwrap().port();
            Some(listener)
        }
    };

    let (settings, wallet_listener, _, _) =
        wallet_server_settings_and_listener(auth, &EXAMPLE_START_DISCLOSURE_REQUEST).await;
    let hsm = settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()
        .unwrap();
    let auth = &settings.requester_server;

    let internal_url = internal_url(&settings);

    start_wallet_server(
        wallet_listener,
        requester_listener,
        settings.clone(),
        hsm,
        MemorySessionStore::default(),
    )
    .await;

    let client = default_reqwest_client_builder().build().unwrap();

    // check if using no token returns a 401 on the (public) start URL if an API key is used and a 404 otherwise
    // (because it is served on the internal URL)
    let response = client
        .post(settings.server_settings.public_url.join("disclosure/sessions"))
        .json(LazyLock::force(&EXAMPLE_START_DISCLOSURE_REQUEST))
        .send()
        .await
        .unwrap();

    match auth {
        RequesterAuth::Authentication(_) => assert_eq!(response.status(), StatusCode::UNAUTHORIZED),
        _ => assert_eq!(response.status(), StatusCode::NOT_FOUND),
    };

    // check if using no token returns a 401 on the (internal) start URL if an API key is used and a 200 otherwise
    let response = client
        .post(internal_url.join("disclosure/sessions"))
        .json(LazyLock::force(&EXAMPLE_START_DISCLOSURE_REQUEST))
        .send()
        .await
        .unwrap();

    match auth {
        RequesterAuth::InternalEndpoint(_) => assert_eq!(response.status(), StatusCode::OK),
        _ => assert_eq!(response.status(), StatusCode::UNAUTHORIZED),
    };

    // check if using a token returns a 200 on the (public) start URL if an API key is used and a 404 otherwise (because
    // it is served on the internal URL)
    let response = client
        .post(settings.server_settings.public_url.join("disclosure/sessions"))
        .header("Authorization", "Bearer secret_key")
        .json(LazyLock::force(&EXAMPLE_START_DISCLOSURE_REQUEST))
        .send()
        .await
        .unwrap();

    match auth {
        RequesterAuth::Authentication(_) => assert_eq!(response.status(), StatusCode::OK),
        _ => assert_eq!(response.status(), StatusCode::NOT_FOUND),
    };

    // check if using a token returns a 200 on the (internal) start URL (even if none is required)
    let response = client
        .post(internal_url.join("disclosure/sessions"))
        .header("Authorization", "Bearer secret_key")
        .json(LazyLock::force(&EXAMPLE_START_DISCLOSURE_REQUEST))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let session_token = response.json::<StartDisclosureResponse>().await.unwrap().session_token;
    let public_disclosed_attributes_url = settings
        .server_settings
        .public_url
        .join(&format!("disclosure/sessions/{session_token}/disclosed_attributes"));
    let internal_disclosed_attributes_url =
        internal_url.join(&format!("disclosure/sessions/{session_token}/disclosed_attributes"));

    // check if using no token returns a 401 on the (public) attributes URL if an API key is used and a 404 otherwise
    // (because it is served on the internal URL)
    let response = client
        .get(public_disclosed_attributes_url.clone())
        .json(LazyLock::force(&EXAMPLE_START_DISCLOSURE_REQUEST))
        .send()
        .await
        .unwrap();

    match auth {
        RequesterAuth::Authentication(_) => assert_eq!(response.status(), StatusCode::UNAUTHORIZED),
        _ => assert_eq!(response.status(), StatusCode::NOT_FOUND),
    };

    // check if using no token returns a 401 on the (internal) attributes URL if an API key is used and a 400 otherwise
    // (because the session is not yet finished)
    let response = client
        .get(internal_disclosed_attributes_url.clone())
        .json(LazyLock::force(&EXAMPLE_START_DISCLOSURE_REQUEST))
        .send()
        .await
        .unwrap();

    match auth {
        RequesterAuth::InternalEndpoint(_) => assert_eq!(response.status(), StatusCode::BAD_REQUEST),
        _ => assert_eq!(response.status(), StatusCode::UNAUTHORIZED),
    };

    // check if using a token returns a 400 on the (public) attributes URL if an API key is used and a 404 otherwise
    // (because it is served on the internal URL)
    let response = client
        .get(public_disclosed_attributes_url)
        .header("Authorization", "Bearer secret_key")
        .json(LazyLock::force(&EXAMPLE_START_DISCLOSURE_REQUEST))
        .send()
        .await
        .unwrap();

    match auth {
        RequesterAuth::Authentication(_) => assert_eq!(response.status(), StatusCode::BAD_REQUEST),
        _ => assert_eq!(response.status(), StatusCode::NOT_FOUND),
    };

    // check if using a token returns a 400 on the (internal) attributes URL (because the session is not yet finished)
    let response = client
        .get(internal_disclosed_attributes_url)
        .header("Authorization", "Bearer secret_key")
        .json(LazyLock::force(&EXAMPLE_START_DISCLOSURE_REQUEST))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

async fn test_http_json_error_body(
    response: Response,
    status_code: StatusCode,
    error_type: &str,
) -> HttpJsonErrorBody<String> {
    assert_eq!(response.status(), status_code);

    let body = serde_json::from_slice::<HttpJsonErrorBody<String>>(&response.bytes().await.unwrap())
        .expect("response body should deserialize to HttpJsonErrorBody");

    assert_eq!(body.r#type, error_type);
    assert_eq!(body.status, Some(status_code));

    body
}

#[tokio::test]
async fn test_new_session_parameters_error() {
    let (requester_server, requester_listener) = request_server_settings_and_listener().await;
    let (settings, wallet_listener, _, _) =
        wallet_server_settings_and_listener(requester_server, &EXAMPLE_START_DISCLOSURE_REQUEST).await;
    let hsm = settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()
        .unwrap();

    let internal_url = internal_url(&settings);
    start_wallet_server(
        wallet_listener,
        requester_listener,
        settings,
        hsm,
        MemorySessionStore::default(),
    )
    .await;
    let client = default_reqwest_client_builder().build().unwrap();

    let bad_use_case_request = {
        let mut request = EXAMPLE_START_DISCLOSURE_REQUEST.clone();
        request.usecase = "bad".to_string();
        request
    };

    let bad_return_url_request = {
        let mut request = EXAMPLE_START_DISCLOSURE_REQUEST.clone();
        request.return_url_template = None;
        request
    };

    for request in [bad_use_case_request, bad_return_url_request] {
        let response = client
            .post(internal_url.join("disclosure/sessions"))
            .json(&request)
            .send()
            .await
            .unwrap();

        test_http_json_error_body(response, StatusCode::BAD_REQUEST, "invalid_request").await;
    }
}

#[tokio::test]
async fn test_disclosure_not_found() {
    let (requester_server, requester_listener) = request_server_settings_and_listener().await;
    let (settings, wallet_listener, _, _) =
        wallet_server_settings_and_listener(requester_server, &EXAMPLE_START_DISCLOSURE_REQUEST).await;
    let hsm = settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()
        .unwrap();

    let internal_url = internal_url(&settings);
    start_wallet_server(
        wallet_listener,
        requester_listener,
        settings.clone(),
        hsm,
        MemorySessionStore::default(),
    )
    .await;

    let client = default_reqwest_client_builder().build().unwrap();

    // check if a non-existent token returns a 404 on the status URL
    let status_url = settings
        .server_settings
        .public_url
        .join("disclosure/sessions/nonexistent_session");
    let response = client.get(status_url).send().await.unwrap();

    test_http_json_error_body(response, StatusCode::NOT_FOUND, "unknown_session").await;

    // check if a non-existent token returns a 404 on the cancel URL
    let cancel_url = settings
        .server_settings
        .public_url
        .join("disclosure/sessions/nonexistent_session");
    let response = client.delete(cancel_url).send().await.unwrap();

    test_http_json_error_body(response, StatusCode::NOT_FOUND, "unknown_session").await;

    // check if a non-existent token returns a 404 on the disclosed_attributes URL
    let response = client
        .get(internal_url.join("disclosure/sessions/nonexistent_session/disclosed_attributes"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    test_http_json_error_body(response, StatusCode::NOT_FOUND, "unknown_session").await;

    // As to `request_uri` which is invoked by the wallet: this endpoint can only be invoked with a valid
    // ephemeral ID token over the session token, which the server normally hands out at the status endpoint.
    // But the server will not hand out such a token for a session token that does not refer to an existing session.
    // So getting a 404 from this endpoint is not possible, i.e. will not be happening in production scenarios,
    // so there is no need to test this case here.
}

fn format_status_url(public_url: &BaseUrl, session_token: &SessionToken, session_type: Option<SessionType>) -> Url {
    let mut status_url = public_url.join(&format!("disclosure/sessions/{session_token}"));

    if let Some(session_type) = session_type {
        let status_query = serde_urlencoded::to_string(StatusParams {
            session_type: Some(session_type),
        })
        .unwrap();
        status_url.set_query(status_query.as_str().into());
    }

    status_url
}

async fn get_status_ok(client: &Client, status_url: Url) -> StatusResponse {
    let response = client.get(status_url.clone()).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    response.json::<StatusResponse>().await.unwrap()
}

async fn start_disclosure<S>(
    disclosure_sessions: S,
    request: &StartDisclosureRequest,
) -> (
    VerifierSettings,
    Client,
    SessionToken,
    BaseUrl,
    Ca,
    TrustAnchor<'static>,
)
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let (requester_server, requester_listener) = request_server_settings_and_listener().await;
    let (settings, wallet_listener, issuer_ca, rp_trust_anchor) =
        wallet_server_settings_and_listener(requester_server, request).await;
    let hsm = settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()
        .unwrap();

    let internal_url = internal_url(&settings);

    start_wallet_server(
        wallet_listener,
        requester_listener,
        settings.clone(),
        hsm,
        disclosure_sessions,
    )
    .await;

    // Create a new disclosure session, which should return 200.
    let client = default_reqwest_client_builder().build().unwrap();
    let response = client
        .post(internal_url.join("disclosure/sessions"))
        .json(request)
        .send()
        .await
        .unwrap();

    let disclosure_response = response.json::<StartDisclosureResponse>().await.unwrap();

    (
        settings,
        client,
        disclosure_response.session_token,
        internal_url,
        issuer_ca,
        rp_trust_anchor,
    )
}

#[tokio::test]
async fn test_disclosure_missing_session_type() {
    let (settings, client, session_token, _, _, _) =
        start_disclosure(MemorySessionStore::default(), &EXAMPLE_START_DISCLOSURE_REQUEST).await;

    // Check if requesting the session status without a session_type returns a 200, but without the universal link.
    let status_url = format_status_url(&settings.server_settings.public_url, &session_token, None);

    assert_matches!(
        get_status_ok(&client, status_url).await,
        StatusResponse::Created { ul: None }
    );
}

#[tokio::test]
async fn test_disclosure_cancel() {
    let (settings, client, session_token, internal_url, _, _) =
        start_disclosure(MemorySessionStore::default(), &EXAMPLE_START_DISCLOSURE_REQUEST).await;

    // Fetching the status should return OK and be in the Created state.
    let status_url = format_status_url(
        &settings.server_settings.public_url,
        &session_token,
        Some(SessionType::SameDevice),
    );

    assert_matches!(
        get_status_ok(&client, status_url.clone()).await,
        StatusResponse::Created { ul: Some(_) }
    );

    // Cancel the newly created session, which should return 204 and no body.
    let cancel_url = settings
        .server_settings
        .public_url
        .join(&format!("disclosure/sessions/{session_token}"));
    let response = client.delete(cancel_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(response.content_length(), Some(0));

    // Fetching the status should return OK and be in the Cancelled state.
    assert_matches!(get_status_ok(&client, status_url).await, StatusResponse::Cancelled);

    // Cancelling the session again should return a 400.
    let cancel_url = settings
        .server_settings
        .public_url
        .join(&format!("disclosure/sessions/{session_token}"));
    let response = client.delete(cancel_url).send().await.unwrap();

    test_http_json_error_body(response, StatusCode::BAD_REQUEST, "session_state").await;

    // The disclosed attributes endpoint should also return a 400 and the response
    // body should include information about the session being cancelled.
    let disclosed_attributes_url = internal_url.join(&format!(
        "disclosure/sessions/{}/disclosed_attributes",
        session_token.as_ref()
    ));

    let response = client.get(disclosed_attributes_url).send().await.unwrap();

    let error_body = test_http_json_error_body(response, StatusCode::BAD_REQUEST, "session_state").await;
    itertools::assert_equal(error_body.extra.keys(), ["session_status"]);
    assert_eq!(
        error_body.extra.get("session_status").unwrap(),
        &serde_json::Value::from("CANCELLED")
    );
}

async fn test_disclosure_expired<S, F, Fut>(
    storage: Storage,
    session_store: S,
    mock_time: &RwLock<DateTime<Utc>>,
    create_cleanup_awaiter: F,
) where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
    F: Fn() -> Fut,
    Fut: Future<Output = ()>,
{
    let (settings, client, session_token, internal_url, _, _) =
        start_disclosure(session_store, &EXAMPLE_START_DISCLOSURE_REQUEST).await;
    let timeouts = SessionStoreTimeouts::from(&storage);

    // Fetch the status, this should return OK and be in the Created state.
    let status_url = format_status_url(
        &settings.server_settings.public_url,
        &session_token,
        Some(SessionType::SameDevice),
    );
    assert_matches!(
        get_status_ok(&client, status_url.clone()).await,
        StatusResponse::Created { ul: Some(_) }
    );

    // Fetching the disclosed attributes should return 400, since the session is not finished.
    let disclosed_attributes_url =
        internal_url.join(&format!("disclosure/sessions/{session_token}/disclosed_attributes"));
    let response = client.get(disclosed_attributes_url.clone()).send().await.unwrap();

    test_http_json_error_body(response, StatusCode::BAD_REQUEST, "session_state").await;

    // Advance the clock just enough so that session expiry will have occurred.
    let expiry_time = Utc::now() + timeouts.expiration;
    *mock_time.write() = expiry_time;

    time::pause();
    let cleanup_awaiter = create_cleanup_awaiter();
    time::advance(CLEANUP_INTERVAL_SECONDS + Duration::from_millis(1)).await;
    time::resume();

    // Wait for the database to have run the cleanup.
    time::timeout(Duration::from_secs(10), cleanup_awaiter)
        .await
        .expect("timeout waiting for database cleanup");

    // Fetching the status should return OK and be in the Expired state.
    assert_matches!(
        get_status_ok(&client, status_url.clone()).await,
        StatusResponse::Expired
    );

    // Fetching the disclosed attributes should still return 400, since the session did not succeed.
    let response = client.get(disclosed_attributes_url.clone()).send().await.unwrap();

    test_http_json_error_body(response, StatusCode::BAD_REQUEST, "session_state").await;

    // Advance the clock again so that the expired session will be purged.
    *mock_time.write() = expiry_time + timeouts.failed_deletion + Duration::from_millis(1);

    time::pause();
    let cleanup_awaiter = create_cleanup_awaiter();
    time::advance(CLEANUP_INTERVAL_SECONDS + Duration::from_millis(1)).await;
    time::resume();

    // Wait for the database to have run the cleanup.
    time::timeout(Duration::from_secs(10), cleanup_awaiter)
        .await
        .expect("timeout waiting for database cleanup");

    // Both the status and disclosed attribute requests should now return 404.
    let response = client.get(status_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let response = client.get(disclosed_attributes_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_disclosure_expired_memory() {
    let storage = memory_storage_settings();

    let timeouts = SessionStoreTimeouts::from(&storage);
    let time_generator = MockTimeGenerator::default();
    let mock_time = Arc::clone(&time_generator.time);
    let session_store = MemorySessionStore::new_with_time(timeouts, time_generator);

    // The `MemorySessionStore` performs cleanup instantly, so we simply pass
    // an already completed future as the cleanup awaiter in the closure.
    test_disclosure_expired(storage, session_store, mock_time.as_ref(), || future::ready(())).await;
}

#[cfg(feature = "db_test")]
mod db_test {
    use std::future::Future;
    use std::mem;
    use std::sync::Arc;

    use futures::FutureExt;
    use parking_lot::Mutex;
    use tokio::sync::oneshot;

    use openid4vc::server_state::SessionState;
    use openid4vc::server_state::SessionStore;
    use openid4vc::server_state::SessionStoreError;
    use openid4vc::server_state::SessionStoreTimeouts;
    use openid4vc::server_state::SessionToken;
    use openid4vc::verifier::DisclosureData;
    use server_utils::settings::ServerSettings;
    use server_utils::store::postgres;
    use server_utils::store::postgres::PostgresSessionStore;
    use utils::generator::mock::MockTimeGenerator;
    use verification_server::settings::VerifierSettings;

    use super::test_disclosure_expired;

    /// Helper type created along with [`PostgresSessionStoreProxy`]
    /// that can be used to register awaiters for the cleanup task.
    struct PostgresSessionStoreNotifier {
        cleanup_oneshots: Arc<Mutex<Vec<oneshot::Sender<()>>>>,
    }

    impl PostgresSessionStoreNotifier {
        // Note that this method is not async, which means it creates a new
        // oneshot channel as soon as it is called, i.e. hot instead of cold.
        fn register_cleanup_awaiter(&self) -> impl Future<Output = ()> {
            let (tx, rx) = oneshot::channel();

            self.cleanup_oneshots.lock().push(tx);

            // Ignore errors.
            rx.map(|result| result.unwrap())
        }
    }

    /// A wrapper for [`PostgresSessionStore`] that can be used to
    /// monitor execution of cleanups from another tokio task.
    struct PostgresSessionStoreProxy {
        session_store: PostgresSessionStore<MockTimeGenerator>,
        cleanup_oneshots: Arc<Mutex<Vec<oneshot::Sender<()>>>>,
    }

    impl PostgresSessionStoreProxy {
        /// This creates both a [`PostgresSessionStoreProxy`] and a [`PostgresSessionStoreNotifier`].
        fn new(session_store: PostgresSessionStore<MockTimeGenerator>) -> (Self, PostgresSessionStoreNotifier) {
            let cleanup_oneshots = Arc::new(Mutex::new(Vec::new()));

            let proxy = PostgresSessionStoreProxy {
                session_store,
                cleanup_oneshots: Arc::clone(&cleanup_oneshots),
            };
            let notifier = PostgresSessionStoreNotifier {
                cleanup_oneshots: Arc::clone(&cleanup_oneshots),
            };

            (proxy, notifier)
        }
    }

    impl SessionStore<DisclosureData> for PostgresSessionStoreProxy {
        async fn get(&self, token: &SessionToken) -> Result<Option<SessionState<DisclosureData>>, SessionStoreError> {
            self.session_store.get(token).await
        }

        async fn write(&self, session: SessionState<DisclosureData>, is_new: bool) -> Result<(), SessionStoreError> {
            self.session_store.write(session, is_new).await
        }

        async fn cleanup(&self) -> Result<(), SessionStoreError> {
            // Before performing cleanup, take all of the oneshots that are currently
            // waiting and replace the value of the mutex with an empty Vec.
            let cleanup_oneshots: Vec<oneshot::Sender<()>> = mem::take(&mut self.cleanup_oneshots.lock());

            <PostgresSessionStore<MockTimeGenerator> as SessionStore<DisclosureData>>::cleanup(&self.session_store)
                .await
                .inspect(|_| {
                    // Then after the cleanup, notify all of the those oneshots, which consumes them.
                    for cleanup_oneshot in cleanup_oneshots {
                        cleanup_oneshot.send(()).unwrap()
                    }
                })
        }
    }

    #[tokio::test]
    async fn test_disclosure_expired_postgres() {
        // Combine the generated settings with the storage settings from the configuration file.
        let storage = VerifierSettings::new("verification_server.toml", "verification_server")
            .unwrap()
            .server_settings
            .storage;

        assert_eq!(
            storage.url.scheme(),
            "postgres",
            "should be configured to use PostgreSQL storage"
        );

        let timeouts = SessionStoreTimeouts::from(&storage);
        let time_generator = MockTimeGenerator::default();
        let mock_time = Arc::clone(&time_generator.time);
        let session_store = PostgresSessionStore::new_with_time(
            postgres::new_connection(storage.url.clone()).await.unwrap(),
            timeouts,
            time_generator,
        );
        let (store_proxy, cleanup_notify) = PostgresSessionStoreProxy::new(session_store);

        test_disclosure_expired(storage, store_proxy, mock_time.as_ref(), || {
            cleanup_notify.register_cleanup_awaiter()
        })
        .await;
    }
}

async fn prepare_example_holder_mocks(issuer_ca: &Ca) -> (Mdoc, MockRemoteWscd) {
    let payload_preview = pid_example_payload(&MockTimeGenerator::default()).previewable_payload;

    let issuer_key_pair = generate_issuer_mock(issuer_ca, Some(IssuerRegistration::new_mock())).unwrap();

    // Generate a new private key and use that and the issuer key to sign the Mdoc.
    let mdoc_private_key_id = crypto::utils::random_string(16);
    let mdoc_private_key = MockRemoteEcdsaKey::new_random(mdoc_private_key_id.clone());
    let mdoc_public_key = mdoc_private_key.verifying_key();

    let unchecked_metadata = UncheckedTypeMetadata::pid_example();
    let type_metadata = TypeMetadata::try_new(unchecked_metadata.clone()).unwrap();
    let (_, metadata_integrity, _) = TypeMetadataDocuments::from_single_example(type_metadata);

    let mdoc = payload_preview
        .into_signed_mdoc_unverified::<MockRemoteEcdsaKey>(
            metadata_integrity,
            mdoc_private_key_id,
            mdoc_public_key,
            &issuer_key_pair,
        )
        .await
        .unwrap();

    // Place the private key in a `MockRemoteWscd` and return it, along with the mdoc.
    let wscd = MockRemoteWscd::new(vec![mdoc_private_key]);

    (mdoc, wscd)
}

async fn perform_full_disclosure(session_type: SessionType) -> (Client, SessionToken, BaseUrl, Option<BaseUrl>) {
    // Start the verification_server and create a disclosure request.
    let (settings, client, session_token, internal_url, issuer_ca, rp_trust_anchor) =
        start_disclosure(MemorySessionStore::default(), &EXAMPLE_PID_START_DISCLOSURE_REQUEST).await;

    // Fetching the status should return OK, be in the Created state and include a universal link.
    let status_url = format_status_url(&settings.server_settings.public_url, &session_token, Some(session_type));

    let StatusResponse::Created { ul: Some(ul) } = get_status_ok(&client, status_url.clone()).await else {
        panic!("session should be in CREATED state and a universal link should be provided")
    };

    // Prepare a holder with a valid example Mdoc. Use the query portion of the
    // universal link to have holder code contact the wallet_sever and start disclosure.
    // This should result in a proposal to disclosure for the holder.
    let (mdoc, wscd) = prepare_example_holder_mocks(&issuer_ca).await;

    let request_uri_query = ul.as_ref().query().unwrap().to_string();
    let uri_source = match session_type {
        SessionType::SameDevice => DisclosureUriSource::Link,
        SessionType::CrossDevice => DisclosureUriSource::QrCode,
    };
    let disclosure_client = VpDisclosureClient::new_http(default_reqwest_client_builder()).unwrap();
    let disclosure_session = disclosure_client
        .start(&request_uri_query, uri_source, &[rp_trust_anchor])
        .await
        .expect("disclosure session should start at client side");

    // The status endpoint should now respond that the session is in the WaitingForResponse state.
    assert_matches!(
        get_status_ok(&client, status_url.clone()).await,
        StatusResponse::WaitingForResponse
    );

    // Have the holder actually disclosure the example attributes to the verification_server,
    // after which the status endpoint should report that the session is Done.
    let return_url = disclosure_session
        .disclose(vec![mdoc].try_into().unwrap(), &wscd)
        .await
        .expect("should disclose attributes successfully");

    assert_matches!(get_status_ok(&client, status_url).await, StatusResponse::Done);

    (client, session_token, internal_url, return_url)
}

fn check_example_disclosed_attributes(disclosed_attributes: &[DisclosedAttestation]) {
    itertools::assert_equal(
        disclosed_attributes
            .iter()
            .map(|attestation| &attestation.attestation_type),
        [PID_ATTESTATION_TYPE],
    );
    let attributes = disclosed_attributes
        .iter()
        .find(|attestation| attestation.attestation_type == *PID_ATTESTATION_TYPE)
        .unwrap()
        .attributes
        .clone()
        .unwrap_mso_mdoc();
    itertools::assert_equal(attributes.keys(), [PID_ATTESTATION_TYPE]);
    let (first_entry_name, first_entry_value) = attributes.get(PID_ATTESTATION_TYPE).unwrap().first().unwrap();
    assert_eq!(first_entry_name, EXAMPLE_ATTR_NAME);
    assert_eq!(first_entry_value.to_string(), "De Bruijn".to_owned());
}

#[tokio::test]
async fn test_disclosed_attributes_without_nonce() {
    let (client, session_token, internal_url, _) = perform_full_disclosure(SessionType::CrossDevice).await;

    // Check if the disclosed attributes endpoint returns a 200 for the session, with the attributes.
    let disclosed_attributes_url = internal_url.join(&format!(
        "disclosure/sessions/{}/disclosed_attributes",
        session_token.as_ref()
    ));

    let response = client.get(disclosed_attributes_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check the disclosed attributes against the example attributes.
    let disclosed_attributes = response.json::<Vec<DisclosedAttestation>>().await.unwrap();

    check_example_disclosed_attributes(&disclosed_attributes);
}

#[tokio::test]
async fn test_disclosed_attributes_with_nonce() {
    let (client, session_token, internal_url, return_url) = perform_full_disclosure(SessionType::SameDevice).await;

    // Check if the disclosed attributes endpoint returns a 400 error when
    // requesting the attributes without a nonce or with an incorrect nonce.
    let mut disclosed_attributes_url = internal_url.join(&format!(
        "disclosure/sessions/{}/disclosed_attributes",
        session_token.as_ref()
    ));

    let response = client.get(disclosed_attributes_url.clone()).send().await.unwrap();

    test_http_json_error_body(response, StatusCode::UNAUTHORIZED, "nonce").await;

    let mut disclosed_attributes_url_incorrect_nonce = disclosed_attributes_url.clone();
    disclosed_attributes_url_incorrect_nonce
        .query_pairs_mut()
        .append_pair("nonce", "incorrect");
    let response = client
        .get(disclosed_attributes_url_incorrect_nonce)
        .send()
        .await
        .unwrap();

    test_http_json_error_body(response, StatusCode::UNAUTHORIZED, "nonce").await;

    // Check if the disclosed attributes endpoint returns a 200 for the session,
    // with the attributes, when we include the nonce from the return URL.
    let nonce = return_url
        .expect("a same-device disclosure session should procude a return URL")
        .into_inner()
        .query_pairs()
        .find_map(|(key, value)| (key == "nonce").then_some(value.into_owned()))
        .unwrap();
    disclosed_attributes_url.query_pairs_mut().append_pair("nonce", &nonce);

    let response = client.get(disclosed_attributes_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check the disclosed attributes against the example attributes.
    let disclosed_attributes = response.json::<Vec<DisclosedAttestation>>().await.unwrap();

    check_example_disclosed_attributes(&disclosed_attributes);
}

#[tokio::test]
async fn test_disclosed_attributes_failed_session() {
    // Start the verification_server and create a disclosure request.
    let (settings, client, session_token, internal_url, issuer_ca, rp_trust_anchor) =
        start_disclosure(MemorySessionStore::default(), &EXAMPLE_START_DISCLOSURE_REQUEST).await;

    // Fetching the status should return OK, be in the Created state and include a universal link.
    let status_url = format_status_url(
        &settings.server_settings.public_url,
        &session_token,
        Some(SessionType::CrossDevice),
    );

    let StatusResponse::Created { ul: Some(ul) } = get_status_ok(&client, status_url.clone()).await else {
        panic!("session should be in CREATED state and a universal link should be provided")
    };

    let request_uri_query = ul.as_ref().query().unwrap().to_string();
    let disclosure_client = VpDisclosureClient::new_http(default_reqwest_client_builder()).unwrap();
    let disclosure_session = disclosure_client
        .start(&request_uri_query, DisclosureUriSource::QrCode, &[rp_trust_anchor])
        .await
        .expect("disclosure session should start at client side");

    // Disclose the expired attestation from the examples in the ISO specification.
    let mdoc = Mdoc::new_example_resigned(&issuer_ca).await;
    disclosure_session
        .disclose(vec![mdoc].try_into().unwrap(), &MockRemoteWscd::new_example())
        .await
        .expect_err("disclosing attributes should result in an error");

    // Check that the disclosed attributes endpoint now returns a 400 error and that the response
    // body contains information that the session has FAILED, as well as the reason why.
    let disclosed_attributes_url = internal_url.join(&format!(
        "disclosure/sessions/{}/disclosed_attributes",
        session_token.as_ref()
    ));

    let response = client.get(disclosed_attributes_url).send().await.unwrap();

    let error_body = test_http_json_error_body(response, StatusCode::BAD_REQUEST, "session_state").await;
    itertools::assert_equal(error_body.extra.keys().sorted(), ["session_error", "session_status"]);
    assert_eq!(
        error_body.extra.get("session_status").unwrap(),
        &serde_json::Value::from("FAILED")
    );
    // Simply check for the presence of the word "expired" to avoid fully matching the error string.
    assert!(
        error_body
            .extra
            .get("session_error")
            .unwrap()
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains("expired")
    );
}
