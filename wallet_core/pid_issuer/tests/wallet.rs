use std::{
    net::{IpAddr, TcpListener},
    str::FromStr,
};

use async_trait::async_trait;
use tracing_subscriber::FmtSubscriber;
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::RequestKeyGenerationMessage,
    holder::{cbor_http_client_builder, IssuanceUserConsent, Wallet as MdocWallet},
    utils::mdocs_map::MdocsMap,
    ServiceEngagement,
};
use platform_support::utils::software::SoftwareUtilities;
use wallet::{
    mock::{MockConfigurationRepository, MockStorage, RemoteAccountServerClient},
    wallet::{Configuration, Wallet},
};
use wallet_common::keys::software::SoftwareEcdsaKey;

use pid_issuer::{
    app::{
        mock::{MockAttributesLookup, MockBsnLookup},
        AttributesLookup, BsnLookup,
    },
    digid::OpenIdClient as DigidClient,
    server,
    settings::Settings,
};

fn local_base_url(port: u16) -> Url {
    Url::parse(&format!("http://localhost:{}/", port)).expect("Could not create url")
}

fn test_wallet_config(base_url: Url) -> MockConfigurationRepository {
    let mut config = MockConfigurationRepository::default();
    config.0.digid.pid_issuer_url = base_url;
    config
}

/// Create an instance of [`Wallet`].
async fn create_test_wallet(
    base_url: Url,
) -> (
    Configuration,
    Wallet<MockConfigurationRepository, RemoteAccountServerClient, MockStorage, SoftwareEcdsaKey>,
) {
    let wallet = Wallet::init::<SoftwareUtilities>(test_wallet_config(base_url.clone()))
        .await
        .expect("Could not create test wallet");
    (test_wallet_config(base_url).0, wallet)
}

fn find_listener_port() -> u16 {
    TcpListener::bind("localhost:0")
        .expect("Could not find TCP port")
        .local_addr()
        .expect("Could not get local address from TCP listener")
        .port()
}

fn start_pid_issuer<A, B>() -> u16
where
    A: AttributesLookup + Send + Sync + 'static,
    B: BsnLookup + Send + Sync + 'static,
{
    let port = find_listener_port();

    let mut settings = Settings::new().expect("Could not read settings");
    settings.webserver.ip = IpAddr::from_str("127.0.0.1").expect("Could not parse IP address");
    settings.webserver.port = port;
    settings.public_url = format!("http://localhost:{}/", port).parse().unwrap();

    tokio::spawn(async { server::serve::<A, B>(settings).await.expect("Could not start server") });

    let _ = tracing::subscriber::set_global_default(FmtSubscriber::new());

    port
}

#[tokio::test]
async fn test_pid_issuance_mock_bsn() {
    let port = start_pid_issuer::<MockAttributesLookup, MockBsnLookup>();
    let config = test_wallet_config(local_base_url(port)).0;

    // Start the PID issuance session, sending a mock access token which the `MockBsnLookup` always accepts
    let service_engagement: ServiceEngagement = reqwest::Client::new()
        .post(local_base_url(port).join("start").unwrap())
        .bearer_auth("mock_token")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // We can't use the actual `Wallet` yet because it can't yet mock the DigiD authentication,
    // so for now we directly use the mdoc issuance function. TODO use `Wallet` when possible.
    let mdoc_wallet = MdocWallet::new(MdocsMap::new());
    mdoc_wallet
        .do_issuance::<SoftwareEcdsaKey>(
            service_engagement,
            &always_agree(),
            &cbor_http_client_builder(),
            &config.mdoc_trust_anchors,
        )
        .await
        .unwrap();

    let mdocs = mdoc_wallet.list_mdocs::<SoftwareEcdsaKey>();
    dbg!(&mdocs);

    let pid_mdocs = mdocs.first().unwrap().1;
    let namespace = pid_mdocs.first().unwrap();
    let attrs = namespace.first().unwrap().1;
    assert!(!attrs.is_empty());
}

fn always_agree() -> impl IssuanceUserConsent {
    struct AlwaysAgree;
    #[async_trait]
    impl IssuanceUserConsent for AlwaysAgree {
        async fn ask(&self, _: &RequestKeyGenerationMessage) -> bool {
            true
        }
    }
    AlwaysAgree
}

// This test connects to the DigiD bridge and is disabled by default.
// Enable the `live_test` feature to include it.
#[tokio::test]
#[cfg_attr(not(feature = "digid_test"), ignore)]
async fn test_pid_issuance_digid_bridge() {
    let port = start_pid_issuer::<MockAttributesLookup, DigidClient>();
    let (config, wallet) = create_test_wallet(local_base_url(port)).await;

    // Prepare DigiD flow
    let mut connector = wallet
        .digid_connector()
        .await
        .expect("failed to get digid connector")
        .lock()
        .await;
    let authorization_url = connector
        .get_digid_authorization_url()
        .expect("failed to get digid url");

    // Do fake DigiD authentication and parse the access token out of the redirect URL
    let redirect_url = fake_digid_auth(&authorization_url, &config.digid.digid_url).await;

    // Consume the redirect URL and do PID issuance
    connector.issue_pid(redirect_url).await.expect("PID issuance failed");
}

// Use the mock flow of the DigiD bridge to simulate a DigiD login,
// invoking the same URLs at the DigiD bridge that would normally be invoked by the app and browser in the mock
// flow of the DigiD bridge.
// Note that this depends of part of the internal API of the DigiD bridge, so it may break when the bridge
// is updated.
async fn fake_digid_auth(authorization_url: &Url, digid_base_url: &Url) -> Url {
    let client = reqwest::Client::new();

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
        "{}/acs?SAMLart=999991772&RelayState={}&mocking=1",
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
