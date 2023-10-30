use std::{
    collections::HashMap,
    net::{IpAddr, TcpListener},
    str::FromStr,
};

use base64::prelude::*;
use chrono::Duration;
use indexmap::IndexMap;
use p256::pkcs8::EncodePrivateKey;
use reqwest::StatusCode;
use tracing_subscriber::FmtSubscriber;
use url::Url;

use nl_wallet_mdoc::{
    server_state::{MemorySessionStore, SessionState, SessionStore, SessionToken},
    utils::{
        reader_auth::{DeletionPolicy, Organization, ReaderRegistration, RetentionPolicy, SharingPolicy},
        serialization::cbor_deserialize,
        x509::{Certificate, CertificateType},
    },
    verifier::DisclosureData,
    ItemsRequest, ReaderEngagement,
};
use wallet_server::{
    server,
    settings::{KeyPair, Server, Settings},
    verifier::{StartDisclosureRequest, StartDisclosureResponse},
};

fn find_listener_port() -> u16 {
    TcpListener::bind("localhost:0")
        .expect("Could not find TCP port")
        .local_addr()
        .expect("Could not get local address from TCP listener")
        .port()
}

// Test fixture
fn get_my_reader_auth() -> ReaderRegistration {
    let my_organization = Organization {
        display_name: vec![("nl", "Mijn Organisatienaam"), ("en", "My Organization Name")].into(),
        legal_name: vec![("nl", "Organisatie"), ("en", "Organization")].into(),
        description: vec![
            ("nl", "Beschrijving van Mijn Organisatie"),
            ("en", "Description of My Organization"),
        ]
        .into(),
        city: Some(vec![("nl", "Den Haag"), ("en", "The Hague")].into()),
        country: Some("nl".to_owned()),
        web_url: Some(Url::parse("https://www.ons-dorp.nl").unwrap()),
        privacy_policy_url: Some(Url::parse("https://www.ons-dorp.nl/privacy").unwrap()),
        logo: None,
    };
    ReaderRegistration {
        id: "some-service-id".to_owned(),
        name: vec![("nl", "Naam van mijn dienst"), ("en", "My Service Name")].into(),
        purpose_statement: vec![("nl", "Beschrijving van mijn dienst"), ("en", "My Service Description")].into(),
        retention_policy: RetentionPolicy {
            intent_to_retain: true,
            max_duration: Some(Duration::minutes(525600)),
        },
        sharing_policy: SharingPolicy { intent_to_share: true },
        deletion_policy: DeletionPolicy { deleteable: true },
        organization: my_organization,
        attributes: Default::default(),
    }
}

fn wallet_server_settings() -> Settings {
    let port = find_listener_port();
    let port2 = find_listener_port();

    let mut settings = Settings {
        wallet_server: Server {
            ip: IpAddr::from_str("127.0.0.1").unwrap(),
            port,
        },
        requester_server: Some(Server {
            ip: IpAddr::from_str("127.0.0.1").unwrap(),
            port: port2,
        }),
        public_url: format!("http://127.0.0.1:{}/", port).parse().unwrap(),
        internal_url: Some(format!("http://127.0.0.1:{}/", port2).parse().unwrap()),
        usecases: HashMap::new(),
        trust_anchors: Vec::new(),
    };
    let (ca, ca_privkey) = Certificate::new_ca("ca.example.com").unwrap();
    let (cert, cert_privkey) = Certificate::new(
        &ca,
        &ca_privkey,
        "cert.example.com",
        CertificateType::ReaderAuth(Box::new(get_my_reader_auth())),
    )
    .unwrap();

    settings.usecases.insert(
        "example_usecase".to_owned(),
        KeyPair {
            certificate: BASE64_STANDARD.encode(cert.as_bytes()),
            private_key: BASE64_STANDARD.encode(
                cert_privkey
                    .to_pkcs8_der()
                    .expect("could not serialize private key")
                    .as_bytes(),
            ),
        },
    );

    settings
}

async fn start_wallet_server<S>(settings: Settings, sessions: S)
where
    S: SessionStore<Data = SessionState<DisclosureData>> + Send + Sync + 'static,
{
    let public_url = settings.public_url.clone();
    tokio::spawn(async move {
        server::serve::<S>(&settings, sessions)
            .await
            .expect("Could not start server");
    });

    let _ = tracing::subscriber::set_global_default(FmtSubscriber::new());

    // wait for the server to come up
    let client = reqwest::Client::new();
    loop {
        match client.get(public_url.join("/health").unwrap()).send().await {
            Ok(_) => break,
            _ => {
                println!("Waiting for wallet_server...");
                tokio::time::sleep(Duration::milliseconds(100).to_std().unwrap()).await
            }
        }
    }
}

fn parse_wallet_url(engagement_url: Url) -> Url {
    let reader_engagement: ReaderEngagement = cbor_deserialize(
        BASE64_URL_SAFE_NO_PAD
            .decode(
                engagement_url
                    .path_segments()
                    .expect("no path in engagement_url")
                    .last()
                    .expect("empty path in engagement_url"),
            )
            .unwrap()
            .as_slice(),
    )
    .unwrap();

    reader_engagement
        .0
        .connection_methods
        .expect("no connection_methods in reader_engagement")
        .first()
        .expect("empty connection_methods in reader_engagement")
        .0
        .connection_options
        .0
        .uri
        .clone()
}

#[tokio::test]
async fn test_start_session() {
    let settings = wallet_server_settings();
    let sessions = MemorySessionStore::<DisclosureData>::new();

    start_wallet_server(settings.clone(), sessions).await;

    let client = reqwest::Client::new();

    let start_request = StartDisclosureRequest {
        usecase: "example_usecase".to_owned(),
        items_requests: vec![ItemsRequest {
            doc_type: "example_doctype".to_owned(),
            request_info: None,
            name_spaces: IndexMap::from([(
                "example_namespace".to_owned(),
                IndexMap::from_iter(
                    [("first_name", true), ("family_name", false)]
                        .iter()
                        .map(|(name, intent_to_retain)| (name.to_string(), *intent_to_retain)),
                ),
            )]),
        }],
    };
    let response = client
        .post(
            settings
                .internal_url
                .unwrap()
                .join("/sessions")
                .expect("could not join url with endpoint"),
        )
        .json(&start_request)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // does it exist for the RP side of things?
    let StartDisclosureResponse {
        session_url,
        engagement_url,
    } = response.json::<StartDisclosureResponse>().await.unwrap();
    let response = client.get(session_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // does it exist for the wallet side of things?
    let wallet_url = parse_wallet_url(engagement_url);
    let response = client.post(wallet_url).body("hello").send().await.unwrap();

    assert_ne!(response.status(), StatusCode::NOT_FOUND);

    // TODO construct a valid body when we have the code to do so, until then this is a bad request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_session_not_found() {
    let settings = wallet_server_settings();
    let sessions = MemorySessionStore::<DisclosureData>::new();

    start_wallet_server(settings.clone(), sessions).await;

    let client = reqwest::Client::new();

    // does it exist for the RP side of things?
    let response = client
        .get(
            settings
                .internal_url
                .unwrap()
                .join(&format!("/sessions/{}/status", SessionToken::new()))
                .unwrap(),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // does it exist for the wallet side of things?
    let response = client
        .post(settings.public_url.join(&format!("/{}", SessionToken::new())).unwrap())
        // .json(...) // there's no way to construct a valid body here
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
