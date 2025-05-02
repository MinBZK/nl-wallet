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
use mdoc::verifier::DocumentDisclosedAttributes;
use openid4vc::issuable_document::IssuableDocument;
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
}

pub fn create_routers(settings: Settings) -> (Router, Router) {
    let application_state = Arc::new(ApplicationState {
        usecases: settings.usecases,
        wallet_web: settings.wallet_web,
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
    start_url: Url,
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

async fn usecase(
    State(state): State<Arc<ApplicationState>>,
    Path(usecase): Path<String>,
    language: Language,
) -> Response {
    if !state.usecases.contains_key(&usecase) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let start_url = "https://example.com/start_url_here".parse().unwrap(); // TODO
    UsecaseTemplate {
        usecase: &usecase,
        start_url,
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

    dbg!(&state.usecases);

    // TODO add attributes for the lookup to be based upon
    let documents: Vec<IssuableDocument> = usecase
        .data
        .get("9999999999")
        .map(|docs| docs.clone().into_inner())
        .unwrap_or_default();

    // let documents: Vec<IssuableDocument> = vec![IssuableDocument::try_new(
    //     usecase.attestation_type.clone(),
    //     IndexMap::from_iter(vec![
    //         (
    //             "city".to_string(),
    //             Attribute::Single(AttributeValue::Text("The Capital".to_string())),
    //         ),
    //         (
    //             "street".to_string(),
    //             Attribute::Single(AttributeValue::Text("Main St.".to_string())),
    //         ),
    //         (
    //             "house".to_string(),
    //             Attribute::Single(AttributeValue::Text("Main St.".to_string())),
    //         ),
    //     ]),
    // )
    // .unwrap()]
    // .try_into()
    // .unwrap();

    Json(documents).into_response()
}
