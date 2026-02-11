use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::OnceLock;

use askama::Template;
use askama_web::WebTemplate;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::handler::HandlerWithoutStateExt;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use strum::IntoEnumIterator;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::warn;
use url::Url;

use attestation_data::attributes::AttributeValue;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use demo_utils::WALLET_WEB_CSS_SHA256;
use demo_utils::WALLET_WEB_JS_SHA256;
use demo_utils::disclosure::DemoDisclosedAttestation;
use http_utils::health::create_health_router;
use http_utils::urls::BaseUrl;
use http_utils::urls::ConnectSource;
use http_utils::urls::SourceExpression;
use openid4vc::server_state::SessionToken;
use server_utils::log_requests::log_request_response;
use utils::path::prefix_local_path;
use web_utils::error::Result;
use web_utils::headers::cors_layer;
use web_utils::headers::set_content_security_policy;
use web_utils::headers::set_static_cache_control;
use web_utils::language::LANGUAGE_JS_SHA256;
use web_utils::language::Language;
use web_utils::language::LanguageParam;

use crate::client::WalletServerClient;
use crate::settings::ReturnUrlMode;
use crate::settings::Settings;
use crate::settings::Usecase;
use crate::translations::TRANSLATIONS;
use crate::translations::Words;

const RETURN_URL_SEGMENT: &str = "return";

struct ApplicationState {
    client: WalletServerClient,
    public_wallet_server_url: BaseUrl,
    public_url: BaseUrl,
    help_base_url: BaseUrl,
    demo_index_url: BaseUrl,
    usecases: HashMap<String, Usecase>,
}

// Bundled CSS constants - placeholders in dev mode, full bundles in release mode.
// In dev mode, CSS is served from the filesystem via ServeDir.
pub const AMSTERDAM_INDEX_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/mijn_amsterdam-index.css"));
pub const AMSTERDAM_RETURN_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/mijn_amsterdam-return.css"));
pub const MONKEY_BIKE_INDEX_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/monkey_bike-index.css"));
pub const MONKEY_BIKE_RETURN_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/monkey_bike-return.css"));
pub const ONLINE_MARKETPLACE_INDEX_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/online_marketplace-index.css"));
pub const ONLINE_MARKETPLACE_RETURN_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/online_marketplace-return.css"));
pub const XYZ_BANK_INDEX_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/xyz_bank-index.css"));
pub const XYZ_BANK_RETURN_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/xyz_bank-return.css"));
pub const JOB_FINDER_INDEX_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/job_finder-index.css"));
pub const JOB_FINDER_RETURN_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/job_finder-return.css"));

static CSP_HEADER: OnceLock<String> = OnceLock::new();

/// Serves a bundled CSS file with caching headers. Only used in release mode.
#[cfg(not(debug_assertions))]
fn serve_bundled_css(css: &'static str) -> Response {
    use axum::http::header;
    (
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8".to_string()),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable".to_string()),
        ],
        css,
    )
        .into_response()
}

