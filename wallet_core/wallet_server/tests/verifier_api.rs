use std::{
    collections::HashMap,
    net::{IpAddr, TcpListener},
    process,
    str::FromStr,
    sync::{Arc, LazyLock},
    time::Duration,
};

use assert_matches::assert_matches;
use chrono::{DateTime, Days, Utc};
use http::StatusCode;
use indexmap::IndexMap;
use itertools::Itertools;
use p256::{ecdsa::SigningKey, pkcs8::EncodePrivateKey};
use parking_lot::RwLock;
use rand_core::OsRng;
use reqwest::{Client, Response};
use rstest::rstest;
use tokio::time;

use nl_wallet_mdoc::{
    examples::{Example, EXAMPLE_ATTR_NAME, EXAMPLE_ATTR_VALUE, EXAMPLE_DOC_TYPE, EXAMPLE_NAMESPACE},
    holder::{mock::MockMdocDataSource, Mdoc, TrustAnchor},
    server_keys::KeyPair,
    software_key_factory::SoftwareKeyFactory,
    unsigned::{Entry, UnsignedMdoc},
    utils::{
        issuer_auth::IssuerRegistration, mock_time::MockTimeGenerator, reader_auth::ReaderRegistration,
        serialization::TaggedBytes,
    },
    verifier::DisclosedAttributes,
    DeviceResponse, IssuerSigned, ItemsRequest,
};
use openid4vc::{
    disclosure_session::{DisclosureSession, DisclosureUriSource, HttpVpMessageClient},
    server_state::{
        MemorySessionStore, SessionState, SessionStore, SessionStoreTimeouts, SessionToken, CLEANUP_INTERVAL_SECONDS,
    },
    verifier::{
        DisclosureData, Done, SessionResult, SessionType, SessionTypeReturnUrl, StatusResponse, VerifierUrlParameters,
    },
    ErrorResponse,
};
use url::Url;
use wallet_common::{
    config::wallet_config::BaseUrl, generator::TimeGenerator, http_error::HttpJsonErrorBody,
    keys::software::SoftwareEcdsaKey, reqwest::default_reqwest_client_builder, trust_anchor::OwnedTrustAnchor, utils,
};
use wallet_server::{
    settings::{Authentication, RequesterAuth, Server, Settings, Storage, Urls, Verifier, VerifierUseCase},
    verifier::{StartDisclosureRequest, StartDisclosureResponse, StatusParams},
};

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

#[cfg(feature = "issuance")]
fn fake_issuer_settings() -> wallet_server::settings::Issuer {
    use wallet_server::settings::{Digid, Issuer};

    let url: BaseUrl = "http://fake.fake".parse().unwrap();

    Issuer {
        private_keys: Default::default(),
        wallet_client_ids: Default::default(),
        digid: Digid {
            issuer_url: url.clone(),
            bsn_privkey: Default::default(),
            trust_anchors: Default::default(),
        },
        brp_server: url,
    }
}

