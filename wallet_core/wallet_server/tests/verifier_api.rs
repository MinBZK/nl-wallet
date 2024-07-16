use std::{
    net::{IpAddr, TcpListener},
    process,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use assert_matches::assert_matches;
use chrono::{DateTime, Utc};
use http::StatusCode;
use indexmap::IndexMap;
use parking_lot::RwLock;
use reqwest::{Client, Response};
use rstest::rstest;
use tokio::time;

use nl_wallet_mdoc::{
    server_state::{MemorySessionStore, SessionStore, SessionStoreTimeouts, SessionToken, CLEANUP_INTERVAL_SECONDS},
    utils::mock_time::MockTimeGenerator,
    verifier::{ReturnUrlTemplate, SessionType},
    ItemsRequest,
};
use openid4vc::{
    verifier::{DisclosureData, StatusResponse, VerifierUrlParameters},
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

async fn test_http_json_error_body(response: Response, status_code: StatusCode, error_type: &str) {
    assert_eq!(response.status(), status_code);

    let body = serde_json::from_slice::<HttpJsonErrorBody<String>>(&response.bytes().await.unwrap())
        .expect("response body should deserialize to HttpJsonErrorBody");

    assert_eq!(body.r#type, error_type);
    assert_eq!(body.status, Some(status_code));
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
    test_http_json_error_body(response, StatusCode::NOT_FOUND, "unknown_session").await
}

fn format_status_url(public_url: &BaseUrl, session_token: &SessionToken, session_type: SessionType) -> Url {
    let mut status_url = public_url.join(&format!("disclosure/sessions/{session_token}"));

    let status_query = serde_urlencoded::to_string(StatusParams { session_type }).unwrap();
    status_url.set_query(status_query.as_str().into());

    status_url
}

async fn get_status_ok(client: &Client, status_url: Url) -> StatusResponse {
    let response = client.get(status_url.clone()).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    response.json::<StatusResponse>().await.unwrap()
}

#[tokio::test]
async fn test_disclosure_cancel() {
    let settings = wallet_server_settings();
    let internal_url = internal_url(&settings.requester_server, &settings.urls.public_url);

    start_wallet_server(settings.clone(), MemorySessionStore::default()).await;

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
    let session_token = disclosure_response.session_token;

    // Fetching the status should return OK and be in the Created state.
    let status_url = format_status_url(&settings.urls.public_url, &session_token, SessionType::SameDevice);

    assert_matches!(
        get_status_ok(&client, status_url.clone()).await,
        StatusResponse::Created { .. }
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
    let internal_url = internal_url(&settings.requester_server, &settings.urls.public_url);
    start_wallet_server(settings.clone(), session_store).await;

    let client = default_reqwest_client_builder().build().unwrap();

    // Create a new disclosure session, which should return 200.
    let response = client
        .post(internal_url.join("disclosure/sessions"))
        .json(&start_disclosure_request())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let disclosure_response = response.json::<StartDisclosureResponse>().await.unwrap();
    let session_token = disclosure_response.session_token;
    let status_url = format_status_url(&settings.urls.public_url, &session_token, SessionType::SameDevice);
    let disclosed_attributes_url =
        internal_url.join(&format!("disclosure/sessions/{}/disclosed_attributes", session_token));

    // Fetch the status, this should return OK and be in the Created state.
    assert_matches!(
        get_status_ok(&client, status_url.clone()).await,
        StatusResponse::Created { .. }
    );

    // Fetching the disclosed attributes should return 400, since the session is not finished.
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
