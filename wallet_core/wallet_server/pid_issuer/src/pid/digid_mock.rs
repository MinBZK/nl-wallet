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
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum::response::Response;
use axum::routing::get;
use reqwest::header;
use serde::Deserialize;
use serde::Serialize;
use url::Url;

/// Path (relative to the issuer's public URL) of the mock login page.
pub const MOCK_LOGIN_PATH: &str = "/digid/mock-login";

/// Path of the per-card selection endpoint that drives the bridge and hands off to `/digid/callback`.
const MOCK_LOGIN_SELECT_PATH: &str = "/digid/mock-login/select";

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
    let after = line
        .split("value=\"")
        .nth(1)
        .ok_or(MockDigidError::RelayStateNotFound)?;
    let value = after.split('"').next().ok_or(MockDigidError::RelayStateNotFound)?;

    Ok(value.to_string())
}

/// Everything the mock login routes need: a redirect-disabled HTTP client trusting nl-rdo-max, and
/// the configured test identities.
#[derive(Clone)]
pub struct MockLoginState {
    client: reqwest::Client,
    subjects: Arc<Vec<MockSubject>>,
}

impl MockLoginState {
    pub fn new(client: reqwest::Client, subjects: Vec<MockSubject>) -> Self {
        Self {
            client,
            subjects: Arc::new(subjects),
        }
    }

    /// Mount the mock login page and its per-card selection endpoint on a fresh [`Router`]. The
    /// pid_issuer's `server` module merges this with the issuance/authorization/callback routers.
    pub fn router(self) -> Router {
        Router::new()
            .route(MOCK_LOGIN_PATH, get(mock_login_page))
            .route(MOCK_LOGIN_SELECT_PATH, get(mock_login_select))
            .with_state(self)
    }
}

/// One rendered identity card: its display fields plus the pre-built, percent-encoded selection link.
struct Card {
    name: String,
    bsn: String,
    href: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "mock_login.askama", escape = "html", ext = "html")]
struct MockLoginTemplate {
    cards: Vec<Card>,
}

/// Query parameters of the mock login page, set by the flow's `authorize` redirect: the nl-rdo-max
/// `/authorize` URL that a selected card will drive.
#[derive(Deserialize)]
struct PageQuery {
    authorize_url: String,
}

/// `GET /digid/mock-login`: render the grid of identity cards.
async fn mock_login_page(
    State(state): State<MockLoginState>,
    Query(PageQuery { authorize_url }): Query<PageQuery>,
) -> MockLoginTemplate {
    let cards = state
        .subjects
        .iter()
        .map(|subject| {
            // Absolute link so relative resolution from `/digid/mock-login` is unambiguous.
            let query = url::form_urlencoded::Serializer::new(String::new())
                .append_pair("authorize_url", &authorize_url)
                .append_pair("bsn", &subject.bsn)
                .finish();

            Card {
                name: subject.name.clone(),
                bsn: subject.bsn.clone(),
                href: format!("{MOCK_LOGIN_SELECT_PATH}?{query}"),
            }
        })
        .collect();

    MockLoginTemplate { cards }
}

/// Query parameters of a card selection.
#[derive(Deserialize)]
struct SelectQuery {
    authorize_url: Url,
    bsn: String,
}

/// `GET /digid/mock-login/select`: "click the button" for one card. Drive nl-rdo-max's mock login,
/// then 302 the user-agent to the resulting `/digid/callback` URL (which consumes the bridge entry
/// and redirects on to the wallet).
async fn mock_login_select(
    State(state): State<MockLoginState>,
    Query(SelectQuery { authorize_url, bsn }): Query<SelectQuery>,
) -> Result<Redirect, MockDigidError> {
    let callback_url = drive_mock_digid_login(&state.client, authorize_url, &bsn).await?;

    Ok(Redirect::to(callback_url.as_str()))
}
