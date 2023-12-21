use std::{
    collections::HashMap,
    net::{IpAddr, TcpListener},
    str::FromStr,
};

use async_trait::async_trait;
use base64::prelude::*;
use chrono::Duration;
use indexmap::IndexMap;
use openid4vc::{
    issuer::{AttributeService, Created},
    token::TokenRequest,
    NL_WALLET_CLIENT_ID,
};
use p256::pkcs8::EncodePrivateKey;
use reqwest::{Client, StatusCode};
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    mock::SoftwareKeyFactory,
    server_state::{SessionState, SessionStore, SessionToken},
    utils::{
        reader_auth::{DeletionPolicy, Organization, ReaderRegistration, RetentionPolicy, SharingPolicy},
        serialization::cbor_deserialize,
        x509::{Certificate, CertificateType},
    },
    verifier::{DisclosureData, SessionType},
    ItemsRequest, ReaderEngagement,
};
use wallet::{
    mock::{default_configuration, MockDigidSession},
    wallet_deps::{
        DigidSession, HttpDigidSession, HttpOpenIdClient, HttpOpenidPidIssuerClient, OpenidPidIssuerClient,
        S256PkcePair,
    },
};
use wallet_common::{config::wallet_config::PidIssuanceConfiguration, trust_anchor::DerTrustAnchor};
use wallet_server::{
    pid::{
        attributes::{AttributesLookup, PidAttributeService},
        mock::{MockAttributesLookup, MockBsnLookup},
    },
    server,
    settings::{Digid, Issuer, KeyPair, Server, Settings},
    store::new_session_store,
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
        category: vec![("nl", "Categorie"), ("en", "Category")].into(),
        kvk: Some("1234 1234".to_owned()),
        city: Some(vec![("nl", "Den Haag"), ("en", "The Hague")].into()),
        department: Some(vec![("nl", "Afdeling"), ("en", "Department")].into()),
        country_code: Some("nl".to_owned()),
        web_url: Some(Url::parse("https://www.ons-dorp.nl").unwrap()),
        privacy_policy_url: Some(Url::parse("https://www.ons-dorp.nl/privacy").unwrap()),
        logo: None,
    };
    ReaderRegistration {
        id: "some-service-id".to_owned(),
        purpose_statement: vec![("nl", "Beschrijving van mijn dienst"), ("en", "My Service Description")].into(),
        retention_policy: RetentionPolicy {
            intent_to_retain: true,
            max_duration_in_minutes: Some(60 * 24 * 365),
        },
        sharing_policy: SharingPolicy { intent_to_share: true },
        deletion_policy: DeletionPolicy { deleteable: true },
        organization: my_organization,
        attributes: Default::default(),
    }
}

fn wallet_server_settings() -> (Settings, Certificate) {
    let port = find_listener_port();
    let port2 = find_listener_port();

    let (issuance_ca, issuance_ca_privkey) = Certificate::new_ca("ca.example.com").unwrap();
    let (issuer_cert, issuer_privkey) = Certificate::new(
        &issuance_ca,
        &issuance_ca_privkey,
        "cert.example.com",
        CertificateType::Mdl,
    )
    .unwrap();

    // Pick up the private key for decrypting the BSN from the mock DigiD issuer from the .gitignore'd settings file
    let bsn_privkey = Settings::new().unwrap().issuer.digid.bsn_privkey.clone();

    let keypair = KeyPair {
        private_key: issuer_privkey.to_pkcs8_der().unwrap().to_bytes().to_vec().into(),
        certificate: issuer_cert.as_bytes().to_vec().into(),
    };

    let mut settings = Settings {
        wallet_server: Server {
            ip: IpAddr::from_str("127.0.0.1").unwrap(),
            port,
        },
        requester_server: Server {
            ip: IpAddr::from_str("127.0.0.1").unwrap(),
            port: port2,
        },
        public_url: format!("http://localhost:{}/", port).parse().unwrap(),
        internal_url: format!("http://localhost:{}/", port2).parse().unwrap(),
        usecases: HashMap::new(),
        trust_anchors: Vec::new(),
        #[cfg(feature = "postgres")]
        store_url: "postgres://postgres@127.0.0.1:5432/wallet_server".parse().unwrap(),
        #[cfg(not(feature = "postgres"))]
        store_url: "memory://".parse().unwrap(),
        issuer: Issuer {
            wallet_client_ids: vec![NL_WALLET_CLIENT_ID.to_string()],
            digid: Digid {
                issuer_url: "https://localhost:8006/".parse().unwrap(),
                client_id: "3e58016e-bc2e-40d5-b4b1-a3e25f6193b9".to_string(),
                bsn_privkey,
            },
            private_keys: HashMap::from([
                ("com.example.pid".to_string(), keypair.clone()),
                ("com.example.address".to_string(), keypair),
            ]),
        },
    };

    let (rp_cert, rp_privkey) = Certificate::new(
        &issuance_ca,
        &issuance_ca_privkey,
        "cert.example.com",
        CertificateType::ReaderAuth(Box::new(get_my_reader_auth()).into()),
    )
    .unwrap();

    settings.usecases.insert(
        "example_usecase".to_owned(),
        KeyPair {
            certificate: rp_cert.as_bytes().to_vec().into(),
            private_key: rp_privkey
                .to_pkcs8_der()
                .expect("could not serialize private key")
                .as_bytes()
                .to_vec()
                .into(),
        },
    );

    (settings, issuance_ca)
}

