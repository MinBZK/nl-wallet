//! A pid_issuer-hosted replacement for nl-rdo-max's mock DigiD login page.
//!
//! This module lets the pid_issuer serve a grid of selectable test identities (BSN + name). Selecting a card "clicks
//! the button" by driving nl-rdo-max's mock flow server-side and then hands off to the existing `/digid/callback`, so
//! nl-rdo-max stays fully in the loop and switching to real DigiD is just a matter of not configuring any mock
//! subjects.

use std::sync::Arc;

use askama::Template;
use askama_web::WebTemplate;
use axum::Router;
use axum::extract::Form;
use axum::extract::Query;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::http::header;
use axum::middleware;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Deserialize;
use url::Url;
use utils::vec_at_least::VecNonEmpty;
use web_utils::css::serve_bundled_css;
use web_utils::headers::set_content_security_policy;
use web_utils::language::LANGUAGE_JS;
use web_utils::language::LANGUAGE_SELECTOR_CHECKMARK_SVG;
use web_utils::language::LANGUAGE_SELECTOR_CSS;
use web_utils::language::LANGUAGE_SELECTOR_DOWN_SVG;
use web_utils::language::Language;

/// Path (relative to the issuer's public URL) of the mock login page.
pub const MOCK_LOGIN_PATH: &str = "/digid/mock-login";

/// Path of the per-card selection endpoint that drives the bridge and hands off to `/digid/callback`.
const MOCK_LOGIN_SELECT_PATH: &str = "/digid/mock-login/select";

/// Paths of the page's (same-origin) stylesheet and script, so a strict CSP can use `'self'` rather
/// than allowing inline styles/scripts.
const MOCK_LOGIN_CSS_PATH: &str = "/digid/mock-login/mock_login.css";
const MOCK_LOGIN_JS_PATH: &str = "/digid/mock-login/mock_login.js";
const MOCK_LOGIN_LOGO_PATH: &str = "/digid/mock-login/digid.svg";

/// Paths of the shared language selector's assets. The stylesheet refers to its icons as `../images/*.svg`,
/// so it is served from a `css/` sibling of `images/`.
const LANGUAGE_SELECTOR_CSS_PATH: &str = "/digid/mock-login/css/language_selector.css";
const LANGUAGE_SELECTOR_DOWN_PATH: &str = "/digid/mock-login/images/down.svg";
const LANGUAGE_SELECTOR_CHECKMARK_PATH: &str = "/digid/mock-login/images/checkmark.svg";
const LANGUAGE_JS_PATH: &str = "/digid/mock-login/language.js";

const MOCK_LOGIN_CSS: &str = include_str!("../../static/mock_login.css");
const MOCK_LOGIN_JS: &str = include_str!("../../static/mock_login.js");
const MOCK_LOGIN_LOGO: &str = include_str!("../../static/non-free/digid.svg");

const CSS_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("text/css");
const JS_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("text/javascript");
const SVG_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("image/svg+xml");

/// The languages offered by the language selector, each labelled in its own language.
const LANGUAGE_OPTIONS: &[(Language, &str)] = &[(Language::Nl, "Nederlands"), (Language::En, "English")];

/// The configured selectable mock identities, as a map from BSN to display name.
pub type MockSubjects = IndexMap<String, String>;