pub fn create_router(settings: Settings) -> Router {
    let application_state = Arc::new(ApplicationState {
        client: WalletServerClient::new(settings.internal_wallet_server_url.clone()),
        public_wallet_server_url: settings.public_wallet_server_url,
        public_url: settings.public_url,
        help_base_url: settings.help_base_url,
        demo_index_url: settings.demo_index_url,
        usecases: settings.usecases,
    });

    let connect_src = settings
        .connect_src
        .unwrap_or(ConnectSource::List(vec![SourceExpression::SelfSource]))
        .to_string();
    let app = Router::new()
        .route("/sessions", post(create_session))
        .route("/{usecase}/", get(usecase))
        .route(&format!("/{{usecase}}/{RETURN_URL_SEGMENT}"), get(disclosed_attributes));

    // In release mode, serve bundled CSS from route handlers.
    // In debug mode, CSS is served from the filesystem via the ServeDir fallback.
    #[cfg(not(debug_assertions))]
    let app = app
        .route("/static/css/mijn_amsterdam-index.css", get(|| async { serve_bundled_css(AMSTERDAM_INDEX_CSS) }))
        .route("/static/css/mijn_amsterdam-return.css", get(|| async { serve_bundled_css(AMSTERDAM_RETURN_CSS) }))
        .route("/static/css/monkey_bike-index.css", get(|| async { serve_bundled_css(MONKEY_BIKE_INDEX_CSS) }))
        .route("/static/css/monkey_bike-return.css", get(|| async { serve_bundled_css(MONKEY_BIKE_RETURN_CSS) }))
        .route("/static/css/online_marketplace-index.css", get(|| async { serve_bundled_css(ONLINE_MARKETPLACE_INDEX_CSS) }))
        .route("/static/css/online_marketplace-return.css", get(|| async { serve_bundled_css(ONLINE_MARKETPLACE_RETURN_CSS) }))
        .route("/static/css/xyz_bank-index.css", get(|| async { serve_bundled_css(XYZ_BANK_INDEX_CSS) }))
        .route("/static/css/xyz_bank-return.css", get(|| async { serve_bundled_css(XYZ_BANK_RETURN_CSS) }))
        .route("/static/css/job_finder-index.css", get(|| async { serve_bundled_css(JOB_FINDER_INDEX_CSS) }))
        .route("/static/css/job_finder-return.css", get(|| async { serve_bundled_css(JOB_FINDER_RETURN_CSS) }));

    let mut app = app
        .fallback_service(
            ServiceBuilder::new()
                .layer(middleware::from_fn(set_static_cache_control))
                .service(
                    ServeDir::new(prefix_local_path(std::path::Path::new("assets")))
                        .not_found_service({ StatusCode::NOT_FOUND }.into_service()),
                ),
        )
        .with_state(application_state)
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(move |req, next| {
            let csp_header = CSP_HEADER.get_or_init(|| {
                let script_src = format!(
                    "'sha256-{}' 'sha256-{}' 'sha256-{}'",
                    *LANGUAGE_JS_SHA256, *USECASE_JS_SHA256, *WALLET_WEB_JS_SHA256
                );
                let style_src = format!("'self' 'sha256-{}'", *WALLET_WEB_CSS_SHA256);

                format!(
                    "default-src 'self'; script-src {script_src}; style-src {style_src}; img-src 'self' data:; \
                     font-src 'self' data:; form-action 'self'; frame-ancestors 'none'; object-src 'none'; base-uri \
                     'none'; connect-src {connect_src};"
                )
            });

            set_content_security_policy(req, next, csp_header)
        }));

    if let Some(cors_origin) = settings.allow_origins {
        app = app.layer(cors_layer(cors_origin));
    }

    if settings.log_requests {
        app = app.layer(axum::middleware::from_fn(log_request_response));
    }

    app.merge(create_health_router([]))
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    all(test, feature = "ts_rs"),
    derive(ts_rs::TS),
    ts(export, export_to = "relying_party.ts")
)]
struct SessionOptions {
    usecase: String,
}

#[derive(Serialize)]
#[cfg_attr(
    all(test, feature = "ts_rs"),
    derive(ts_rs::TS),
    ts(export, export_to = "relying_party.ts")
)]
struct SessionResponse {
    #[cfg_attr(all(test, feature = "ts_rs"), ts(type = "URL"))]
    status_url: Url,
    #[cfg_attr(all(test, feature = "ts_rs"), ts(type = "string"))]
    session_token: SessionToken,
}

async fn create_session(
    State(state): State<Arc<ApplicationState>>,
    language: Language,
    Json(options): Json<SessionOptions>,
) -> Result<Json<SessionResponse>> {
    let usecase = state.usecases.get(&options.usecase).cloned().unwrap_or_default();

    let return_url_template = match usecase.return_url {
        ReturnUrlMode::None => None,
        _ => Some(
            format!(
                "{}/{}?session_token={{session_token}}&lang={}",
                state.public_url.join(&options.usecase),
                RETURN_URL_SEGMENT,
                language,
            )
            .parse()
            .expect("should always be a valid ReturnUrlTemplate"),
        ),
    };

    let session_token = state
        .client
        .start(options.usecase, usecase.dcql_query, return_url_template)
        .await?;

    let result = SessionResponse {
        status_url: state
            .public_wallet_server_url
            .join(&format!("disclosure/sessions/{session_token}")),
        session_token,
    };
    Ok(result.into())
}

struct BaseTemplate<'a> {
    session_token: Option<SessionToken>,
    nonce: Option<&'a str>,
    selected_lang: Language,
    trans: &'a Words<'a>,
    available_languages: &'a [Language],
    language_js_sha256: &'a str,
}