fn wallet_server_settings() -> (Settings, KeyPair, OwnedTrustAnchor) {
    // Set up the hostname and ports.
    let localhost = IpAddr::from_str("127.0.0.1").unwrap();
    let ws_port = find_listener_port();
    let rp_port = find_listener_port();

    // Set up the default storage timeouts.
    let default_store_timeouts = SessionStoreTimeouts::default();

    // Create the issuer CA and derive the trust anchor from it.
    let issuer_ca = KeyPair::generate_issuer_mock_ca().unwrap();
    let issuer_key_pair = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .unwrap();
    let issuer_trust_anchor = issuer_ca.certificate().try_into().unwrap();

    // Create the RP CA, derive the trust anchor from it and generate
    // a reader registration, based on the example items request.
    let rp_ca = KeyPair::generate_reader_mock_ca().unwrap();
    let rp_trust_anchor = OwnedTrustAnchor::from(&rp_ca.certificate().try_into().unwrap());
    let reader_registration = Some(ReaderRegistration::new_mock_from_requests(
        &EXAMPLE_START_DISCLOSURE_REQUEST.items_requests,
    ));

    // Set up the use case, based on RP CA and reader registration.
    let usecase_keypair = rp_ca.generate_reader_mock(reader_registration).unwrap();
    let usecases = HashMap::from([(
        USECASE_NAME.to_string(),
        VerifierUseCase {
            session_type_return_url: SessionTypeReturnUrl::SameDevice,
            key_pair: wallet_server::settings::KeyPair {
                certificate: usecase_keypair.certificate().as_bytes().to_vec(),
                private_key: usecase_keypair
                    .private_key()
                    .to_pkcs8_der()
                    .unwrap()
                    .as_bytes()
                    .to_vec(),
            },
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
        requester_server: RequesterAuth::InternalEndpoint(Server {
            ip: localhost,
            port: rp_port,
        }),
        urls: Urls {
            public_url: format!("http://localhost:{ws_port}/").parse().unwrap(),
            universal_link_base_url: "http://universal.link/".parse().unwrap(),
        },
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
        #[cfg(feature = "issuance")]
        issuer: fake_issuer_settings(),
        verifier: Verifier {
            usecases,
            ephemeral_id_secret: utils::random_bytes(64).try_into().unwrap(),
            trust_anchors: vec![issuer_trust_anchor],
        },
        sentry: None,
    };

    (settings, issuer_key_pair, rp_trust_anchor)
}

async fn start_wallet_server<S>(settings: Settings, disclosure_sessions: S)
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let public_url = settings.urls.public_url.clone();

    tokio::spawn(async move {
        if let Err(error) = wallet_server::server::verification_server::serve(settings, disclosure_sessions).await {
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
    let internal_url = internal_url(&auth, &settings.urls.public_url);
    settings.requester_server = auth.clone();

    start_wallet_server(settings.clone(), MemorySessionStore::default()).await;

    let client = default_reqwest_client_builder().build().unwrap();

    // check if using no token returns a 401 on the (public) start URL if an API key is used and a 404 otherwise (because it is served on the internal URL)
    let response = client
        .post(settings.urls.public_url.join("disclosure/sessions"))
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

    // check if using a token returns a 200 on the (public) start URL if an API key is used and a 404 otherwise (because it is served on the internal URL)
    let response = client
        .post(settings.urls.public_url.join("disclosure/sessions"))
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
        .urls
        .public_url
        .join(&format!("disclosure/sessions/{}/disclosed_attributes", session_token));
    let internal_disclosed_attributes_url =
        internal_url.join(&format!("disclosure/sessions/{}/disclosed_attributes", session_token));

    // check if using no token returns a 401 on the (public) attributes URL if an API key is used and a 404 otherwise (because it is served on the internal URL)
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

    // check if using no token returns a 401 on the (internal) attributes URL if an API key is used and a 400 otherwise (because the session is not yet finished)
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

    // check if using a token returns a 400 on the (public) attributes URL if an API key is used and a 404 otherwise (because it is served on the internal URL)
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
    let internal_url = internal_url(&settings.requester_server, &settings.urls.public_url);
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
    let internal_url = internal_url(&settings.requester_server, &settings.urls.public_url);
    start_wallet_server(settings.clone(), MemorySessionStore::default()).await;

    let client = default_reqwest_client_builder().build().unwrap();

    // check if a non-existent token returns a 404 on the status URL
    let status_url = settings.urls.public_url.join("disclosure/sessions/nonexistent_session");
    let response = client.get(status_url).send().await.unwrap();

    test_http_json_error_body(response, StatusCode::NOT_FOUND, "unknown_session").await;

    // check if a non-existent token returns a 404 on the cancel URL
    let cancel_url = settings.urls.public_url.join("disclosure/sessions/nonexistent_session");
    let response = client.delete(cancel_url).send().await.unwrap();

    test_http_json_error_body(response, StatusCode::NOT_FOUND, "unknown_session").await;

    // check if a non-existent token returns a 404 on the wallet URL
    let mut request_uri = settings
        .urls
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
        let status_query = serde_urlencoded::to_string(StatusParams { session_type }).unwrap();
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
) -> (Settings, Client, SessionToken, BaseUrl, KeyPair, OwnedTrustAnchor)
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let (settings, issuer_key_pair, rp_trust_anchor) = wallet_server_settings();
    let internal_url = internal_url(&settings.requester_server, &settings.urls.public_url);

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
    let status_url = format_status_url(&settings.urls.public_url, &session_token, None);

    assert_matches!(
        get_status_ok(&client, status_url).await,
        StatusResponse::Created { ul: None }
    );
}

#[tokio::test]
async fn test_disclosure_cancel() {
    let (settings, client, session_token, internal_url, _, _) = start_disclosure(MemorySessionStore::default()).await;

    // Fetching the status should return OK and be in the Created state.
    let status_url = format_status_url(&settings.urls.public_url, &session_token, Some(SessionType::SameDevice));

    assert_matches!(
        get_status_ok(&client, status_url.clone()).await,
        StatusResponse::Created { ul: Some(_) }
    );

    // Cancel the newly created session, which should return 204 and no body.
    let cancel_url = settings
        .urls
        .public_url
        .join(&format!("disclosure/sessions/{session_token}"));
    let response = client.delete(cancel_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(response.content_length(), Some(0));

    // Fetching the status should return OK and be in the Cancelled state.
    assert_matches!(get_status_ok(&client, status_url).await, StatusResponse::Cancelled);

    // Cancelling the session again should return a 400.
    let cancel_url = settings
        .urls
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

async fn test_disclosure_expired<S>(
    settings: Settings,
    session_store: S,
    mock_time: &RwLock<DateTime<Utc>>,
    use_delay: bool,
) where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let timeouts = SessionStoreTimeouts::from(&settings.storage);

    let (settings, client, session_token, internal_url, _, _) = start_disclosure(session_store).await;

    // Fetch the status, this should return OK and be in the Created state.
    let status_url = format_status_url(&settings.urls.public_url, &session_token, Some(SessionType::SameDevice));
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
    time::advance(CLEANUP_INTERVAL_SECONDS + Duration::from_millis(1)).await;
    time::resume();

    // Wait for the database to have run the cleanup.
    if use_delay {
        time::sleep(Duration::from_millis(100)).await;
    }

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
    time::advance(CLEANUP_INTERVAL_SECONDS + Duration::from_millis(1)).await;
    time::resume();

    // Wait for the database to have run the cleanup.
    if use_delay {
        time::sleep(Duration::from_millis(100)).await;
    }

    // Both the status and disclosed attribute requests should now return 404.
    let response = client.get(status_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let response = client.get(disclosed_attributes_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_disclosure_expired_memory() {
    let (settings, _, _) = wallet_server_settings();

    let timeouts = SessionStoreTimeouts::from(&settings.storage);
    let time_generator = MockTimeGenerator::default();
    let mock_time = Arc::clone(&time_generator.time);
    let session_store = MemorySessionStore::new_with_time(timeouts, time_generator);

    test_disclosure_expired(settings, session_store, mock_time.as_ref(), false).await;
}

#[cfg(feature = "db_test")]
#[tokio::test]
async fn test_disclosure_expired_postgres() {
    use wallet_server::store::postgres::PostgresSessionStore;

    // Combine the generated settings with the storage settings from the configuration file.
    let (mut settings, _, _) = wallet_server_settings();
    let storage_settings = Settings::new_custom("ws_integration_test.toml", "ws_integration_test")
        .unwrap()
        .storage;
    settings.storage = storage_settings;

    assert_eq!(
        settings.storage.url.scheme(),
        "postgres",
        "should be configured to use PostgreSQL storage"
    );

    let timeouts = SessionStoreTimeouts::from(&settings.storage);
    let time_generator = MockTimeGenerator::default();
    let mock_time = Arc::clone(&time_generator.time);
    let session_store = PostgresSessionStore::try_new_with_time(settings.storage.url.clone(), timeouts, time_generator)
        .await
        .unwrap();

    test_disclosure_expired(settings, session_store, mock_time.as_ref(), true).await;
}

/// This utility function is used to prepare the two mocks necessary to emulate a valid holder.
/// It will populate a [`MockMdocDataSource`] with a single [`Mdoc`] that is based on the
/// attributes in the example from the ISO spec, resigned with the keys generated during test
/// setup. The private key used to sign this [`Mdoc`] is placed in a [`SoftwareKeyFactory`].
async fn prepare_example_holder_mocks(
    issuer_key_pair: &KeyPair,
    issuer_trust_anchors: &[TrustAnchor<'_>],
) -> (MockMdocDataSource, SoftwareKeyFactory) {
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

    // Generate a new private key and use that and the issuer key to sign the Mdoc.
    let mdoc_private_key_id = utils::random_string(16);
    let mdoc_private_key = SigningKey::random(&mut OsRng);
    let mdoc_public_key = mdoc_private_key.verifying_key().try_into().unwrap();
    let issuer_signed = IssuerSigned::sign(unsigned_mdoc, mdoc_public_key, issuer_key_pair)
        .await
        .unwrap();
    let mdoc = Mdoc::new::<SoftwareEcdsaKey>(
        mdoc_private_key_id.clone(),
        issuer_signed,
        &TimeGenerator,
        issuer_trust_anchors,
    )
    .unwrap();

    // Place the Mdoc in a MockMdocDataSource and the private key in a SoftwareKeyFactory and return them.
    let mdoc_data_source = MockMdocDataSource::new(vec![mdoc]);
    let key_factory = SoftwareKeyFactory::new(HashMap::from([(mdoc_private_key_id, mdoc_private_key)]));

    (mdoc_data_source, key_factory)
}

async fn perform_full_disclosure(session_type: SessionType) -> (Client, SessionToken, BaseUrl, Option<BaseUrl>) {
    // Start the wallet_server and create a disclosure request.
    let (settings, client, session_token, internal_url, issuer_key_pair, rp_trust_anchor) =
        start_disclosure(MemorySessionStore::default()).await;

    // Fetching the status should return OK, be in the Created state and include a universal link.
    let status_url = format_status_url(&settings.urls.public_url, &session_token, Some(session_type));

    let StatusResponse::Created { ul: Some(ul) } = get_status_ok(&client, status_url.clone()).await else {
        panic!("session should be in CREATED state and a universal link should be provided")
    };

    // Prepare a holder with a valid example Mdoc. Use the query portion of the
    // universal link to have holder code contact the wallet_sever and start disclosure.
    // This should result in a proposal to disclosure for the holder.
    let (mdoc_data_source, key_factory) = prepare_example_holder_mocks(
        &issuer_key_pair,
        &settings
            .verifier
            .trust_anchors
            .iter()
            .map(|anchor| (&anchor.owned_trust_anchor).into())
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
        &[rp_trust_anchor].iter().map(Into::into).collect_vec(),
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
    let first_entry = attributes.get(EXAMPLE_NAMESPACE).unwrap().first().unwrap();
    assert_eq!(first_entry.name, EXAMPLE_ATTR_NAME);
    assert_eq!(&first_entry.value, LazyLock::force(&EXAMPLE_ATTR_VALUE));
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
async fn test_disclosed_attributes_error_session_state() {
    let failed_session_token = SessionToken::new("failed");
    let failed_session = SessionState::new(
        failed_session_token.clone(),
        DisclosureData::Done(Done {
            session_result: SessionResult::Failed {
                error: "This is the error reason.".to_string(),
            },
        }),
    );

    let session_store = MemorySessionStore::<DisclosureData>::default();
    session_store.write(failed_session, true).await.unwrap();

    // Start the wallet server with this session store.
    let (settings, _, _) = wallet_server_settings();
    let internal_url = internal_url(&settings.requester_server, &settings.urls.public_url);
    start_wallet_server(settings.clone(), session_store).await;
    let client = default_reqwest_client_builder().build().unwrap();

    let response = client
        .get(internal_url.join(&format!(
            "disclosure/sessions/{}/disclosed_attributes",
            failed_session_token.as_ref()
        )))
        .send()
        .await
        .unwrap();

    let error_body = test_http_json_error_body(response, StatusCode::BAD_REQUEST, "session_state").await;
    itertools::assert_equal(error_body.extra.keys().sorted(), ["session_error", "session_status"]);
    assert_eq!(
        error_body.extra.get("session_status").unwrap(),
        &serde_json::Value::from("FAILED")
    );
    assert_eq!(
        error_body.extra.get("session_error").unwrap(),
        &serde_json::Value::from("This is the error reason.")
    );
}
