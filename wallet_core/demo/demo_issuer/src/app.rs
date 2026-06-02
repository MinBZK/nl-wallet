use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;

use askama::Template;
use askama_web::WebTemplate;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::State;
use axum::handler::HandlerWithoutStateExt;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use demo_utils::WALLET_WEB_CSS_SHA256;
use demo_utils::WALLET_WEB_JS_SHA256;
use demo_utils::disclosure::DemoDisclosedAttestations;
use http_utils::health::create_health_router;
use http_utils::urls::BaseUrl;
use http_utils::urls::disclosure_based_issuance_base_uri;
use itertools::Itertools;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::openid4vp::ClientId;
use openid4vc::openid4vp::VpRequestUri;
use openid4vc::openid4vp::VpRequestUriMethod;
use openid4vc::openid4vp::VpRequestUriObject;
use openid4vc::verifier::SessionType;
use openid4vc::verifier::VerifierUrlParameters;
use pacf_issuance_server::offer::OfferRequest;
use pacf_issuance_server::offer::OfferResponse;
use server_utils::log_requests::log_request_response;
use strum::IntoEnumIterator;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use url::Url;
use utils::path::prefix_local_path;
use utils::vec_at_least::VecNonEmpty;
use web_utils::error::Result;
use web_utils::headers::set_content_security_policy;
use web_utils::headers::set_static_cache_control;
use web_utils::language::LANGUAGE_JS_SHA256;
use web_utils::language::Language;

use crate::settings::IssuableDocumentTemplates;
use crate::settings::Settings;
use crate::settings::Usecase;
use crate::translations::TRANSLATIONS;
use crate::translations::Words;

struct ApplicationState {
    usecases: HashMap<String, Usecase>,
    issuance_server_url: BaseUrl,
    universal_link_base_url: BaseUrl,
    help_base_url: BaseUrl,
}

// Bundled CSS constants - placeholders in dev mode, full bundles in release mode.
// In dev mode, CSS is served from the filesystem via ServeDir.
pub const HOUSING_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/housing.css"));
pub const INSURANCE_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/insurance.css"));
pub const UNIVERSITY_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/university.css"));

static CSP_HEADER: LazyLock<String> = LazyLock::new(|| {
    let script_src = format!("'sha256-{}' 'sha256-{}'", *LANGUAGE_JS_SHA256, *WALLET_WEB_JS_SHA256);
    let style_src = format!("'self' 'sha256-{}'", *WALLET_WEB_CSS_SHA256);

    format!(
        "default-src 'self'; script-src {script_src}; style-src {style_src}; img-src 'self' data:; font-src 'self' \
         data:; form-action 'self'; frame-ancestors 'none'; object-src 'none'; base-uri 'none';"
    )
});

pub fn create_routers(settings: Settings) -> (Router, Router) {
    let application_state = Arc::new(ApplicationState {
        usecases: settings.usecases,
        issuance_server_url: settings.issuance_server_url,
        universal_link_base_url: settings.universal_link_base_url,
        help_base_url: settings.help_base_url,
    });

    let app = Router::new().route("/{usecase}/", get(usecase));

    // In release mode, serve bundled CSS from route handlers.
    // In debug mode, CSS is served from the filesystem via the ServeDir fallback.
    #[cfg(not(debug_assertions))]
    let app = app
        .route(
            "/static/css/housing.css",
            get(|h: axum::http::HeaderMap| async move { web_utils::css::serve_bundled_css(&h, HOUSING_CSS) }),
        )
        .route(
            "/static/css/insurance.css",
            get(|h: axum::http::HeaderMap| async move { web_utils::css::serve_bundled_css(&h, INSURANCE_CSS) }),
        )
        .route(
            "/static/css/university.css",
            get(|h: axum::http::HeaderMap| async move { web_utils::css::serve_bundled_css(&h, UNIVERSITY_CSS) }),
        );

    let mut app = app
        .fallback_service(
            ServiceBuilder::new()
                .layer(middleware::from_fn(set_static_cache_control))
                .service(
                    ServeDir::new(prefix_local_path(std::path::Path::new("assets")))
                        .not_found_service({ StatusCode::NOT_FOUND }.into_service()),
                ),
        )
        .with_state(Arc::clone(&application_state))
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(|req, next| {
            set_content_security_policy(req, next, &CSP_HEADER)
        }));

    let attestation_router = Router::new()
        .route("/{usecase}/", post(attestation))
        .with_state(application_state)
        .layer(TraceLayer::new_for_http());

    if settings.log_requests {
        app = app.layer(axum::middleware::from_fn(log_request_response));
    }

    (app.merge(create_health_router([])), attestation_router)
}

struct BaseTemplate<'a> {
    selected_lang: Language,
    trans: &'a Words<'a>,
    available_languages: &'a [Language],
    language_js_sha256: &'a str,
}

#[derive(Template, WebTemplate)]
#[template(path = "usecase/usecase.askama", escape = "html", ext = "html")]
struct UsecaseTemplate<'a> {
    usecase: &'a str,
    same_device_ul: Url,
    cross_device_ul: Url,
    help_base_url: Url,
    wallet_web_sha256: &'a str,
    base: BaseTemplate<'a>,
}

