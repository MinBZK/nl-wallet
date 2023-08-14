use std::{
    net::{IpAddr, TcpListener},
    str::FromStr,
};

use pid_issuer::{application::mock::MockAttributesLookup, server, settings::Settings};
use reqwest::Client;
use tracing_subscriber::FmtSubscriber;
use url::Url;

use platform_support::utils::software::SoftwareUtilities;
use wallet::{
    mock::{MockConfigurationRepository, MockStorage, RemoteAccountServerClient},
    wallet::{Configuration, Wallet},
};
use wallet_common::keys::software::SoftwareEcdsaKey;

fn local_base_url(port: u16) -> Url {
    Url::parse(&format!("http://localhost:{}/", port)).expect("Could not create url")
}

/// Create an instance of [`Wallet`].
async fn create_test_wallet(
    base_url: Url,
) -> (
    Configuration,
    Wallet<MockConfigurationRepository, RemoteAccountServerClient, MockStorage, SoftwareEcdsaKey>,
) {
    // Create mock Wallet from settings. We want to pass an owned version of this to the wallet
    // as wel as to the caller, but MockConfigurationRepository doesn't and can't inplement `Clone`.
    // So use a closure that returns a fresh one.
    let config = || {
        let mut config = MockConfigurationRepository::default();
        config.0.digid.pid_issuer_url = base_url.clone();
        config
    };

    let wallet = Wallet::init::<SoftwareUtilities>(config())
        .await
        .expect("Could not create test wallet");
    (config().0, wallet)
}

fn find_listener_port() -> u16 {
    TcpListener::bind("localhost:0")
        .expect("Could not find TCP port")
        .local_addr()
        .expect("Could not get local address from TCP listener")
        .port()
}

fn start_pid_issuer() -> u16 {
    let port = find_listener_port();

    let mut settings = Settings::new().expect("Could not read settings");
    settings.webserver.ip = IpAddr::from_str("127.0.0.1").expect("Could not parse IP address");
    settings.webserver.port = port;
    settings.public_url = format!("http://localhost:{}/mdoc/", port).parse().unwrap();

    tokio::spawn(async {
        server::serve::<MockAttributesLookup>(settings)
            .await
            .expect("Could not start server")
    });

    let _ = tracing::subscriber::set_global_default(FmtSubscriber::new());

    port
}

#[tokio::test]
async fn test_pid_issuance() {
    let port = start_pid_issuer();
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
    let client = Client::new();

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

async fn get_text(client: &Client, url: String) -> String {
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
