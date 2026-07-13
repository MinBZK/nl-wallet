use http_utils::reqwest::tls_reqwest_client_builder;
use pid_issuer::pid::digid_mock::drive_mock_digid_login;
use reqwest::Certificate;
use reqwest::Response;
use reqwest::header;
use reqwest::redirect::Policy;
use tracing::debug;
use url::Url;

// Use the mock flow of the DigiD bridge to simulate a DigiD login, invoking the same URLs at the DigiD bridge that
// would normally be invoked by the app and browser in the mock flow of the DigiD bridge.
// Note that this depends of part of the internal API of the DigiD bridge, so it may break when the bridge
// is updated.
//
// The core of driving the bridge's mock login (scraping `RelayState`, hitting `/acs`) is shared with
// the pid_issuer-hosted mock login page via [`drive_mock_digid_login`]; this function wraps it with the
// PAR hop in front and the callback hop after, both of which are specific to driving the flow headlessly.
pub async fn fake_digid_auth(authorization_url: Url, digid_trust_anchors: Vec<Certificate>, bsn: &str) -> Url {
    // TODO (PVW-5612): remove https_only(false) once the PID issuer runs on HTTPS.
    let http_client = tls_reqwest_client_builder(digid_trust_anchors)
        .https_only(false)
        .redirect(Policy::none())
        .build()
        .unwrap();

    // Follow the PAR redirect to the DigiD authorization endpoint.
    let redirect_auth_url = http_client.get(authorization_url.clone()).send().await.unwrap();
    let digid_auth_url = redirect_auth_url
        .headers()
        .get(header::LOCATION)
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<Url>()
        .unwrap();

    // When the pid_issuer is configured to serve its own mock login page, it redirects here instead
    // of straight to nl-rdo-max, carrying the real nl-rdo-max authorize URL as the `authorize_url`
    // query parameter. Unwrap that so this headless driver behaves the same whether or not the mock
    // login page is enabled.
    let digid_auth_url = digid_auth_url
        .query_pairs()
        .find(|(key, _)| key == "authorize_url")
        .map(|(_, value)| {
            value
                .parse::<Url>()
                .expect("mock login authorize_url should be a valid URL")
        })
        .unwrap_or(digid_auth_url);

    debug!("authorization_url: {}", authorization_url);
    debug!("digid authorization_url: {}", digid_auth_url);

    // Drive the DigiD bridge's mock login (shared with the pid_issuer mock login page): scrape the
    // `RelayState` from the mock SAML page and hit the mock `/acs` endpoint. The result is the issuer's
    // `/digid/callback` URL that nl-rdo-max redirects to.
    let mut issuer_callback_url = drive_mock_digid_login(&http_client, digid_auth_url, bsn).await.unwrap();

    // nl-rdo-max validated and echoed back the issuer's fixed, pre-registered callback URL.
    // Substitute the host and port in that callback url with those of the actual running pid_issuer.
    issuer_callback_url
        .set_host(authorization_url.host_str())
        .expect("authorization_url should have a host");
    issuer_callback_url
        .set_port(authorization_url.port())
        .expect("should be able to set port on issuer callback url");

    // Follow that one extra hop: the issuer's `/digid/callback` exchanges the upstream code for
    // the BSN, queries the BRP, generates its own authorization code, writes the `AuthCodeIssued`
    // session, and 302s back to the wallet's redirect_uri. The resulting Location is the
    // wallet-facing URL (carrying the issuer-minted code + the wallet's state) that
    // `start_issuance` expects.
    let callback_response = do_get_request(&http_client, issuer_callback_url).await;
    callback_response
        .headers()
        .get(header::LOCATION)
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .expect("failed to parse redirect url")
}

async fn do_get_request(client: &reqwest::Client, url: Url) -> Response {
    client.get(url).send().await.expect("failed to GET URL")
}
