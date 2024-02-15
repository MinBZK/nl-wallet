use openid4vc::{
    issuance_client::{HttpIssuerClient, HttpOpenidMessageClient, IssuerClient},
    pkce::S256PkcePair,
};
use reqwest::Client;
use url::Url;

use nl_wallet_mdoc::{holder::TrustAnchor, software_key_factory::SoftwareKeyFactory};
use wallet::{
    mock::default_configuration,
    wallet_deps::{DigidSession, HttpDigidSession, HttpOpenIdClient},
    WalletConfiguration,
};
use wallet_common::config::wallet_config::{PidIssuanceConfiguration, ISSUANCE_REDIRECT_URI};

use crate::common::*;

pub mod common;

#[tokio::test]
#[cfg_attr(not(feature = "digid_test"), ignore)]
async fn test_pid_issuance_digid_bridge() {
    let settings = common::wallet_server_settings();
    start_wallet_server(settings.clone()).await;

    let pid_issuance_config = &PidIssuanceConfiguration {
        pid_issuer_url: local_base_url(settings.public_url.port().unwrap()),
        digid_url: settings.issuer.digid.issuer_url.clone(),
        digid_client_id: settings.issuer.digid.client_id.clone(),
    };

    let digid_session = HttpDigidSession::<HttpOpenIdClient, S256PkcePair>::start(
        pid_issuance_config.digid_url.clone(),
        pid_issuance_config.digid_client_id.to_string(),
        ISSUANCE_REDIRECT_URI.to_owned(),
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
        HttpOpenidMessageClient::new(reqwest::Client::new()),
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