#[derive(Template, WebTemplate)]
#[template(path = "usecase/usecase.askama", escape = "html", ext = "html")]
struct UsecaseTemplate<'a> {
    usecase: &'a str,
    start_url: Url,
    help_base_url: Url,
    usecase_js_sha256: &'a str,
    wallet_web_sha256: &'a str,
    base: BaseTemplate<'a>,
}

static USECASE_JS_SHA256: LazyLock<String> =
    LazyLock::new(|| BASE64_STANDARD.encode(crypto::utils::sha256(include_bytes!("../static/usecase.js"))));

fn format_start_url(public_url: &BaseUrl, lang: Language) -> Url {
    let mut start_url = public_url.join("/sessions");
    start_url.set_query(Some(
        serde_urlencoded::to_string(LanguageParam { lang }).unwrap().as_str(),
    ));
    start_url
}

async fn usecase(
    State(state): State<Arc<ApplicationState>>,
    Path(usecase): Path<String>,
    language: Language,
) -> Response {
    let start_url = format_start_url(&state.public_url, language);
    let base = BaseTemplate {
        session_token: None,
        nonce: None,
        selected_lang: language,
        trans: &TRANSLATIONS[language],
        available_languages: &Language::iter().collect_vec(),
        language_js_sha256: &LANGUAGE_JS_SHA256,
    };
    UsecaseTemplate {
        usecase: &usecase,
        start_url,
        help_base_url: state.help_base_url.clone().into_inner(),
        usecase_js_sha256: &USECASE_JS_SHA256,
        wallet_web_sha256: &WALLET_WEB_JS_SHA256,
        base,
    }
    .into_response()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisclosedAttributesParams {
    pub nonce: Option<String>,
    pub session_token: SessionToken,
}

#[derive(Template, WebTemplate)]
#[template(path = "disclosed/attributes.askama", escape = "html", ext = "html")]
struct DisclosedAttributesTemplate<'a> {
    usecase: &'a str,
    attributes: Vec<DemoDisclosedAttestation>,
    demo_index_url: Url,
    base: BaseTemplate<'a>,
}

async fn disclosed_attributes(
    State(state): State<Arc<ApplicationState>>,
    Path(usecase): Path<String>,
    Query(params): Query<DisclosedAttributesParams>,
    language: Language,
) -> Response {
    let attributes = state
        .client
        .disclosed_attributes(params.session_token.clone(), params.nonce.clone())
        .await;

    let start_url = format_start_url(&state.public_url, language);
    let base = BaseTemplate {
        session_token: Some(params.session_token),
        nonce: params.nonce.as_deref(),
        selected_lang: language,
        trans: &TRANSLATIONS[language],
        available_languages: &Language::iter().collect_vec(),
        language_js_sha256: &LANGUAGE_JS_SHA256,
    };

    match attributes {
        Ok(attributes) => DisclosedAttributesTemplate {
            usecase: &usecase,
            demo_index_url: state.demo_index_url.clone().into_inner(),
            attributes: attributes
                .into_inner()
                .into_iter()
                .flat_map(|disclosed_attestations| disclosed_attestations.attestations.into_inner())
                .collect(),
            base,
        }
        .into_response(),
        Err(err) => {
            warn!("Error getting disclosed attributes: {err}");
            UsecaseTemplate {
                usecase: &usecase,
                start_url,
                help_base_url: state.help_base_url.clone().into_inner(),
                usecase_js_sha256: &USECASE_JS_SHA256,
                wallet_web_sha256: &WALLET_WEB_JS_SHA256,
                base,
            }
            .into_response()
        }
    }
}

mod filters {
    use demo_utils::disclosure::DemoDisclosedAttestation;

    // searches for an attribute with a specific key, the key is a dot-separated string
    pub fn attribute(
        attestations: &[DemoDisclosedAttestation],
        _: &dyn askama::Values,
        name: &str,
    ) -> askama::Result<String> {
        for attestation in attestations {
            if let Some(attribute_value) = attestation
                .attributes
                .flattened()
                .iter()
                .find_map(|(path, attribute_value)| (path.as_ref().join(".") == name).then_some(attribute_value))
            {
                return Ok(attribute_value.to_string());
            }
        }

        Err(askama::Error::custom("attribute '{name}' cannot be found"))
    }
}
