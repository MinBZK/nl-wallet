//! A pid_issuer-hosted login page.
//!
//! This module lets the pid_issuer serve a grid of selectable test identities (BSN + name).
//! Selecting a card "clicks the button" and completes the login flow.

use std::sync::Arc;

use askama::Template;
use askama_web::WebTemplate;
use axum::Router;
use axum::extract::Query;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::HeaderValue;
use axum::http::header;
use axum::middleware;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
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

use crate::pid::attributes::Person;

/// Path (relative to the issuer's public URL) of the login page.
const LOGIN_PATH: &str = "/login";

/// Paths of the page's (same-origin) stylesheet and script, so a strict CSP can use `'self'` rather
/// than allowing inline styles/scripts.
const LOGIN_CSS_PATH: &str = "/login/login.css";
const LOGIN_JS_PATH: &str = "/login/login.js";
const LOGIN_LOGO_PATH: &str = "/login/digid.svg";

/// Paths of the shared language selector's assets. The stylesheet refers to its icons as `../images/*.svg`,
/// so it is served from a `css/` sibling of `images/`.
const LANGUAGE_SELECTOR_CSS_PATH: &str = "/login/css/language_selector.css";
const LANGUAGE_SELECTOR_DOWN_PATH: &str = "/login/images/down.svg";
const LANGUAGE_SELECTOR_CHECKMARK_PATH: &str = "/login/images/checkmark.svg";
const LANGUAGE_JS_PATH: &str = "/login/language.js";

const LOGIN_CSS: &str = include_str!("../../static/login.css");
const LOGIN_JS: &str = include_str!("../../static/login.js");
const LOGIN_LOGO: &str = include_str!("../../static/non-free/digid.svg");

const CSS_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("text/css");
const JS_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("text/javascript");
const SVG_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("image/svg+xml");

/// The languages offered by the language selector, each labelled in its own language.
const LANGUAGE_OPTIONS: &[(Language, &str)] = &[(Language::Nl, "Nederlands"), (Language::En, "English")];

/// Build the `Content-Security-Policy` served on the login page. Assets are same-origin
/// (`style-src`/`script-src 'self'`), so no inline styles/scripts are needed. `form-action` must
/// allow both the page's own origin (`'self'`, for the POST to `/login`) and the wallet's redirect
/// target: selecting a card 303-redirects toward it and browsers enforce `form-action` across
/// redirects.
pub fn build_login_csp(wallet_redirect_uris: &VecNonEmpty<Url>) -> String {
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

/// Everything the login page needs: the state needed to serve the login page, and the
/// `Content-Security-Policy` to serve on the page.
#[derive(Clone)]
pub struct LoginPage {
    state: LoginPageState,
    csp: &'static str,
}

#[derive(Clone)]
struct LoginPageState {
    subjects: Arc<Vec<Subject>>,
    custom_bsn: bool,
}

impl LoginPage {
    pub fn new<'a>(persons: impl IntoIterator<Item = &'a Person>, custom_bsn: bool, csp: &'static str) -> Self {
        let subjects = Arc::new(
            persons
                .into_iter()
                .map(|p| Subject {
                    bsn: p.bsn.clone(),
                    name: format!("{} {}", p.given_name, p.family_name),
                })
                .collect(),
        );
        Self {
            state: LoginPageState { subjects, custom_bsn },
            csp,
        }
    }

    pub fn path(&self) -> &str {
        LOGIN_PATH
    }

    /// The login routes (page, card selection and static assets), with the CSP layered on.
    pub fn router(self) -> Router {
        let csp = self.csp;

        Router::new()
            .route(LOGIN_PATH, get(login_page))
            .route(LOGIN_CSS_PATH, get(login_css))
            .route(LOGIN_JS_PATH, get(|| async { static_asset(JS_CONTENT_TYPE, LOGIN_JS) }))
            .route(
                LOGIN_LOGO_PATH,
                get(|| async { static_asset(SVG_CONTENT_TYPE, LOGIN_LOGO) }),
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
            .with_state(self.state)
    }
}

/// `GET /login/login.css`: the page's stylesheet.
async fn login_css(headers: HeaderMap) -> Response {
    serve_bundled_css(&headers, LOGIN_CSS)
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
    // The "enter custom BSN" card, for BSNs not in the configured list.
    custom_heading: &'static str,
    custom_placeholder: &'static str,
    custom_submit: &'static str,
    custom_invalid_bsn: &'static str,
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
            custom_heading: "Ander test BSN",
            custom_placeholder: "Voer een BSN in",
            custom_submit: "Inloggen",
            custom_invalid_bsn: "Geen geldig BSN",
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
            custom_heading: "Custom test BSN",
            custom_placeholder: "Enter a BSN",
            custom_submit: "Sign in",
            custom_invalid_bsn: "Invalid BSN",
        },
    }
}

/// One identity, rendered as a card (a form that POSTs its `bsn`).
#[derive(Debug, Clone)]
struct Subject {
    name: String,
    bsn: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "login.askama", escape = "html", ext = "html")]
struct LoginTemplate {
    /// Selected language, for the `<html lang>` attribute and the language selector.
    lang: Language,
    trans: Translations,
    state: String,
    subjects: Vec<Subject>,
    custom_bsn: bool,
    language_options: &'static [(Language, &'static str)],
}

/// Query parameters of the login page, only state is required. The language is resolve
/// separately via the [`Language`] extractor (`?lang=` or `Accept-Language`).
#[derive(Deserialize)]
struct PageQuery {
    state: String,
}

/// `GET /login`: render the grid of identity cards.
async fn login_page(
    State(LoginPageState { subjects, custom_bsn }): State<LoginPageState>,
    language: Language,
    Query(PageQuery { state }): Query<PageQuery>,
) -> LoginTemplate {
    LoginTemplate {
        lang: language,
        trans: translations(language),
        custom_bsn,
        state,
        subjects: subjects.iter().cloned().collect(),
        language_options: LANGUAGE_OPTIONS,
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use axum::http::header::CACHE_CONTROL;
    use server_utils::server::add_cache_control_no_store_layer;
    use tower::ServiceExt;

    use super::LOGIN_CSS_PATH;
    use super::LOGIN_JS_PATH;
    use super::LOGIN_LOGO_PATH;
    use super::LoginPage;
    use crate::pid::mock::MOCK_PERSON;

    #[tokio::test]
    async fn login_assets_send_no_store_cache_control() {
        let router =
            add_cache_control_no_store_layer(LoginPage::new([&*MOCK_PERSON], false, "default-src 'self'").router());

        for path in [LOGIN_CSS_PATH, LOGIN_JS_PATH, LOGIN_LOGO_PATH] {
            let response = router
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();

            assert_eq!(response.headers().get(CACHE_CONTROL).unwrap(), "no-store");
        }
    }
}
