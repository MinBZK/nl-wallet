use std::sync::Arc;
use std::sync::LazyLock;

use askama::Template;
use askama_web::WebTemplate;
use axum::extract::Path;
use axum::extract::State;
use axum::handler::HandlerWithoutStateExt;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;
use base64::prelude::*;
use indexmap::IndexMap;
use itertools::Itertools;
use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;
use strum::IntoEnumIterator;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::warn;
use url::Url;

use demo_utils::headers::set_static_cache_control;
use demo_utils::language::Language;
use http_utils::urls::disclosure_based_issuance_base_uri;
use http_utils::urls::BaseUrl;
use http_utils::urls::DEFAULT_UNIVERSAL_LINK_BASE;
use mdoc::verifier::DocumentDisclosedAttributes;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::openid4vp::RequestUriMethod;
use openid4vc::openid4vp::VpRequestUriObject;
use openid4vc::verifier::SessionType;
use openid4vc::verifier::VerifierUrlParameters;
use utils::path::prefix_local_path;

use crate::settings::Settings;
use crate::settings::Usecase;
use crate::settings::WalletWeb;
use crate::translations::Words;
use crate::translations::TRANSLATIONS;

#[nutype(derive(Debug, From, AsRef))]
pub struct Error(anyhow::Error);

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        warn!("error result: {:?}", self);
        (StatusCode::INTERNAL_SERVER_ERROR, self.as_ref().to_string()).into_response()
    }
}

struct ApplicationState {
    usecases: IndexMap<String, Usecase>,
    wallet_web: WalletWeb,
    issuance_server_url: BaseUrl,
}

pub fn create_routers(settings: Settings) -> (Router, Router) {
    let application_state = Arc::new(ApplicationState {
        usecases: settings.usecases,
        wallet_web: settings.wallet_web,
        issuance_server_url: settings.issuance_server_url,
    });

    let app = Router::new()
        .route("/{usecase}/", get(usecase))
        .fallback_service(
            ServiceBuilder::new()
                .layer(middleware::from_fn(set_static_cache_control))
                .service(
                    ServeDir::new(prefix_local_path("assets".as_ref())).fallback(
                        ServiceBuilder::new()
                            .service(ServeDir::new(prefix_local_path("../demo_utils/assets".as_ref())))
                            .not_found_service({ StatusCode::NOT_FOUND }.into_service()),
                    ),
                ),
        )
        .with_state(Arc::clone(&application_state))
        .layer(TraceLayer::new_for_http());

    let attestation_router = Router::new()
        .route("/{usecase}/", post(attestation))
        .with_state(application_state)
        .layer(TraceLayer::new_for_http());

    (app, attestation_router)
}

#[derive(Serialize, Deserialize)]
struct SessionOptions {
    usecase: String,
}

struct BaseTemplate<'a> {
    selected_lang: Language,
    trans: &'a Words<'a>,
    available_languages: &'a [Language],
}

#[derive(Template, WebTemplate)]
#[template(path = "usecase/usecase.askama", escape = "html", ext = "html")]
struct UsecaseTemplate<'a> {
    usecase: &'a str,
    universal_links: (Url, Url),
    usecase_js_sha256: &'a str,
    wallet_web_filename: &'a str,
    wallet_web_sha256: &'a str,
    base: BaseTemplate<'a>,
}

static USECASE_JS_SHA256: LazyLock<String> = LazyLock::new(|| {
    BASE64_STANDARD.encode(crypto::utils::sha256(include_bytes!(
        "../../demo_utils/assets/usecase.js"
    )))
});

fn disclosure_based_issuance_universal_links(issuance_server_url: &BaseUrl) -> (Url, Url) {
    SessionType::iter()
        .map(|session_type| {
            let params = serde_urlencoded::to_string(VerifierUrlParameters {
                session_type,
                ephemeral_id_params: None,
            })
            .unwrap();

            let mut issuance_server_url = issuance_server_url.join("/disclosure/disclosure_based_issuance/request_uri");
            issuance_server_url.set_query(Some(&params));

            let query = serde_urlencoded::to_string(VpRequestUriObject {
                request_uri: issuance_server_url.try_into().unwrap(),
                request_uri_method: Some(RequestUriMethod::POST),
                client_id: "disclosure_based_issuance.example.com".to_string(), // TODO
            })
            .unwrap();

            let mut uri =
                disclosure_based_issuance_base_uri(&DEFAULT_UNIVERSAL_LINK_BASE.parse().unwrap()).into_inner();
            uri.set_query(Some(&query));
            uri
        })
        .collect_tuple()
        .unwrap()
}

async fn usecase(
    State(state): State<Arc<ApplicationState>>,
    Path(usecase): Path<String>,
    language: Language,
) -> Response {
    if !state.usecases.contains_key(&usecase) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let universal_links = disclosure_based_issuance_universal_links(&state.issuance_server_url);
    UsecaseTemplate {
        usecase: &usecase,
        universal_links,
        usecase_js_sha256: &USECASE_JS_SHA256,
        wallet_web_filename: &state.wallet_web.filename.to_string_lossy(),
        wallet_web_sha256: &state.wallet_web.sha256,
        base: BaseTemplate {
            selected_lang: language,
            trans: &TRANSLATIONS[language],
            available_languages: &Language::iter().collect_vec(),
        },
    }
    .into_response()
}

async fn attestation(
    State(state): State<Arc<ApplicationState>>,
    Path(usecase): Path<String>,
    Json(_disclosed): Json<IndexMap<String, DocumentDisclosedAttributes>>,
) -> Response {
    let Some(usecase) = state.usecases.get(&usecase) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    // TODO add attributes for the lookup to be based upon
    let documents: Vec<IssuableDocument> = usecase
        .data
        .get("999991772")
        .map(|docs| docs.clone().into_inner())
        .unwrap_or_default();

    Json(documents).into_response()
}
