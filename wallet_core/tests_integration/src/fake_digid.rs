use http_utils::reqwest::HttpJsonClient;
use http_utils::reqwest::tls_reqwest_client_builder;
use http_utils::urls::BaseUrl;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::metadata::oauth_metadata::AuthorizationServerMetadata;
use openid4vc::metadata::oauth_metadata::OidcProviderMetadata;
use openid4vc::metadata::well_known::WellKnownPath;
use openid4vc::metadata::well_known::fetch_well_known;
use reqwest::Certificate;
use reqwest::Response;
use reqwest::header::LOCATION;
use reqwest::redirect::Policy;
use tracing::debug;
use url::Url;

// Use the mock flow of the DigiD bridge to simulate a DigiD login,
// invoking the same URLs at the DigiD bridge that would normally be invoked by the app and browser in the mock
// flow of the DigiD bridge.
// Note that this depends of part of the internal API of the DigiD bridge, so it may break when the bridge
// is updated.
pub async fn fake_digid_auth(
    original_authorization_url: Url,
    issuer_identifier: &IssuerIdentifier,
    digid_url: &str,
    digid_trust_anchors: Vec<Certificate>,
    digid_client_id: &str,
    bsn: &str,
) -> Url {
    let http_client = tls_reqwest_client_builder(digid_trust_anchors.clone())
        .redirect(Policy::none())
        .build()
        .unwrap();

    // TODO (PVW-5612): remove https_only(false) once the PID issuer runs on HTTPS.
    let well_known_client =
        HttpJsonClient::try_new(tls_reqwest_client_builder(digid_trust_anchors).https_only(false)).unwrap();

    // Fetch the credential issuer metadata in order to retrieve the original authorization endpoint.
    let oauth_metadata: AuthorizationServerMetadata = fetch_well_known(
        &well_known_client,
        issuer_identifier,
        WellKnownPath::OauthAuthorizationServer,
    )
    .await
    .unwrap();
    let original_auth_endpoint = oauth_metadata.authorization_endpoint.unwrap();

    // Fetch the DigiD metadata in order to retrieve the DigiD authorization endpoint.
    let digid_identifier = IssuerIdentifier::try_new(String::from(digid_url)).unwrap();
    let oidc_metadata: OidcProviderMetadata = fetch_well_known(
        &well_known_client,
        &digid_identifier,
        WellKnownPath::OpenidConfiguration,
    )
    .await
    .unwrap();
    let digid_auth_endpoint = oidc_metadata.authorization_endpoint.unwrap();

    // Replace the original authorization endpoint with the DigiD authorization endpoint.
    let mut authorization_url = Url::parse(
        &original_authorization_url
            .as_str()
            .replace(original_auth_endpoint.as_str(), digid_auth_endpoint.as_str()),
    )
    .unwrap();

    // Replace the client_id query parameter
    let query_params: Vec<(String, String)> = authorization_url
        .query_pairs()
        .filter(|(name, _)| name != "client_id")
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    authorization_url
        .query_pairs_mut()
        .clear()
        .extend_pairs(query_params)
        .append_pair("client_id", digid_client_id);

    // Avoid the DigiD/mock DigiD landing page of the DigiD bridge by preselecting the latter
    authorization_url
        .query_pairs_mut()
        .append_pair("login_hint", "digid_mock");

    debug!("original_authorization_url: {}", original_authorization_url);
    debug!("digid base url: {}", digid_url);
    debug!("rewritten authorization_url: {}", authorization_url.to_string());

    // Start authentication by GETting the authorization URL.
    // In the resulting HTML page, find the "RelayState" parameter which we need for the following URL.
    let relay_state_page = do_get_as_text(&http_client, authorization_url).await;

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
    let finish_digid_path = format!("acs?SAMLart={bsn}&RelayState={relay_state}&mocking=1");

    let digid_base_url: BaseUrl = digid_url.parse::<Url>().unwrap().try_into().unwrap();
    let response = do_get_request(&http_client, digid_base_url.join(&finish_digid_path)).await;
    let redirect_url = response.headers().get(LOCATION).unwrap().to_str().unwrap();

    Url::parse(redirect_url).expect("failed to parse redirect url")
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
