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
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use url::Url;
use utils::vec_at_least::VecNonEmpty;
use web_utils::css::serve_bundled_css;
use web_utils::headers::set_content_security_policy;
use web_utils::language::Language;

/// Path (relative to the issuer's public URL) of the mock login page.
pub const MOCK_LOGIN_PATH: &str = "/digid/mock-login";

/// Path of the per-card selection endpoint that drives the bridge and hands off to `/digid/callback`.
const MOCK_LOGIN_SELECT_PATH: &str = "/digid/mock-login/select";

/// Paths of the page's (same-origin) stylesheet and script, so a strict CSP can use `'self'` rather
/// than allowing inline styles/scripts.
const MOCK_LOGIN_CSS_PATH: &str = "/digid/mock-login/mock_login.css";
const MOCK_LOGIN_JS_PATH: &str = "/digid/mock-login/mock_login.js";

const MOCK_LOGIN_CSS: &str = include_str!("../../static/mock_login.css");
const MOCK_LOGIN_JS: &str = include_str!("../../static/mock_login.js");

/// A selectable mock identity, rendered as a card on the mock login page.
///
/// `bsn` must resolve in the BRP proxy's dataset, since the real `/digid/callback` performs a BRP
/// lookup for it; `name` is a display label only.
#[derive(Clone, Serialize, Deserialize)]
pub struct MockSubject {
    pub bsn: String,
    pub name: String,
}

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

    // Mock ACS: in mock mode SAMLart is literally the BSN. `/acs` lives at the authorize origin;
    // joining an absolute path drops the `/authorize` path and its query for us.
    let mut acs_url = authorize_url.join("/acs").expect("\"/acs\" is a valid path");
    acs_url
        .query_pairs_mut()
        .append_pair("SAMLart", bsn)
        .append_pair("RelayState", &relay_state)
        .append_pair("mocking", "1");

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
    subjects: Arc<Vec<MockSubject>>,
    csp: &'static str,
}

impl MockLoginState {
    pub fn new(client: reqwest::Client, subjects: Vec<MockSubject>, csp: &'static str) -> Self {
        Self {
            client,
            subjects: Arc::new(subjects),
            csp,
        }
    }

    /// The non-cacheable dynamic mock login routes (the page and per-card selection), with the CSP layered on.
    pub fn page_router(self) -> Router {
        let csp = self.csp;

        Router::new()
            .route(MOCK_LOGIN_PATH, get(mock_login_page))
            .route(MOCK_LOGIN_SELECT_PATH, post(mock_login_select))
            .layer(middleware::from_fn(move |request, next| {
                set_content_security_policy(request, next, csp)
            }))
            .with_state(self)
    }

    /// The cacheable static asset routes (CSS + JS), with the CSP layered on.
    pub fn assets_router(&self) -> Router {
        let csp = self.csp;

        Router::new()
            .route(MOCK_LOGIN_CSS_PATH, get(mock_login_css))
            .route(MOCK_LOGIN_JS_PATH, get(mock_login_js))
            .layer(middleware::from_fn(move |request, next| {
                set_content_security_policy(request, next, csp)
            }))
    }
}

/// `GET /digid/mock-login/mock_login.css`: the page's stylesheet.
async fn mock_login_css(headers: HeaderMap) -> Response {
    serve_bundled_css(&headers, MOCK_LOGIN_CSS)
}

/// `GET /digid/mock-login/mock_login.js`: the page's script.
async fn mock_login_js() -> impl IntoResponse {
    (
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/javascript; charset=utf-8"),
            ),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=604800"),
            ),
        ],
        MOCK_LOGIN_JS,
    )
}

struct Translations {
    title: &'static str,
    intro: &'static str,
    // Overlay shown (via JS) once a card has been submitted.
    signing_in: &'static str,
    // The "enter your own BSN" card, for BSNs not in the configured list.
    custom_heading: &'static str,
    custom_placeholder: &'static str,
    custom_submit: &'static str,
}

fn translations(language: Language) -> Translations {
    match language {
        Language::Nl => Translations {
            title: "Kies een test-identiteit",
            intro: "Selecteer een test-identiteit of voer zelf een BSN in.",
            signing_in: "Bezig met inloggen…",
            custom_heading: "Eigen BSN",
            custom_placeholder: "Voer een BSN in",
            custom_submit: "Inloggen",
        },
        Language::En => Translations {
            title: "Choose a test identity",
            intro: "Select a test identity or enter a BSN yourself.",
            signing_in: "Signing in…",
            custom_heading: "Custom BSN",
            custom_placeholder: "Enter a BSN",
            custom_submit: "Sign in",
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
    /// Selected language code (`"nl"`/`"en"`), for the `<html lang>` attribute and the language bar.
    lang: String,
    trans: Translations,
    /// The nl-rdo-max authorize URL, round-tripped as a hidden field so each card can POST it back.
    authorize_url: String,
    subjects: Vec<Subject>,
    /// Links that switch the page language while preserving `authorize_url`.
    nl_href: String,
    en_href: String,
    select_path: &'static str,
    css_path: &'static str,
    js_path: &'static str,
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
    // A language-switch link for the given code, preserving the authorize URL across the switch.
    let lang_href = |code: &str| {
        let query = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("lang", code)
            .append_pair("authorize_url", &authorize_url)
            .finish();
        format!("{MOCK_LOGIN_PATH}?{query}")
    };

    let subjects = state
        .subjects
        .iter()
        .map(|subject| Subject {
            name: subject.name.clone(),
            bsn: subject.bsn.clone(),
        })
        .collect();

    MockLoginTemplate {
        lang: language.to_string(),
        trans: translations(language),
        nl_href: lang_href("nl"),
        en_href: lang_href("en"),
        authorize_url,
        subjects,
        select_path: MOCK_LOGIN_SELECT_PATH,
        css_path: MOCK_LOGIN_CSS_PATH,
        js_path: MOCK_LOGIN_JS_PATH,
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