struct MockAttributeService;

#[async_trait]
impl AttributeService for MockAttributeService {
    type Error = wallet_server::verifier::Error; // arbitrary type that implements the required trait bounds
    type Settings = ();

    async fn new(_settings: &Self::Settings) -> Result<Self, Self::Error> {
        Ok(MockAttributeService)
    }

    async fn attributes(
        &self,
        _session: &SessionState<Created>,
        _token_request: TokenRequest,
    ) -> Result<Vec<UnsignedMdoc>, Self::Error> {
        let mock_bsn = MockBsnLookup::default().bsn("access_token").await.unwrap();
        Ok(MockAttributesLookup::default().attributes(&mock_bsn))
    }
}

async fn start_wallet_server<A, S>(settings: Settings, sessions: S, attr_service: A)
where
    A: AttributeService,
    S: SessionStore<Data = SessionState<DisclosureData>> + Send + Sync + 'static,
{
    let public_url = settings.public_url.clone();
    tokio::spawn(async move {
        server::serve::<A, S>(&settings, sessions, attr_service)
            .await
            .expect("Could not start wallet_server");
    });

    let _ = tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .finish(),
    );

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
    let (settings, _) = wallet_server_settings();
    let sessions = new_session_store(settings.store_url.clone()).await.unwrap();

    start_wallet_server(settings.clone(), sessions, MockAttributeService).await;

    let client = reqwest::Client::new();

    let start_request = StartDisclosureRequest {
        usecase: "example_usecase".to_owned(),
        session_type: SessionType::SameDevice,
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
        }]
        .into(),
    };
    let response = client
        .post(
            settings
                .internal_url
                .join("/disclosure/sessions")
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
    let (settings, _) = wallet_server_settings();
    let sessions = new_session_store(settings.store_url.clone()).await.unwrap();

    start_wallet_server(settings.clone(), sessions, MockAttributeService).await;

    let client = reqwest::Client::new();
    // does it exist for the RP side of things?
    let response = client
        .get(
            settings
                .internal_url
                .join(&format!("/disclosure/sessions/{}/status", SessionToken::new()))
                .unwrap(),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    // does it exist for the wallet side of things?
    let response = client
        .post(
            settings
                .public_url
                .join(&format!("/disclosure/{}", SessionToken::new()))
                .unwrap(),
        )
        // .json(...) // there's no way to construct a valid body here
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_mock_issuance() {
    let (settings, issuance_ca) = wallet_server_settings();
    let sessions = new_session_store(settings.store_url.clone()).await.unwrap();
    let attr_service = MockAttributeService::new(&()).await.unwrap();

    start_wallet_server(settings.clone(), sessions, attr_service).await;

    // Setup a mock DigiD session from which the issuer client gets its token request
    let digid_session = {
        let mut digid_session = MockDigidSession::new();
        digid_session
            .expect_into_pre_authorized_code_request()
            .return_once(|code| TokenRequest {
                grant_type: openid4vc::token::TokenRequestGrantType::PreAuthorizedCode {
                    pre_authorized_code: code,
                },
                code_verifier: Some("my_code_verifier".to_string()),
                client_id: Some("my_client_id".to_string()),
                redirect_uri: Some("redirect://here".parse().unwrap()),
            });
        digid_session
    };

    // Exchange the authorization code for an access token and the attestation previews
    let mut pid_issuer_client = HttpOpenidPidIssuerClient::default();
    pid_issuer_client
        .start_retrieve_pid(
            digid_session,
            &local_base_url(settings.public_url.port().unwrap())
                .join("issuance/")
                .unwrap(),
            "authorization_code_that_digid_would_pass_to_us".to_string(),
        )
        .await
        .unwrap();

    // Accept the attestations and finish issuance
    let key_factory = SoftwareKeyFactory::default();
    let trust_anchor = DerTrustAnchor::from_der(issuance_ca.as_bytes().to_vec()).unwrap();
    let mdocs = pid_issuer_client
        .accept_pid(
            &[(&trust_anchor.owned_trust_anchor).into()],
            &key_factory,
            &settings.public_url,
        )
        .await
        .unwrap();

    assert_eq!(2, mdocs.len());
    assert_eq!(2, mdocs[0].cred_copies.len())
}

#[tokio::test]
#[cfg_attr(not(feature = "digid_test"), ignore)]
async fn test_pid_issuance_digid_bridge() {
    let (settings, issuance_ca) = wallet_server_settings();
    let sessions = new_session_store(settings.store_url.clone()).await.unwrap();
    let attr_service = PidAttributeService::new(&settings.issuer.digid).await.unwrap();
    start_wallet_server(settings.clone(), sessions, attr_service).await;

    let pid_issuance_config = &PidIssuanceConfiguration {
        pid_issuer_url: local_base_url(settings.public_url.port().unwrap()),
        digid_url: settings.issuer.digid.issuer_url.clone(),
        digid_client_id: settings.issuer.digid.client_id.clone(),
        digid_redirect_path: "authentication".to_string(),
    };

    let digid_session = HttpDigidSession::<HttpOpenIdClient, S256PkcePair>::start(
        pid_issuance_config.digid_url.clone(),
        pid_issuance_config.digid_client_id.to_string(),
        pid_issuance_config.digid_redirect_uri().unwrap(),
    )
    .await
    .unwrap();

    // Prepare DigiD flow
    let authorization_url = digid_session.auth_url();

    // Do fake DigiD authentication and parse the access token out of the redirect URL
    let redirect_url = fake_digid_auth(&authorization_url, &default_configuration().pid_issuance.digid_url).await;

    let authorization_code = digid_session.get_authorization_code(&redirect_url).unwrap();

    // Exchange the authorization code for an access token and the attestation previews
    let mut pid_issuer_client = HttpOpenidPidIssuerClient::default();
    pid_issuer_client
        .start_retrieve_pid(
            digid_session,
            &pid_issuance_config.pid_issuer_url.join("issuance/").unwrap(),
            authorization_code,
        )
        .await
        .unwrap();

    let key_factory = SoftwareKeyFactory::default();

    let trust_anchor = DerTrustAnchor::from_der(issuance_ca.as_bytes().to_vec()).unwrap();

    let mdocs = pid_issuer_client
        .accept_pid(
            &[(&trust_anchor.owned_trust_anchor).into()],
            &key_factory,
            &settings.public_url,
        )
        .await
        .unwrap();

    assert_eq!(2, mdocs.len());
    assert_eq!(2, mdocs[0].cred_copies.len())
}

// Use the mock flow of the DigiD bridge to simulate a DigiD login,
// invoking the same URLs at the DigiD bridge that would normally be invoked by the app and browser in the mock
// flow of the DigiD bridge.
// Note that this depends of part of the internal API of the DigiD bridge, so it may break when the bridge
// is updated.
async fn fake_digid_auth(authorization_url: &Url, digid_base_url: &Url) -> Url {
    let client_builder = Client::builder();
    #[cfg(feature = "disable_tls_validation")]
    let client_builder = client_builder.danger_accept_invalid_certs(true);
    let client = client_builder.build().unwrap();

    // Avoid the DigiD/mock DigiD landing page of the DigiD bridge by preselecting the latter
    let authorization_url = authorization_url.to_string() + "&login_hint=digid_mock";

    // Start authentication by GETting the authorization URL.
    // In the resulting HTML page, find the "RelayState" parameter which we need for the following URL.
    let relay_state_page = get_text(&client, authorization_url).await;
    let relay_state_line = relay_state_page
        .lines()
        .find(|l| l.contains("RelayState"))
        .expect("failed to find RelayState");
    let relay_state = find_in_text(relay_state_line, "value=\"", "\"");

    // Note: the above HTTP response contains a HTML form that is normally automatically submitted
    // by the browser, leading to a page that contains the link that we invoke below.
    // To actually simulate autosubmitting that form and running some related JavaScript would be a bit of a hassle,
    // so here we skip autosubmitting that form. Turns out the DigiD bridge is fine with this.

    // Get the HTML page containing the redirect_uri back to our own app
    let finish_digid_url = format!(
        "{}acs?SAMLart=999991772&RelayState={}&mocking=1",
        digid_base_url, relay_state
    );
    let redirect_page = get_text(&client, finish_digid_url).await;
    let redirect_url = find_in_text(&redirect_page, "url=", "\"");

    Url::parse(redirect_url).expect("failed to parse redirect url")
}

async fn get_text(client: &reqwest::Client, url: String) -> String {
    client
        .get(url)
        .send()
        .await
        .expect("failed to GET URL")
        .text()
        .await
        .expect("failed to get body text")
}

fn find_in_text<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = text.find(start).expect("start not found");
    let remaining = &text[start_index + start.len()..];
    let end_index = remaining.find(end).expect("end not found");
    &remaining[..end_index]
}

fn local_base_url(port: u16) -> Url {
    Url::parse(&format!("http://localhost:{}/", port)).expect("Could not create url")
}
