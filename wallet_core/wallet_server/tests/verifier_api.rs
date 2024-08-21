use std::{
    net::{IpAddr, TcpListener},
    process,
    str::FromStr,
    sync::{Arc, LazyLock},
    time::Duration,
};

use assert_matches::assert_matches;
use chrono::{DateTime, Utc};
use http::StatusCode;
use indexmap::IndexMap;
use itertools::Itertools;
use parking_lot::RwLock;
use reqwest::{Client, Response};
use rstest::rstest;
use tokio::time;

use nl_wallet_mdoc::{
    examples::{
        Example, Examples, IsoCertTimeGenerator, EXAMPLE_ATTR_NAME, EXAMPLE_ATTR_VALUE, EXAMPLE_DOC_TYPE,
        EXAMPLE_NAMESPACE,
    },
    utils::{
        mock_time::MockTimeGenerator,
        serialization::{CborSeq, TaggedBytes},
    },
    verifier::DisclosedAttributes,
    DeviceAuthenticationBytes, DeviceResponse, ItemsRequest,
};
use openid4vc::{
    return_url::ReturnUrlTemplate,
    server_state::{
        MemorySessionStore, SessionState, SessionStore, SessionStoreTimeouts, SessionToken, CLEANUP_INTERVAL_SECONDS,
    },
    verifier::{DisclosureData, Done, SessionResult, SessionType, StatusResponse, VerifierUrlParameters},
    ErrorResponse,
};
use url::Url;
use wallet_common::{
    config::wallet_config::BaseUrl, http_error::HttpJsonErrorBody, reqwest::default_reqwest_client_builder,
};
use wallet_server::{
    settings::{Authentication, RequesterAuth, Server, Settings},
    verifier::{StartDisclosureRequest, StartDisclosureResponse, StatusParams},
};

fn start_disclosure_request() -> StartDisclosureRequest {
    StartDisclosureRequest {
        usecase: String::from("xyz_bank_no_return_url"),
        return_url_template: None,
        items_requests: vec![ItemsRequest {
            doc_type: "com.example.pid".to_owned(),
            request_info: None,
            name_spaces: IndexMap::from([(
                "com.example.pid".to_owned(),
                IndexMap::from_iter(
                    [("given_name", true)]
                        .into_iter()
                        .map(|(name, intent_to_retain)| (name.to_string(), intent_to_retain)),
                ),
            )]),
        }]
        .into(),
    }
}

fn find_listener_port() -> u16 {
    TcpListener::bind("localhost:0")
        .expect("Could not find TCP port")
        .local_addr()
        .expect("Could not get local address from TCP listener")
        .port()
}

