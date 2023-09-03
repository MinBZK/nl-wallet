use std::{
    net::{IpAddr, TcpListener},
    str::FromStr,
    sync::Arc,
};

use tokio::sync::Mutex;
use tracing_subscriber::FmtSubscriber;
use url::Url;

use nl_wallet_mdoc::{
    holder::{CborHttpClient, Wallet as MdocWallet},
    utils::mdocs_map::MdocsMap,
};
use platform_support::utils::software::SoftwareUtilities;
use wallet::{
    mock::{MockConfigurationRepository, MockDigidAuthenticator, MockStorage},
    wallet::{Configuration, DigidAuthenticator, Wallet},
    wallet_deps::{DigidClient, PidIssuerClient, RemoteAccountServerClient},
};
use wallet_common::keys::software::SoftwareEcdsaKey;

use pid_issuer::{
    app::{
        mock::{MockAttributesLookup, MockBsnLookup},
        AttributesLookup, BsnLookup,
    },
    digid::OpenIdClient,
    server,
    settings::Settings,
};

fn local_base_url(port: u16) -> Url {
    Url::parse(&format!("http://localhost:{}/", port)).expect("Could not create url")
}

fn test_wallet_config(base_url: Url) -> MockConfigurationRepository {
    let mut config = MockConfigurationRepository::default();
    config.0.pid_issuance.pid_issuer_url = base_url;
    config
}

/// Create an instance of [`Wallet`].
async fn create_test_wallet<D: DigidAuthenticator>(
    base_url: Url,
    digid_client: D,
    pid_issuer_client: PidIssuerClient,
) -> (
    Configuration,
    Wallet<MockConfigurationRepository, RemoteAccountServerClient, MockStorage, SoftwareEcdsaKey, D, PidIssuerClient>,
) {
    let wallet = Wallet::init_wp_and_storage::<SoftwareUtilities>(
        test_wallet_config(base_url.clone()),
        digid_client,
        pid_issuer_client,
    )
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

fn pid_issuer_settings() -> (Settings, u16) {
    let port = find_listener_port();

    let mut settings = Settings::new().expect("Could not read settings");
    settings.webserver.ip = IpAddr::from_str("127.0.0.1").expect("Could not parse IP address");
    settings.webserver.port = port;
    settings.public_url = format!("http://localhost:{}/", port).parse().unwrap();

    (settings, port)
}

fn start_pid_issuer<A, B>(settings: Settings, attributes_lookup: A, bsn_lookup: B)
where
    A: AttributesLookup + Send + Sync + 'static,
    B: BsnLookup + Send + Sync + 'static,
{
    tokio::spawn(async {
        server::serve::<A, B>(settings, attributes_lookup, bsn_lookup)
            .await
            .expect("Could not start server")
    });

    let _ = tracing::subscriber::set_global_default(FmtSubscriber::new());
}

#[tokio::test]
async fn test_pid_issuance_mock_bsn() {
    // Set up pid issuer.
    let (settings, port) = pid_issuer_settings();
    start_pid_issuer(settings, MockAttributesLookup, MockBsnLookup);

    // Set up mock DigiD client and its responses.
    let digid_client = {
        let mut digid_client = MockDigidAuthenticator::new();

        // Return a mock access token from the mock DigiD client that the `MockBsnLookup` always accepts.
        digid_client
            .expect_get_access_token()
            .returning(|_| Ok("mock_token".to_string()));

        digid_client
    };

    // Set up real pid issuer client.
    let client = CborHttpClient(reqwest::Client::new());
    let mdoc_wallet = Arc::new(Mutex::new(MdocWallet::new(MdocsMap::new(), client)));
    let pid_issuer_client = PidIssuerClient::new(Arc::clone(&mdoc_wallet));

    // Create wallet with configuration and dependencies.
    let (_, mut wallet) = create_test_wallet(local_base_url(port), digid_client, pid_issuer_client).await;

    wallet
        .continue_pid_issuance(&Url::parse("redirect://here").unwrap())
        .await
        .expect("PID issuance failed");

    let mdocs = mdoc_wallet.lock().await.list_mdocs::<SoftwareEcdsaKey>();
    dbg!(&mdocs);

    let pid_mdocs = mdocs.first().unwrap().1;
    let namespace = pid_mdocs.first().unwrap();
    let attrs = namespace.first().unwrap().1;
    assert!(!attrs.is_empty());
}

// This test connects to the DigiD bridge and is disabled by default.
// Enable the `digid_test` feature to include it.
#[tokio::test]
#[cfg_attr(not(feature = "digid_test"), ignore)]
async fn test_pid_issuance_digid_bridge() {
    let (settings, port) = pid_issuer_settings();
    let bsn_lookup = OpenIdClient::new(&settings.digid).await.unwrap();
    start_pid_issuer(settings, MockAttributesLookup, bsn_lookup);
    let (config, mut wallet) =
        create_test_wallet::<DigidClient>(local_base_url(port), DigidClient::default(), PidIssuerClient::default())
            .await;

    // Prepare DigiD flow
    let authorization_url = wallet
        .create_pid_issuance_redirect_uri()
        .await
        .expect("failed to get digid url");

    // Do fake DigiD authentication and parse the access token out of the redirect URL
    let redirect_url = fake_digid_auth(&authorization_url, &config.pid_issuance.digid_url).await;

    // Use the redirect URL to do PID issuance
    wallet
        .continue_pid_issuance(&redirect_url)
        .await
        .expect("PID issuance failed");
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