#[derive(Debug, thiserror::Error)]
pub enum MockDigidError {
    #[error("HTTP error talking to nl-rdo-max: {0}")]
    Http(#[source] reqwest::Error),

    #[error("could not find RelayState in nl-rdo-max mock page")]
    RelayStateNotFound,

    #[error("nl-rdo-max /acs did not return a Location redirect")]
    MissingCallbackRedirect,

    #[error("nl-rdo-max /acs returned an unparseable callback URL")]
    InvalidCallbackRedirect,
}

impl IntoResponse for MockDigidError {
    fn into_response(self) -> Response {
        // Everything here is an upstream (nl-rdo-max) failure, so surface it as a bad gateway.
        (StatusCode::BAD_GATEWAY, self.to_string()).into_response()
    }
}

/// Drive nl-rdo-max's mock DigiD login for a single BSN and return the URL it redirects to (the
/// issuer's `/digid/callback`, carrying the upstream `code` + `state`).
///
/// This performs the two internal steps a human triggers by clicking through the bridge's mock login
/// page:
///  1. GET the `/authorize` page (preselecting the mock IdP) and scrape its `RelayState`.
///  2. GET `/acs?SAMLart={bsn}&RelayState={relay_state}&mocking=1`.
///
/// `authorize_url` is the nl-rdo-max `/authorize` URL as built by
/// [`DigidClient::authorization_request`]; the `/acs` endpoint is derived from its origin.
///
/// NOTE: this depends on nl-rdo-max's internal endpoints and HTML, so it can break when the bridge is
/// updated. Shared with `tests_integration::fake_digid::fake_digid_auth` — keep the two in sync.
///
/// [`DigidClient::authorization_request`]: crate::pid::digid::DigidClient::authorization_request
pub async fn drive_mock_digid_login(
    client: &reqwest::Client,
    mut authorize_url: Url,
    bsn: &str,
) -> Result<Url, MockDigidError> {
    // Preselect the mock IdP so nl-rdo-max skips its login-method landing page.
    authorize_url.query_pairs_mut().append_pair("login_hint", "digid_mock");

    // Fetch the mock SAML page and scrape the RelayState hidden field. (We deliberately skip
    // auto-submitting that form; nl-rdo-max is fine with hitting /acs directly.)
    let page = client
        .get(authorize_url.clone())
        .send()
        .await
        .map_err(MockDigidError::Http)?
        .text()
        .await
        .map_err(MockDigidError::Http)?;
    let relay_state = scrape_relay_state(&page)?;

    // In mock mode SAMLart is literally the BSN.
    let acs_url = mock_acs_url(&authorize_url, bsn, &relay_state);

    // With redirect following disabled on the client, the 302 Location is the issuer's
    // `/digid/callback` URL carrying the upstream code + state.
    let acs_response = client.get(acs_url).send().await.map_err(MockDigidError::Http)?;
    let location = acs_response
        .headers()
        .get(header::LOCATION)
        .ok_or(MockDigidError::MissingCallbackRedirect)?
        .to_str()
        .map_err(|_| MockDigidError::InvalidCallbackRedirect)?;

    location.parse().map_err(|_| MockDigidError::InvalidCallbackRedirect)
}

/// Scrape the `RelayState` value out of the hidden form field in nl-rdo-max's mock SAML page.
fn scrape_relay_state(page: &str) -> Result<String, MockDigidError> {
    let line = page
        .lines()
        .find(|line| line.contains("RelayState"))
        .ok_or(MockDigidError::RelayStateNotFound)?;
    let after = line.split_once("value=\"").ok_or(MockDigidError::RelayStateNotFound)?.1;
    let value = after.split_once('"').ok_or(MockDigidError::RelayStateNotFound)?.0;

    Ok(value.to_string())
}

/// Build the bridge's mock `/acs` URL for the given `/authorize` URL, BSN and scraped `RelayState`.
///
/// `/acs` is a sibling of the bridge's `/authorize` endpoint, so it is joined as a *relative* reference:
/// this replaces the last path segment (and drops the query) while preserving any base path the bridge
/// is mounted under.
fn mock_acs_url(authorize_url: &Url, bsn: &str, relay_state: &str) -> Url {
    let mut acs_url = authorize_url.join("acs").expect("\"acs\" is a valid relative path");
    acs_url
        .query_pairs_mut()
        .append_pair("SAMLart", bsn)
        .append_pair("RelayState", relay_state)
        .append_pair("mocking", "1");

    acs_url
}

/// Build the `Content-Security-Policy` served on the mock login page. Assets are same-origin
/// (`style-src`/`script-src 'self'`), so no inline styles/scripts are needed. `form-action` must
/// allow both the page's own origin (`'self'`, for the POST to `/select`) and the wallet's redirect
/// target: selecting a card 303-redirects toward it and browsers enforce `form-action` across
/// redirects.
pub fn build_mock_login_csp(wallet_redirect_uris: &VecNonEmpty<Url>) -> String {
    let form_action_sources = wallet_redirect_uris
        .iter()
        .map(|uri| match uri.scheme() {
            "http" | "https" => uri.origin().ascii_serialization(),
            scheme => format!("{scheme}:"),
        })
        .sorted()
        .dedup()
        .join(" ");

    format!(
        "default-src 'self'; style-src 'self'; script-src 'self'; img-src 'self' data:; font-src 'self' data:; \
         form-action 'self' {form_action_sources}; frame-ancestors 'none'; object-src 'none'; base-uri 'none';"
    )
}

/// Everything the mock login routes need: a redirect-disabled HTTP client trusting nl-rdo-max, the
/// configured test identities, and the `Content-Security-Policy` to serve on the page.
#[derive(Clone)]
pub struct MockLoginState {
    client: reqwest::Client,
    subjects: Arc<MockSubjects>,
    csp: &'static str,
}

impl MockLoginState {
    pub fn new(client: reqwest::Client, subjects: MockSubjects, csp: &'static str) -> Self {
        Self {
            client,
            subjects: Arc::new(subjects),
            csp,
        }
    }