fn wallet_server_settings() -> Settings {
    let mut settings = Settings::new_custom("ws_integration_test.toml", "ws_integration_test").unwrap();
    let ws_port = find_listener_port();

    settings.wallet_server.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.wallet_server.port = ws_port;

    let requester_port = find_listener_port();
    settings.requester_server = RequesterAuth::InternalEndpoint(Server {
        ip: IpAddr::from_str("127.0.0.1").unwrap(),
        port: requester_port,
    });

    settings.urls.public_url = format!("http://localhost:{ws_port}/").parse().unwrap();

    settings
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
    let mut settings = wallet_server_settings();
    let internal_url = internal_url(&auth, &settings.urls.public_url);
    settings.requester_server = auth.clone();

    start_wallet_server(settings.clone(), MemorySessionStore::default()).await;

    let client = default_reqwest_client_builder().build().unwrap();

    let start_request = start_disclosure_request();

    // check if using no token returns a 401 on the (public) start URL if an API key is used and a 404 otherwise (because it is served on the internal URL)
    let response = client
        .post(settings.urls.public_url.join("disclosure/sessions"))
        .json(&start_request)
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
        .json(&start_request)
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
        .json(&start_request)
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
        .json(&start_request)
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
        .json(&start_request)
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
        .json(&start_request)
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
        .json(&start_request)
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
        .json(&start_request)
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
    let settings = wallet_server_settings();
    let internal_url = internal_url(&settings.requester_server, &settings.urls.public_url);
    start_wallet_server(settings, MemorySessionStore::default()).await;
    let client = default_reqwest_client_builder().build().unwrap();

    let bad_use_case_request = {
        let mut request = start_disclosure_request();
        request.usecase = "bad".to_string();
        request
    };

    let no_items_request = {
        let mut request = start_disclosure_request();
        request.items_requests = vec![].into();
        request
    };

    let bad_return_url_request = {
        let mut request = start_disclosure_request();
        request.return_url_template = Some(
            "https://example.com/{session_token}"
                .parse::<ReturnUrlTemplate>()
                .unwrap(),
        );
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
    let settings = wallet_server_settings();
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

async fn start_disclosure<S>(disclosure_sessions: S) -> (Settings, Client, SessionToken, BaseUrl)
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let settings = wallet_server_settings();
    let internal_url = internal_url(&settings.requester_server, &settings.urls.public_url);

    start_wallet_server(settings.clone(), disclosure_sessions).await;

    // Create a new disclosure session, which should return 200.
    let client = default_reqwest_client_builder().build().unwrap();
    let response = client
        .post(internal_url.join("disclosure/sessions"))
        .json(&start_disclosure_request())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let disclosure_response = response.json::<StartDisclosureResponse>().await.unwrap();

    (settings, client, disclosure_response.session_token, internal_url)
}

#[tokio::test]
async fn test_disclosure_missing_session_type() {
    let (settings, client, session_token, _) = start_disclosure(MemorySessionStore::default()).await;

    // Check if requesting the session status without a session_type returns a 200, but without the universal link.
    let status_url = format_status_url(&settings.urls.public_url, &session_token, None);

    assert_matches!(
        get_status_ok(&client, status_url).await,
        StatusResponse::Created { ul: None }
    );
}

#[tokio::test]
async fn test_disclosure_cancel() {
    let (settings, client, session_token, _) = start_disclosure(MemorySessionStore::default()).await;

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

    let (settings, client, session_token, internal_url) = start_disclosure(session_store).await;

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
    let settings = wallet_server_settings();

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

    let settings = wallet_server_settings();
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

#[rstest]
#[tokio::test]
async fn test_disclosed_attributes(#[values(None, Some("nonce"))] nonce: Option<&str>) {
    // Create a `DisclosedAttributes` object, based on the example `DeviceResponse`.
    let device_response = DeviceResponse::example();
    let TaggedBytes(CborSeq(device_auth_keyed)) = DeviceAuthenticationBytes::example();
    let disclosed_attributes = device_response
        .verify(
            Some(&Examples::ephemeral_reader_key()),
            &device_auth_keyed.session_transcript,
            &IsoCertTimeGenerator,
            Examples::iaca_trust_anchors(),
        )
        .unwrap();

    // Populate a session store with one session that is `Done` and has these `DisclosedAttributes` available.
    let session_token = SessionToken::new("foobar");
    let session = SessionState::new(
        session_token.clone(),
        DisclosureData::Done(Done {
            session_result: SessionResult::Done {
                disclosed_attributes,
                redirect_uri_nonce: nonce.map(ToOwned::to_owned),
            },
        }),
    );

    let session_store = MemorySessionStore::<DisclosureData>::default();
    session_store.write(session, true).await.unwrap();

    // Start the wallet server with this session store.
    let settings = wallet_server_settings();
    let internal_url = internal_url(&settings.requester_server, &settings.urls.public_url);
    start_wallet_server(settings.clone(), session_store).await;
    let client = default_reqwest_client_builder().build().unwrap();

    // Check if the disclosed attributes endpoint returns a 200 for the session, with the attributes.
    let mut disclosed_attributes_url = internal_url.join(&format!(
        "disclosure/sessions/{}/disclosed_attributes",
        session_token.as_ref()
    ));
    if let Some(nonce) = nonce {
        // Set the return URL nonce as query parameter.
        disclosed_attributes_url.query_pairs_mut().append_pair("nonce", nonce);
    }

    let response = client.get(disclosed_attributes_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let disclosed_attributes = response.json::<DisclosedAttributes>().await.unwrap();

    itertools::assert_equal(disclosed_attributes.keys(), [EXAMPLE_DOC_TYPE]);
    let attributes = &disclosed_attributes.get(EXAMPLE_DOC_TYPE).unwrap().attributes;
    itertools::assert_equal(attributes.keys(), [EXAMPLE_NAMESPACE]);
    let (first_entry_name, first_entry_value) = attributes.get(EXAMPLE_NAMESPACE).unwrap().first().unwrap();
    assert_eq!(first_entry_name, EXAMPLE_ATTR_NAME);
    assert_eq!(first_entry_value, LazyLock::force(&EXAMPLE_ATTR_VALUE));
}

#[tokio::test]
async fn test_disclosed_attributes_error_nonce() {
    // Populate a session store with one session that is `Done` and has a nonce.
    let session_token = SessionToken::new("foobar");
    let session = SessionState::new(
        session_token.clone(),
        DisclosureData::Done(Done {
            session_result: SessionResult::Done {
                disclosed_attributes: Default::default(),
                redirect_uri_nonce: Some("nonce".to_string()),
            },
        }),
    );

    let session_store = MemorySessionStore::<DisclosureData>::default();
    session_store.write(session, true).await.unwrap();

    // Start the wallet server with this session store.
    let settings = wallet_server_settings();
    let internal_url = internal_url(&settings.requester_server, &settings.urls.public_url);
    start_wallet_server(settings.clone(), session_store).await;
    let client = default_reqwest_client_builder().build().unwrap();

    // Check that requesting the disclosed attributes for this session returns the correct errors.
    let mut disclosed_attributes_url = internal_url.join(&format!(
        "disclosure/sessions/{}/disclosed_attributes",
        session_token.as_ref()
    ));

    let response = client.get(disclosed_attributes_url.clone()).send().await.unwrap();

    test_http_json_error_body(response, StatusCode::UNAUTHORIZED, "nonce").await;

    disclosed_attributes_url
        .query_pairs_mut()
        .append_pair("none", "incorrect");
    let response = client.get(disclosed_attributes_url).send().await.unwrap();

    test_http_json_error_body(response, StatusCode::UNAUTHORIZED, "nonce").await;
}

#[tokio::test]
async fn test_disclosed_attributes_error_session_state() {
    let cancelled_session_token = SessionToken::new("cancelled");
    let cancelled_session = SessionState::new(
        cancelled_session_token.clone(),
        DisclosureData::Done(Done {
            session_result: SessionResult::Cancelled,
        }),
    );

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
    session_store.write(cancelled_session, true).await.unwrap();
    session_store.write(failed_session, true).await.unwrap();

    // Start the wallet server with this session store.
    let settings = wallet_server_settings();
    let internal_url = internal_url(&settings.requester_server, &settings.urls.public_url);
    start_wallet_server(settings.clone(), session_store).await;
    let client = default_reqwest_client_builder().build().unwrap();

    let response = client
        .get(internal_url.join(&format!(
            "disclosure/sessions/{}/disclosed_attributes",
            cancelled_session_token.as_ref()
        )))
        .send()
        .await
        .unwrap();

    let error_body = test_http_json_error_body(response, StatusCode::BAD_REQUEST, "session_state").await;
    itertools::assert_equal(error_body.extra.keys(), ["session_status"]);
    assert_eq!(
        error_body.extra.get("session_status").unwrap(),
        &serde_json::Value::from("CANCELLED")
    );

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
