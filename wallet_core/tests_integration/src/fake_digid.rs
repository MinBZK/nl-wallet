use http::Method;
use reqwest::header::LOCATION;
use reqwest::redirect::Policy;
use reqwest::Response;
use url::Url;

use http_utils::reqwest::IntoPinnedReqwestClient;
use http_utils::reqwest::PinnedReqwestClient;
use http_utils::reqwest::ReqwestClientUrl;
use http_utils::tls::pinning::TlsPinningConfig;

// Use the mock flow of the DigiD bridge to simulate a DigiD login,
// invoking the same URLs at the DigiD bridge that would normally be invoked by the app and browser in the mock
// flow of the DigiD bridge.
// Note that this depends of part of the internal API of the DigiD bridge, so it may break when the bridge
// is updated.
pub async fn fake_digid_auth(mut authorization_url: Url, digid_http_config: TlsPinningConfig, bsn: &str) -> Url {
    let client = digid_http_config
        .try_into_custom_client(|client_builder| client_builder.redirect(Policy::none()))
        .unwrap();

    // Avoid the DigiD/mock DigiD landing page of the DigiD bridge by preselecting the latter
    authorization_url
        .query_pairs_mut()
        .append_pair("login_hint", "digid_mock");

    // Start authentication by GETting the authorization URL.
    // In the resulting HTML page, find the "RelayState" parameter which we need for the following URL.
    let relay_state_page = do_get_as_text(&client, ReqwestClientUrl::Absolute(authorization_url)).await;

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
    let finish_digid_path = format!("acs?SAMLart={}&RelayState={}&mocking=1", bsn, relay_state);

    let response = do_get_request(&client, ReqwestClientUrl::Relative(&finish_digid_path)).await;
    let redirect_url = response.headers().get(LOCATION).unwrap().to_str().unwrap();

    Url::parse(redirect_url).expect("failed to parse redirect url")
}

async fn do_get_request(client: &PinnedReqwestClient, url: ReqwestClientUrl<'_>) -> Response {
    client.send_request(Method::GET, url).await.expect("failed to GET URL")
}

async fn do_get_as_text(client: &PinnedReqwestClient, url: ReqwestClientUrl<'_>) -> String {
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