    /// The mock login routes (page, card selection and static assets), with the CSP layered on.
    pub fn router(self) -> Router {
        let csp = self.csp;

        Router::new()
            .route(MOCK_LOGIN_PATH, get(mock_login_page))
            .route(MOCK_LOGIN_SELECT_PATH, post(mock_login_select))
            .route(MOCK_LOGIN_CSS_PATH, get(mock_login_css))
            .route(
                MOCK_LOGIN_JS_PATH,
                get(|| async { static_asset(JS_CONTENT_TYPE, MOCK_LOGIN_JS) }),
            )
            .route(
                MOCK_LOGIN_LOGO_PATH,
                get(|| async { static_asset(SVG_CONTENT_TYPE, MOCK_LOGIN_LOGO) }),
            )
            .route(
                LANGUAGE_SELECTOR_CSS_PATH,
                get(|| async { static_asset(CSS_CONTENT_TYPE, LANGUAGE_SELECTOR_CSS) }),
            )
            .route(
                LANGUAGE_SELECTOR_DOWN_PATH,
                get(|| async { static_asset(SVG_CONTENT_TYPE, LANGUAGE_SELECTOR_DOWN_SVG) }),
            )
            .route(
                LANGUAGE_SELECTOR_CHECKMARK_PATH,
                get(|| async { static_asset(SVG_CONTENT_TYPE, LANGUAGE_SELECTOR_CHECKMARK_SVG) }),
            )
            .route(
                LANGUAGE_JS_PATH,
                get(|| async { static_asset(JS_CONTENT_TYPE, LANGUAGE_JS) }),
            )
            .layer(middleware::from_fn(move |request, next| {
                set_content_security_policy(request, next, csp)
            }))
            .with_state(self)
    }
}

/// `GET /digid/mock-login/mock_login.css`: the page's stylesheet.
async fn mock_login_css(headers: HeaderMap) -> Response {
    serve_bundled_css(&headers, MOCK_LOGIN_CSS)
}

/// Serves a bundled static asset with the given content type.
fn static_asset(content_type: HeaderValue, body: &'static str) -> impl IntoResponse {
    ([(header::CONTENT_TYPE, content_type)], body)
}

struct Translations {
    logo_heading: &'static str,
    title: &'static str,
    paragraphs: &'static [&'static str],
    list_heading: &'static str,
    // Overlay shown (via JS) once a card has been submitted.
    signing_in: &'static str,
}

fn translations(language: Language) -> Translations {
    match language {
        Language::Nl => Translations {
            logo_heading: "Test-ID's",
            title: "Kies een test-ID",
            paragraphs: &[
                "Normaal gebruik je DigiD om te laten zien wie je bent. Daarna komen je eigen gegevens in NL Wallet.",
                "Je gebruikt nu een versie van NL Wallet om uit te proberen. Daarom slaan we DigiD over.",
                "Zo kun je NL Wallet uitproberen zonder je eigen gegevens te gebruiken.",
                "De test-ID die je kiest bestaat alleen uit testgegevens en is niet van een echt persoon.",
            ],
            list_heading: "Kies een test-ID om verder te gaan.",
            signing_in: "Bezig met inloggen…",
        },
        Language::En => Translations {
            logo_heading: "Test-IDs",
            title: "Choose a test ID",
            paragraphs: &[
                "Normally, you use DigiD to show who you are. Your own details are then added to NL Wallet.",
                "You are now using a version of NL Wallet that you can try out. That is why we skip DigiD.",
                "This lets you try NL Wallet without using your own details.",
                "The test ID you choose contains only test data and does not belong to a real person.",
            ],
            list_heading: "Choose a test ID to continue.",
            signing_in: "Signing in…",
        },
    }
}

/// One identity, rendered as a card (a form that POSTs its `bsn`).
struct Subject {
    name: String,
    bsn: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "mock_login.askama", escape = "html", ext = "html")]
