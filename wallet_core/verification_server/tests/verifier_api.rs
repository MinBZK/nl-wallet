use std::collections::HashMap;
use std::future;
use std::future::Future;
use std::net::IpAddr;
use std::net::TcpListener;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;

use assert_matches::assert_matches;
use chrono::DateTime;
use chrono::Days;
use chrono::Utc;
use http::StatusCode;
use indexmap::IndexMap;
use itertools::Itertools;
use p256::ecdsa::SigningKey;
use parking_lot::RwLock;
use reqwest::Client;
use reqwest::Response;
use rstest::rstest;
use rustls_pki_types::TrustAnchor;
use tokio::time;
use url::Url;

use nl_wallet_mdoc::examples::Example;
use nl_wallet_mdoc::examples::EXAMPLE_ATTR_NAME;
use nl_wallet_mdoc::examples::EXAMPLE_ATTR_VALUE;
use nl_wallet_mdoc::examples::EXAMPLE_DOC_TYPE;
use nl_wallet_mdoc::examples::EXAMPLE_NAMESPACE;
use nl_wallet_mdoc::holder::mock::MockMdocDataSource;
use nl_wallet_mdoc::holder::Mdoc;
use nl_wallet_mdoc::server_keys::generate::Ca;
use nl_wallet_mdoc::server_keys::KeyPair;
use nl_wallet_mdoc::unsigned::Entry;
use nl_wallet_mdoc::unsigned::UnsignedMdoc;
use nl_wallet_mdoc::utils::issuer_auth::IssuerRegistration;
use nl_wallet_mdoc::utils::reader_auth::ReaderRegistration;
use nl_wallet_mdoc::utils::serialization::TaggedBytes;
use nl_wallet_mdoc::verifier::DisclosedAttributes;
use nl_wallet_mdoc::DeviceResponse;
use nl_wallet_mdoc::IssuerSigned;
use nl_wallet_mdoc::ItemsRequest;
use openid4vc::disclosure_session::DisclosureSession;
use openid4vc::disclosure_session::DisclosureUriSource;
use openid4vc::disclosure_session::HttpVpMessageClient;
use openid4vc::server_state::MemorySessionStore;
use openid4vc::server_state::SessionStore;
use openid4vc::server_state::SessionStoreTimeouts;
use openid4vc::server_state::SessionToken;
use openid4vc::server_state::CLEANUP_INTERVAL_SECONDS;
use openid4vc::verifier::DisclosureData;
use openid4vc::verifier::SessionType;
use openid4vc::verifier::SessionTypeReturnUrl;
use openid4vc::verifier::StatusResponse;
use openid4vc::verifier::VerifierUrlParameters;
use openid4vc::ErrorResponse;
use openid4vc_server::verifier::StartDisclosureRequest;
use openid4vc_server::verifier::StartDisclosureResponse;
use openid4vc_server::verifier::StatusParams;
use sd_jwt::metadata::TypeMetadata;
use sd_jwt::metadata::TypeMetadataChain;
use verification_server::server;
use verification_server::settings::VerifierSettings;
use verification_server::settings::VerifierUseCase;
use wallet_common::generator::mock::MockTimeGenerator;
use wallet_common::generator::TimeGenerator;
use wallet_common::http_error::HttpJsonErrorBody;
use wallet_common::keys::mock_remote::MockRemoteEcdsaKey;
use wallet_common::keys::mock_remote::MockRemoteKeyFactory;
use wallet_common::reqwest::default_reqwest_client_builder;
use wallet_common::urls::BaseUrl;
use wallet_common::utils;
use wallet_server::settings::Authentication;
use wallet_server::settings::RequesterAuth;
use wallet_server::settings::Server;
use wallet_server::settings::Settings;
use wallet_server::settings::Storage;

const USECASE_NAME: &str = "usecase";

