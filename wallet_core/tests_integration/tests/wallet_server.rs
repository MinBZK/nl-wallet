use base64::prelude::*;
use indexmap::IndexMap;
use openid4vc::issuance_client::{HttpIssuerClient, HttpOpenidMessageClient, IssuerClient};
use reqwest::{Client, StatusCode};
use url::Url;

use nl_wallet_mdoc::{
    holder::TrustAnchor, mock::SoftwareKeyFactory, server_state::SessionToken, utils::serialization::cbor_deserialize,
    verifier::SessionType, ItemsRequest, ReaderEngagement,
};
use wallet::{
    mock::default_configuration,
    wallet_deps::{DigidSession, HttpDigidSession, HttpOpenIdClient, S256PkcePair},
    WalletConfiguration,
};
use wallet_common::config::wallet_config::PidIssuanceConfiguration;
use wallet_server::{
    pid::attributes::{reqwest_client, MockPidAttributeService},
    settings::Settings,
    store::SessionStores,
    verifier::{StartDisclosureRequest, StartDisclosureResponse},
};

use crate::common::*;

pub mod common;

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
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_start_session() {
    let settings = common::wallet_server_settings();
    let sessions = SessionStores::init(settings.store_url.clone()).await.unwrap();

    start_wallet_server(settings.clone(), sessions, MockAttributeService).await;

    let client = reqwest::Client::new();

    let start_request = StartDisclosureRequest {
        usecase: "driving_license".to_owned(),
        session_type: SessionType::SameDevice,
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
        return_url_template: None,
    };
    let response = client
        .post(
            settings
                .internal_url
                .join("disclosure/sessions")
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
        ..
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
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn test_session_not_found() {
    let settings = common::wallet_server_settings();
    let sessions = SessionStores::init(settings.store_url.clone()).await.unwrap();

    start_wallet_server(settings.clone(), sessions, MockAttributeService).await;

    let client = reqwest::Client::new();
    // does it exist for the RP side of things?
    let response = client
        .get(
            settings
                .internal_url
                .join(&format!("/disclosure/{}/status", SessionToken::new()))
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
#[cfg_attr(not(feature = "digid_test"), ignore)]
async fn test_pid_issuance_digid_bridge() {
    let (settings, sessions) = issuance_settings_and_sessions().await;

    let attr_service = MockPidAttributeService::new(&settings.issuer).await.unwrap();
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
    let token_request = digid_session.into_token_request(&redirect_url).unwrap();

    let server_url = local_base_url(settings.public_url.port().unwrap())
        .join("issuance/")
        .unwrap();

    // Start issuance by exchanging the authorization code for the attestation previews
    let (pid_issuer_client, _) = HttpIssuerClient::start_issuance(
        HttpOpenidMessageClient::new(reqwest_client()),
        &server_url,
        token_request,
    )
    .await
    .unwrap();

    let mdocs = pid_issuer_client
        .accept_issuance(
            &trust_anchors(&default_configuration()),
            SoftwareKeyFactory::default(),
            &server_url,
        )
        .await
        .unwrap();

    assert_eq!(2, mdocs.len());
    assert_eq!(2, mdocs[0].cred_copies.len())
}

async fn issuance_settings_and_sessions() -> (Settings, SessionStores) {
    let settings = common::wallet_server_settings();
    #[cfg(feature = "db_test")]
    let store_url = settings.store_url.clone();
    #[cfg(not(feature = "db_test"))]
    let store_url = "memory://".parse().unwrap();

    (settings, SessionStores::init(store_url).await.unwrap())
}

fn trust_anchors(wallet_conf: &WalletConfiguration) -> Vec<TrustAnchor<'_>> {
    wallet_conf
        .mdoc_trust_anchors
        .iter()
        .map(|a| (&a.owned_trust_anchor).into())
        .collect()
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
