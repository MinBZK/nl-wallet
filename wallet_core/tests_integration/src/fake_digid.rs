use http_utils::reqwest::tls_reqwest_client_builder;
use http_utils::urls::BaseUrl;
use reqwest::Certificate;
use reqwest::Response;
use reqwest::header;
use reqwest::redirect::Policy;
use tracing::debug;
use url::Url;

// Use the mock flow of the DigiD bridge to simulate a DigiD login,
// invoking the same URLs at the DigiD bridge that would normally be invoked by the app and browser in the mock
// flow of the DigiD bridge.
// Note that this depends of part of the internal API of the DigiD bridge, so it may break when the bridge
// is updated.
pub async fn fake_digid_auth(
    authorization_url: Url,
    digid_url: &str,
    digid_trust_anchors: Vec<Certificate>,
    bsn: &str,
) -> Url {
    // TODO (PVW-5612): remove https_only(false) once the PID issuer runs on HTTPS.
    let http_client = tls_reqwest_client_builder(digid_trust_anchors.clone())
        .https_only(false)
        .redirect(Policy::none())
        .build()
        .unwrap();

    // Follow the PAR redirect to the DigiD authorization endpoint.
    let redirect_auth_url = http_client.get(authorization_url.clone()).send().await.unwrap();
    let mut digid_auth_url = redirect_auth_url
        .headers()
        .get(header::LOCATION)
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<Url>()
        .unwrap();

    // Avoid the DigiD/mock DigiD landing page of the DigiD bridge by preselecting the latter
    digid_auth_url.query_pairs_mut().append_pair("login_hint", "digid_mock");

    debug!("authorization_url: {}", authorization_url);
    debug!("digid base url: {}", digid_url);
    debug!("digid authorization_url: {}", digid_auth_url.to_string());

    // Start authentication by GETting the authorization URL.
    // In the resulting HTML page, find the "RelayState" parameter which we need for the following URL.
    let relay_state_page = do_get_as_text(&http_client, digid_auth_url).await;

    let relay_state_line = relay_state_page
        .lines()
        .find(|l| l.contains("RelayState"))
        .expect("failed to find RelayState");
    let relay_state = find_in_text(relay_state_line, "value=\"", "\"");

    // Note: the above HTTP response contains a HTML form that is normally automatically submitted
    // by the browser, leading to a page that contains the link that we invoke below.
    // To actually simulate autosubmitting that form and running some related JavaScript would be a bit of a hassle,
    // so here we skip autosubmitting that form. Turns out the DigiD bridge is fine with this.

    // Get the redirect from the DigiD bridge's mock acs endpoint which points at the issuer's `/digid/callback`
    // endpoint.
    let finish_digid_path = format!("acs?SAMLart={bsn}&RelayState={relay_state}&mocking=1");

    let digid_base_url: BaseUrl = digid_url.parse::<Url>().unwrap().try_into().unwrap();
    let acs_response = do_get_request(&http_client, digid_base_url.join(&finish_digid_path)).await;
    let mut issuer_callback_url: Url = acs_response
        .headers()
        .get(header::LOCATION)
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .expect("failed to parse issuer callback url");

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

async fn do_get_as_text(client: &reqwest::Client, url: Url) -> String {
    do_get_request(client, url)
        .await
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