static EXAMPLE_START_DISCLOSURE_REQUEST: LazyLock<StartDisclosureRequest> = LazyLock::new(|| StartDisclosureRequest {
    usecase: USECASE_NAME.to_string(),
    return_url_template: Some("https://return.url/{session_token}".parse().unwrap()),
    items_requests: vec![ItemsRequest {
        doc_type: EXAMPLE_DOC_TYPE.to_string(),
        request_info: None,
        name_spaces: IndexMap::from([(
            EXAMPLE_NAMESPACE.to_string(),
            IndexMap::from_iter(
                [(EXAMPLE_ATTR_NAME.to_string(), true)]
                    .into_iter()
                    .map(|(name, intent_to_retain)| (name.to_string(), intent_to_retain)),
            ),
        )]),
    }]
    .into(),
});

fn find_listener_port() -> u16 {
    TcpListener::bind("localhost:0")
        .expect("Could not find TCP port")
        .local_addr()
        .expect("Could not get local address from TCP listener")
        .port()
}

fn wallet_server_settings() -> (VerifierSettings, KeyPair<SigningKey>, TrustAnchor<'static>) {
    // Set up the hostname and ports.
    let localhost = IpAddr::from_str("127.0.0.1").unwrap();
    let ws_port = find_listener_port();
    let rp_port = find_listener_port();

    // Set up the default storage timeouts.
    let default_store_timeouts = SessionStoreTimeouts::default();

    // Create the issuer CA and derive the trust anchors from it.
    let issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
    let issuer_key_pair = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .unwrap();
    let issuer_trust_anchors = vec![issuer_ca.as_borrowing_trust_anchor().clone()];

    // Create the RP CA, derive the trust anchor from it and generate
    // a reader registration, based on the example items request.
    let rp_ca = Ca::generate_reader_mock_ca().unwrap();
    let reader_trust_anchors = vec![rp_ca.as_borrowing_trust_anchor().clone()];
    let rp_trust_anchor = rp_ca.to_trust_anchor().to_owned();
    let reader_registration = Some(ReaderRegistration::new_mock_from_requests(
        &EXAMPLE_START_DISCLOSURE_REQUEST.items_requests,
    ));

    // Set up the use case, based on RP CA and reader registration.
    let usecase_keypair = rp_ca.generate_reader_mock(reader_registration).unwrap();
    let usecases = HashMap::from([(
        USECASE_NAME.to_string(),
        VerifierUseCase {
            session_type_return_url: SessionTypeReturnUrl::SameDevice,
            key_pair: usecase_keypair.into(),
        },
    )])
    .into();

    // Generate a complete configuration for the verifier, including
    // a section for the issuer if that feature is enabled.
    let settings = Settings {
        wallet_server: Server {
            ip: localhost,
            port: ws_port,
        },

        public_url: format!("http://localhost:{ws_port}/").parse().unwrap(),

        log_requests: true,
        structured_logging: false,
        storage: Storage {
            url: "memory://".parse().unwrap(),
            expiration_minutes: (default_store_timeouts.expiration.as_secs() / 60).try_into().unwrap(),
            successful_deletion_minutes: (default_store_timeouts.successful_deletion.as_secs() / 60)
                .try_into()
                .unwrap(),
            failed_deletion_minutes: (default_store_timeouts.failed_deletion.as_secs() / 60)
                .try_into()
                .unwrap(),
        },
        issuer_trust_anchors,
    };

    let settings = VerifierSettings {
        usecases,
        ephemeral_id_secret: utils::random_bytes(64).try_into().unwrap(),

        allow_origins: None,
        reader_trust_anchors,
        requester_server: RequesterAuth::InternalEndpoint(Server {
            ip: localhost,
            port: rp_port,
        }),

        universal_link_base_url: "http://universal.link/".parse().unwrap(),

        server_settings: settings,
    };

    (settings, issuer_key_pair, rp_trust_anchor)
}