fn disclosure_based_issuance_universal_links(
    issuance_server_url: &BaseUrl,
    usecase: &str,
    universal_link_base: &BaseUrl,
    client_id: &str,
) -> HashMap<SessionType, Url> {
    let client_id: ClientId = client_id
        .parse()
        .expect("configured demo issuer client_id must be a valid OpenID4VP client_id");

    SessionType::iter()
        .map(|session_type| {
            let params = serde_urlencoded::to_string(VerifierUrlParameters {
                session_type,
                ephemeral_id_params: None,
            })
            .unwrap();

            let mut issuance_server_url = issuance_server_url.join(&format!("/disclosure/{usecase}/request_uri"));
            issuance_server_url.set_query(Some(&params));

            let query = serde_urlencoded::to_string(VpRequestUri {
                client_id: client_id.clone(),
                object: VpRequestUriObject::AsReference {
                    request_uri: issuance_server_url.try_into().unwrap(),
                    request_uri_method: Some(VpRequestUriMethod::POST),
                },
            })
            .unwrap();

            let mut uri = disclosure_based_issuance_base_uri(universal_link_base).into_inner();
            uri.set_query(Some(&query));
            (session_type, uri)
        })
        .collect::<HashMap<_, _>>()
}

async fn usecase(
    State(state): State<Arc<ApplicationState>>,
    Path(usecase_id): Path<String>,
    language: Language,
) -> Result<Response> {
    let Some(usecase) = state.usecases.get(&usecase_id) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(match usecase {
        Usecase::PreAuthorized { data, offer_url } => {
            let data = data.clone();
            let offer_url = offer_url.clone();
            pre_authorized_usecase(
                &usecase_id,
                language,
                &data,
                offer_url,
                state.help_base_url.clone().into_inner(),
            )
            .await?
        }
        Usecase::DisclosureBased { client_id, .. } => {
            let client_id = client_id.clone();
            disclosure_based_usecase(
                &state.issuance_server_url,
                &usecase_id,
                &state.universal_link_base_url,
                language,
                &client_id,
                state.help_base_url.clone().into_inner(),
            )
        }
    })
}

fn disclosure_based_usecase(
    issuance_server_url: &BaseUrl,
    usecase: &str,
    universal_link_base_url: &BaseUrl,
    selected_lang: Language,
    client_id: &str,
    help_base_url: Url,
) -> Response {
    let universal_links =
        disclosure_based_issuance_universal_links(issuance_server_url, usecase, universal_link_base_url, client_id);
    UsecaseTemplate {
        usecase,
        same_device_ul: universal_links.get(&SessionType::SameDevice).unwrap().to_owned(),
        cross_device_ul: universal_links.get(&SessionType::CrossDevice).unwrap().to_owned(),
        help_base_url,
        wallet_web_sha256: &WALLET_WEB_JS_SHA256,
        base: BaseTemplate {
            selected_lang,
            trans: &TRANSLATIONS[selected_lang],
            available_languages: &Language::iter().collect_vec(),
            language_js_sha256: &LANGUAGE_JS_SHA256,
        },
    }
    .into_response()
}

async fn pre_authorized_usecase(
    usecase: &str,
    selected_lang: Language,
    data: &IssuableDocumentTemplates,
    offer_url: Url,
    help_base_url: Url,
) -> Result<Response> {
    let documents = data
        .iter()
        .map(|doc| {
            let (format, attestation_type, attributes) = doc.clone().into();
            IssuableDocument::try_new_with_random_id(format, attestation_type, attributes)
                .map_err(|err| web_utils::error::Error::from(anyhow::Error::from(err)))
        })
        .collect::<Result<Vec<_>>>()?
        .try_into()
        .unwrap(); // we started with a VecNonEmpty

    let offer_response = reqwest::Client::new()
        .post(offer_url)
        .json(&OfferRequest { documents })
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(anyhow::Error::from)?
        .json::<OfferResponse>()
        .await
        .map_err(anyhow::Error::from)?;

    Ok(UsecaseTemplate {
        usecase,
        same_device_ul: offer_response.credential_offer_url.clone(),
        cross_device_ul: offer_response.credential_offer_url,
        help_base_url,
        wallet_web_sha256: &WALLET_WEB_JS_SHA256,
        base: BaseTemplate {
            selected_lang,
            trans: &TRANSLATIONS[selected_lang],
            available_languages: &Language::iter().collect_vec(),
            language_js_sha256: &LANGUAGE_JS_SHA256,
        },
    }
    .into_response())
}

async fn attestation(
    State(state): State<Arc<ApplicationState>>,
    Path(usecase): Path<String>,
    Json(disclosed_attestations): Json<VecNonEmpty<DemoDisclosedAttestations>>,
) -> Result<Response> {
    let Some(Usecase::DisclosureBased { data, disclosed, .. }) = state.usecases.get(&usecase) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    // get the requested attribute from the disclosed attributes, ignore everything else as we trust the issuance_server
    // blindly
    let requested_path = disclosed.path.iter().map(String::as_str).collect::<Vec<_>>();
    let attribute_value = disclosed_attestations
        .iter()
        .flat_map(|demo_attestations| &demo_attestations.attestations)
        .filter(|attestation| attestation.attestation_type == disclosed.credential_type)
        .exactly_one()
        .ok()
        .and_then(|document| {
            document
                .attributes
                .flattened()
                .iter()
                .find_map(|(path, attribute_value)| {
                    (path.as_ref() == requested_path).then_some(attribute_value.to_owned())
                })
        })
        .ok_or(anyhow::Error::msg("invalid disclosure result"))?;

    let documents: Vec<IssuableDocument> = data
        .get(attribute_value)
        .map(|docs| {
            docs.iter()
                .cloned()
                .map(|doc| {
                    let (format, attestation_type, attribute) = doc.into();
                    IssuableDocument::try_new_with_random_id(format, attestation_type, attribute)
                        .map_err(|err| web_utils::error::Error::from(anyhow::Error::from(err)))
                })
                .collect::<Result<Vec<_>>>()
                .unwrap()
        })
        .unwrap_or_default();

    Ok(Json(documents).into_response())
}