struct MockLoginTemplate {
    /// Selected language, for the `<html lang>` attribute and the language selector.
    lang: Language,
    trans: Translations,
    /// The nl-rdo-max authorize URL, round-tripped as a hidden field so each card can POST it back and
    /// switching the language keeps it.
    authorize_url: String,
    subjects: Vec<Subject>,
    language_options: &'static [(Language, &'static str)],
}

/// Query parameters of the mock login page: the nl-rdo-max `/authorize` URL a selected card will
/// drive (set by the flow's `authorize` redirect). The language is resolved separately via the
/// [`Language`] extractor (`?lang=` or `Accept-Language`).
#[derive(Deserialize)]
struct PageQuery {
    authorize_url: String,
}

/// `GET /digid/mock-login`: render the grid of identity cards.
async fn mock_login_page(
    State(state): State<MockLoginState>,
    language: Language,
    Query(PageQuery { authorize_url }): Query<PageQuery>,
) -> MockLoginTemplate {
    let subjects = state
        .subjects
        .iter()
        .map(|(bsn, name)| Subject {
            name: name.clone(),
            bsn: bsn.clone(),
        })
        .collect();

    MockLoginTemplate {
        lang: language,
        trans: translations(language),
        authorize_url,
        subjects,
        language_options: LANGUAGE_OPTIONS,
    }
}

/// Form fields of a card selection.
#[derive(Deserialize)]
struct SelectForm {
    authorize_url: Url,
    bsn: String,
}

/// `POST /digid/mock-login/select`: "click the button" for one card. Drive nl-rdo-max's mock login,
/// then 303-redirect the user-agent to the resulting `/digid/callback` URL (which consumes the bridge
/// entry and redirects on to the wallet). The 303 turns the POST into a GET of the callback.
async fn mock_login_select(
    State(state): State<MockLoginState>,
    Form(SelectForm { authorize_url, bsn }): Form<SelectForm>,
) -> Result<Redirect, MockDigidError> {
    let callback_url = drive_mock_digid_login(&state.client, authorize_url, &bsn).await?;

    Ok(Redirect::to(callback_url.as_str()))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use axum::body::Body;
    use axum::http::Request;
    use axum::http::header::CACHE_CONTROL;
    use server_utils::server::add_cache_control_no_store_layer;
    use tower::ServiceExt;
    use url::Url;

    use super::MOCK_LOGIN_CSS_PATH;
    use super::MOCK_LOGIN_JS_PATH;
    use super::MOCK_LOGIN_LOGO_PATH;
    use super::MockLoginState;
    use super::MockSubjects;
    use super::mock_acs_url;

    #[tokio::test]
    async fn mock_login_assets_send_no_store_cache_control() {
        let router = add_cache_control_no_store_layer(
            MockLoginState::new(reqwest::Client::new(), MockSubjects::new(), "default-src 'self'").router(),
        );

        for path in [MOCK_LOGIN_CSS_PATH, MOCK_LOGIN_JS_PATH, MOCK_LOGIN_LOGO_PATH] {
            let response = router
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();

            assert_eq!(response.headers().get(CACHE_CONTROL).unwrap(), "no-store");
        }
    }

    #[test]
    fn mock_acs_url_preserves_bridge_base_path() {
        // nl-rdo-max at the origin root (local devenv): `/acs` sits directly under the origin.
        let root = Url::parse("https://localhost:8006/authorize?login_hint=digid_mock").unwrap();
        assert_eq!(mock_acs_url(&root, "999991772", "relay").path(), "/acs");

        // nl-rdo-max behind a base path prefix (as in CI): the prefix must be preserved, otherwise
        // `/acs` 404s and the bridge returns no callback redirect.
        let prefixed = Url::parse("https://example.com/digid-connector/authorize?login_hint=digid_mock").unwrap();
        let acs = mock_acs_url(&prefixed, "999991772", "relay");
        assert_eq!(acs.path(), "/digid-connector/acs");

        // The BSN (as SAMLart), RelayState and mocking flag are available as query parameters.
        let query: HashMap<_, _> = acs.query_pairs().into_owned().collect();
        assert_eq!(query.get("SAMLart").map(String::as_str), Some("999991772"));
        assert_eq!(query.get("RelayState").map(String::as_str), Some("relay"));
        assert_eq!(query.get("mocking").map(String::as_str), Some("1"));
    }
}