async fn start_wallet_server<S>(settings: VerifierSettings, disclosure_sessions: S)
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let public_url = settings.server_settings.public_url.clone();

    tokio::spawn(async move {
        if let Err(error) = server::serve(settings, disclosure_sessions).await {
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

fn internal_url(auth: &RequesterAuth, public_url: &BaseUrl) -> BaseUrl {
    match auth {
        RequesterAuth::ProtectedInternalEndpoint {
            server: Server { port, .. },
            ..
        }
        | RequesterAuth::InternalEndpoint(Server { port, .. }) => format!("http://localhost:{port}/").parse().unwrap(),
        RequesterAuth::Authentication(_) => public_url.clone(),
    }
}

#[rstest]
#[case(RequesterAuth::Authentication(Authentication::ApiKey(String::from("secret_key"))))]
#[case(RequesterAuth::ProtectedInternalEndpoint {
    authentication: Authentication::ApiKey(String::from("secret_key")),
    server: Server {
        ip: IpAddr::from_str("127.0.0.1").unwrap(),
        port: find_listener_port(),
    }
})]
#[case(RequesterAuth::InternalEndpoint(Server {
    ip: IpAddr::from_str("127.0.0.1").unwrap(),
    port: find_listener_port(),
}))]
#[tokio::test]
async fn test_requester_authentication(#[case] auth: RequesterAuth) {
    let (mut settings, _, _) = wallet_server_settings();
    let internal_url = internal_url(&auth, &settings.server_settings.public_url);
    settings.requester_server = auth.clone();

    start_wallet_server(settings.clone(), MemorySessionStore::default()).await;

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
        .join(&format!("disclosure/sessions/{}/disclosed_attributes", session_token));
    let internal_disclosed_attributes_url =
        internal_url.join(&format!("disclosure/sessions/{}/disclosed_attributes", session_token));

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

async fn test_error_response(response: Response, status_code: StatusCode, error_type: &str) {
    assert_eq!(response.status(), status_code);

    let body = serde_json::from_slice::<ErrorResponse<String>>(&response.bytes().await.unwrap())
        .expect("response body should deserialize to ErrorResponse");

    assert_eq!(body.error, error_type);
}

#[tokio::test]
async fn test_new_session_parameters_error() {
    let (settings, _, _) = wallet_server_settings();
    let internal_url = internal_url(&settings.requester_server, &settings.server_settings.public_url);
    start_wallet_server(settings, MemorySessionStore::default()).await;
    let client = default_reqwest_client_builder().build().unwrap();

    let bad_use_case_request = {
        let mut request = EXAMPLE_START_DISCLOSURE_REQUEST.clone();
        request.usecase = "bad".to_string();
        request
    };

    let no_items_request = {
        let mut request = EXAMPLE_START_DISCLOSURE_REQUEST.clone();
        request.items_requests = vec![].into();
        request
    };

    let bad_return_url_request = {
        let mut request = EXAMPLE_START_DISCLOSURE_REQUEST.clone();
        request.return_url_template = None;
        request
    };

    for request in [bad_use_case_request, no_items_request, bad_return_url_request] {
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
    let (settings, _, _) = wallet_server_settings();
    let internal_url = internal_url(&settings.requester_server, &settings.server_settings.public_url);
    start_wallet_server(settings.clone(), MemorySessionStore::default()).await;

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

    // check if a non-existent token returns a 404 on the wallet URL
    let mut request_uri = settings
        .server_settings
        .public_url
        .join("disclosure/sessions/nonexistent_session/request_uri");
    request_uri.set_query(
        serde_urlencoded::to_string(VerifierUrlParameters {
            session_type: SessionType::SameDevice,
            ephemeral_id: vec![42],
            time: Utc::now(),
        })
        .unwrap()
        .as_str()
        .into(),
    );
    let response = client.get(request_uri).send().await.unwrap();

    test_error_response(response, StatusCode::NOT_FOUND, "unknown_session").await;

    // check if a non-existent token returns a 404 on the disclosed_attributes URL
    let response = client
        .get(internal_url.join("disclosure/sessions/nonexistent_session/disclosed_attributes"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    test_http_json_error_body(response, StatusCode::NOT_FOUND, "unknown_session").await;
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
) -> (
    VerifierSettings,
    Client,
    SessionToken,
    BaseUrl,
    KeyPair<SigningKey>,
    TrustAnchor<'static>,
)
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let (settings, issuer_key_pair, rp_trust_anchor) = wallet_server_settings();
    let internal_url = internal_url(&settings.requester_server, &settings.server_settings.public_url);

    start_wallet_server(settings.clone(), disclosure_sessions).await;

    // Create a new disclosure session, which should return 200.
    let client = default_reqwest_client_builder().build().unwrap();
    let response = client
        .post(internal_url.join("disclosure/sessions"))
        .json(LazyLock::force(&EXAMPLE_START_DISCLOSURE_REQUEST))
        .send()
        .await
        .unwrap();

    let disclosure_response = response.json::<StartDisclosureResponse>().await.unwrap();

    (
        settings,
        client,
        disclosure_response.session_token,
        internal_url,
        issuer_key_pair,
        rp_trust_anchor,
    )
}

#[tokio::test]
async fn test_disclosure_missing_session_type() {
    let (settings, client, session_token, _, _, _) = start_disclosure(MemorySessionStore::default()).await;

    // Check if requesting the session status without a session_type returns a 200, but without the universal link.
    let status_url = format_status_url(&settings.server_settings.public_url, &session_token, None);

    assert_matches!(
        get_status_ok(&client, status_url).await,
        StatusResponse::Created { ul: None }
    );
}

#[tokio::test]
async fn test_disclosure_cancel() {
    let (settings, client, session_token, internal_url, _, _) = start_disclosure(MemorySessionStore::default()).await;

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
    settings: VerifierSettings,
    session_store: S,
    mock_time: &RwLock<DateTime<Utc>>,
    create_cleanup_awaiter: F,
) where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
    F: Fn() -> Fut,
    Fut: Future<Output = ()>,
{
    let timeouts = SessionStoreTimeouts::from(&settings.server_settings.storage);

    let (settings, client, session_token, internal_url, _, _) = start_disclosure(session_store).await;

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
        internal_url.join(&format!("disclosure/sessions/{}/disclosed_attributes", session_token));
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
    let (settings, _, _) = wallet_server_settings();

    let timeouts = SessionStoreTimeouts::from(&settings.server_settings.storage);
    let time_generator = MockTimeGenerator::default();
    let mock_time = Arc::clone(&time_generator.time);
    let session_store = MemorySessionStore::new_with_time(timeouts, time_generator);

    // The `MemorySessionStore` performs cleanup instantly, so we simply pass
    // an already completed future as the cleanup awaiter in the closure.
    test_disclosure_expired(settings, session_store, mock_time.as_ref(), || future::ready(())).await;
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
    use openid4vc_server::store::postgres;
    use openid4vc_server::store::postgres::PostgresSessionStore;
    use verification_server::settings::VerifierSettings;
    use wallet_common::generator::mock::MockTimeGenerator;
    use wallet_server::settings::ServerSettings;

    use super::test_disclosure_expired;
    use super::wallet_server_settings;

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
        let (mut settings, _, _) = wallet_server_settings();
        let storage_settings = VerifierSettings::new_custom("ws_integration_test.toml", "ws_integration_test")
            .unwrap()
            .server_settings
            .storage;
        settings.server_settings.storage = storage_settings;

        assert_eq!(
            settings.server_settings.storage.url.scheme(),
            "postgres",
            "should be configured to use PostgreSQL storage"
        );

        let timeouts = SessionStoreTimeouts::from(&settings.server_settings.storage);
        let time_generator = MockTimeGenerator::default();
        let mock_time = Arc::clone(&time_generator.time);
        let session_store = PostgresSessionStore::new_with_time(
            postgres::new_connection(settings.server_settings.storage.url.clone())
                .await
                .unwrap(),
            timeouts,
            time_generator,
        );
        let (store_proxy, cleanup_notify) = PostgresSessionStoreProxy::new(session_store);

        test_disclosure_expired(settings, store_proxy, mock_time.as_ref(), || {
            cleanup_notify.register_cleanup_awaiter()
        })
        .await;
    }
}

/// This utility function is used to prepare the two mocks necessary to emulate a valid holder.
/// It will populate a [`MockMdocDataSource`] with a single [`Mdoc`] that is based on the
/// attributes in the example from the ISO spec, resigned with the keys generated during test
/// setup. The private key used to sign this [`Mdoc`] is placed in a [`SoftwareKeyFactory`].
async fn prepare_example_holder_mocks(
    issuer_key_pair: &KeyPair<SigningKey>,
    issuer_trust_anchors: &[TrustAnchor<'_>],
) -> (MockMdocDataSource, MockRemoteKeyFactory) {
    // Extract the the attributes from the example DeviceResponse in the ISO specs.
    let example_document = DeviceResponse::example().documents.unwrap().into_iter().next().unwrap();
    let example_attributes = example_document
        .issuer_signed
        .name_spaces
        .unwrap()
        .into_inner()
        .into_iter()
        .map(|(namespace, attributes)| {
            let attributes = attributes
                .into_inner()
                .into_iter()
                .map(|TaggedBytes(item)| Entry {
                    name: item.element_identifier,
                    value: item.element_value,
                })
                .collect();

            (namespace, attributes)
        })
        .collect::<IndexMap<_, Vec<_>>>();

    // Use these attributes to create an unsigned Mdoc.
    let unsigned_mdoc = UnsignedMdoc {
        doc_type: example_document.doc_type,
        valid_from: Utc::now().into(),
        valid_until: (Utc::now() + Days::new(365)).into(),
        attributes: example_attributes.try_into().unwrap(),
        copy_count: 1.try_into().unwrap(),
    };

    let metadata = TypeMetadata::bsn_only_example();
    let metadata_chain = TypeMetadataChain::create(metadata, vec![]).unwrap();

    // Generate a new private key and use that and the issuer key to sign the Mdoc.
    let mdoc_private_key_id = utils::random_string(16);
    let mdoc_private_key = MockRemoteEcdsaKey::new_random(mdoc_private_key_id.clone());
    let mdoc_public_key = mdoc_private_key.verifying_key().try_into().unwrap();
    let issuer_signed = IssuerSigned::sign(unsigned_mdoc, metadata_chain, mdoc_public_key, issuer_key_pair)
        .await
        .unwrap();
    let mdoc =
        Mdoc::new::<MockRemoteEcdsaKey>(mdoc_private_key_id, issuer_signed, &TimeGenerator, issuer_trust_anchors)
            .unwrap();

    // Place the Mdoc in a MockMdocDataSource and the private key in a SoftwareKeyFactory and return them.
    let mdoc_data_source = MockMdocDataSource::new(vec![mdoc]);
    let key_factory = MockRemoteKeyFactory::new(vec![mdoc_private_key]);

    (mdoc_data_source, key_factory)
}

async fn perform_full_disclosure(session_type: SessionType) -> (Client, SessionToken, BaseUrl, Option<BaseUrl>) {
    // Start the wallet_server and create a disclosure request.
    let (settings, client, session_token, internal_url, issuer_key_pair, rp_trust_anchor) =
        start_disclosure(MemorySessionStore::default()).await;

    // Fetching the status should return OK, be in the Created state and include a universal link.
    let status_url = format_status_url(&settings.server_settings.public_url, &session_token, Some(session_type));

    let StatusResponse::Created { ul: Some(ul) } = get_status_ok(&client, status_url.clone()).await else {
        panic!("session should be in CREATED state and a universal link should be provided")
    };

    // Prepare a holder with a valid example Mdoc. Use the query portion of the
    // universal link to have holder code contact the wallet_sever and start disclosure.
    // This should result in a proposal to disclosure for the holder.
    let (mdoc_data_source, key_factory) = prepare_example_holder_mocks(
        &issuer_key_pair,
        &settings
            .server_settings
            .issuer_trust_anchors
            .iter()
            .map(|anchor| anchor.as_trust_anchor().clone())
            .collect_vec(),
    )
    .await;

    let request_uri_query = ul.as_ref().query().unwrap().to_string();
    let uri_source = match session_type {
        SessionType::SameDevice => DisclosureUriSource::Link,
        SessionType::CrossDevice => DisclosureUriSource::QrCode,
    };
    let disclosure_session = DisclosureSession::start(
        HttpVpMessageClient::from(client.clone()),
        &request_uri_query,
        uri_source,
        &mdoc_data_source,
        &[rp_trust_anchor],
    )
    .await
    .expect("disclosure session should start at client side");

    let DisclosureSession::Proposal(proposal) = disclosure_session else {
        panic!("should have received a disclosure proposal")
    };

    // The status endpoint should now respond that the session is in the WaitingForResponse state.
    assert_matches!(
        get_status_ok(&client, status_url.clone()).await,
        StatusResponse::WaitingForResponse
    );

    // Have the holder actually disclosure the example attributes to the wallet_server,
    // after which the status endpoint should report that the session is Done.
    let return_url = proposal
        .disclose(&key_factory)
        .await
        .expect("should disclose attributes successfully");

    assert_matches!(get_status_ok(&client, status_url).await, StatusResponse::Done);

    (client, session_token, internal_url, return_url)
}

fn check_example_disclosed_attributes(disclosed_attributes: &DisclosedAttributes) {
    itertools::assert_equal(disclosed_attributes.keys(), [EXAMPLE_DOC_TYPE]);
    let attributes = &disclosed_attributes.get(EXAMPLE_DOC_TYPE).unwrap().attributes;
    itertools::assert_equal(attributes.keys(), [EXAMPLE_NAMESPACE]);
    let (first_entry_name, first_entry_value) = attributes.get(EXAMPLE_NAMESPACE).unwrap().first().unwrap();
    assert_eq!(first_entry_name, EXAMPLE_ATTR_NAME);
    assert_eq!(first_entry_value, LazyLock::force(&EXAMPLE_ATTR_VALUE));
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
    let disclosed_attributes = response.json::<DisclosedAttributes>().await.unwrap();

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
    let disclosed_attributes = response.json::<DisclosedAttributes>().await.unwrap();

    check_example_disclosed_attributes(&disclosed_attributes);
}

#[tokio::test]
async fn test_disclosed_attributes_failed_session() {
    // Start the wallet_server and create a disclosure request.
    let (settings, client, session_token, internal_url, _, rp_trust_anchor) =
        start_disclosure(MemorySessionStore::default()).await;

    // Fetching the status should return OK, be in the Created state and include a universal link.
    let status_url = format_status_url(
        &settings.server_settings.public_url,
        &session_token,
        Some(SessionType::CrossDevice),
    );

    let StatusResponse::Created { ul: Some(ul) } = get_status_ok(&client, status_url.clone()).await else {
        panic!("session should be in CREATED state and a universal link should be provided")
    };

    // Start a disclosure session with the default MockMdocDataSource, which contains expired
    // attestations from the examples in the ISO specifications, then disclose those.
    let request_uri_query = ul.as_ref().query().unwrap().to_string();
    let disclosure_session = DisclosureSession::start(
        HttpVpMessageClient::from(client.clone()),
        &request_uri_query,
        DisclosureUriSource::QrCode,
        &MockMdocDataSource::new_with_example(),
        &[rp_trust_anchor],
    )
    .await
    .expect("disclosure session should start at client side");

    let DisclosureSession::Proposal(proposal) = disclosure_session else {
        panic!("should have received a disclosure proposal")
    };

    proposal
        .disclose(&MockRemoteKeyFactory::default())
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
    assert!(error_body
        .extra
        .get("session_error")
        .unwrap()
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("expired"));
}
