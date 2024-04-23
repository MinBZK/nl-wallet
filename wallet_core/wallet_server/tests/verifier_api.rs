use std::{
    net::{IpAddr, TcpListener},
    process,
    str::FromStr,
    time::Duration,
};

use http::StatusCode;
use indexmap::IndexMap;
use nl_wallet_mdoc::{verifier::SessionType, ItemsRequest};
use rstest::rstest;
use tokio::time;

use wallet_common::{config::wallet_config::BaseUrl, reqwest::default_reqwest_client_builder};
use wallet_server::{
    settings::{Authentication, RequesterAuth, Server, Settings},
    store::SessionStores,
    verifier::{StartDisclosureRequest, StartDisclosureResponse},
};

fn find_listener_port() -> u16 {
    TcpListener::bind("localhost:0")
        .expect("Could not find TCP port")
        .local_addr()
        .expect("Could not get local address from TCP listener")
        .port()
}

fn wallet_server_settings() -> Settings {
    let mut settings = Settings::new().unwrap();
    let ws_port = find_listener_port();

    settings.wallet_server.ip = IpAddr::from_str("127.0.0.1").unwrap();
    settings.wallet_server.port = ws_port;

    let requester_port = find_listener_port();
    settings.requester_server = RequesterAuth::InternalEndpoint(Server {
        ip: IpAddr::from_str("127.0.0.1").unwrap(),
        port: requester_port,
    });

    settings.public_url = format!("http://localhost:{ws_port}/").parse().unwrap();
    settings.internal_url = format!("http://localhost:{requester_port}/").parse().unwrap();

    settings.store_url = "memory://".parse().unwrap();
    settings
}

async fn start_wallet_server(settings: Settings) {
    let public_url = settings.public_url.clone();
    let sessions = SessionStores::init(settings.store_url.clone()).await.unwrap();
    tokio::spawn(async move {
        if let Err(error) = wallet_server::server::serve_disclosure(settings, sessions).await {
            println!("Could not start wallet_server: {error:?}");

            process::exit(1);
        }
    });

    wait_for_server(public_url.join_base_url("disclosure/")).await;
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
    // fix the internal_url
    match auth {
        RequesterAuth::ProtectedInternalEndpoint {
            server: Server { port, .. },
            ..
        }
        | RequesterAuth::InternalEndpoint(Server { port, .. }) => {
            settings.internal_url = format!("http://localhost:{port}/").parse().unwrap()
        }
        RequesterAuth::Authentication(_) => settings.internal_url = settings.public_url.clone(),
    };

    settings.requester_server = auth.clone();

    start_wallet_server(settings.clone()).await;

    let client = default_reqwest_client_builder().build().unwrap();

    let start_request = StartDisclosureRequest {
        usecase: String::from("xyz_bank"),
        session_type: SessionType::CrossDevice,
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
    };

    // check if using no token returns a 401 on the (public) start URL if an API key is used and a 404 otherwise (because it is served on the internal URL)
    let response = client
        .post(settings.public_url.join("disclosure/sessions"))
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
        .post(settings.internal_url.join("disclosure/sessions"))
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
        .post(settings.public_url.join("disclosure/sessions"))
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
        .post(settings.internal_url.join("disclosure/sessions"))
        .header("Authorization", "Bearer secret_key")
        .json(&start_request)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let disclosed_attributes_path = String::from(
        response
            .json::<StartDisclosureResponse>()
            .await
            .unwrap()
            .disclosed_attributes_url
            .path(),
    );

    // check if using no token returns a 401 on the (public) attributes URL if an API key is used and a 404 otherwise (because it is served on the internal URL)
    let response = client
        .get(settings.public_url.join(&disclosed_attributes_path))
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
        .get(settings.internal_url.join(&disclosed_attributes_path))
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
        .get(settings.public_url.join(&disclosed_attributes_path))
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
        .get(settings.internal_url.join(&disclosed_attributes_path))
        .header("Authorization", "Bearer secret_key")
        .json(&start_request)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_disclosure_not_found() {
    let settings = wallet_server_settings();
    start_wallet_server(settings.clone()).await;

    let client = default_reqwest_client_builder().build().unwrap();
    // check if a non-existent token returns a 404 on the status URL
    let response = client
        .get(
            settings
                .public_url
                .join("disclosure/sessions/nonexistent_session/status"),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // check if a non-existent token returns a 404 on the wallet URL
    let response = client
        .post(settings.public_url.join("disclosure/sessions/nonexistent_session"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // check if a non-existent token returns a 404 on the disclosed_attributes URL
    let response = client
        .get(
            settings
                .internal_url
                .join("disclosure/sessions/nonexistent_session/disclosed_attributes"),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
