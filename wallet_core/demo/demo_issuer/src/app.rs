use std::collections::HashMap;
use std::sync::Arc;

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
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use strum::IntoEnumIterator;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use url::Url;

use demo_utils::error::Result;
use demo_utils::headers::set_static_cache_control;
use demo_utils::language::Language;
use http_utils::urls::disclosure_based_issuance_base_uri;
use http_utils::urls::BaseUrl;
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

struct ApplicationState {
    usecases: IndexMap<String, Usecase>,
    issuance_server_url: BaseUrl,
    universal_link_base_url: BaseUrl,
    wallet_web: WalletWeb,
}

pub fn create_routers(settings: Settings) -> (Router, Router) {
    let application_state = Arc::new(ApplicationState {
        usecases: settings.usecases,
        issuance_server_url: settings.issuance_server_url,
        universal_link_base_url: settings.universal_link_base_url,
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
    same_device_ul: Url,
    cross_device_ul: Url,
    wallet_web_filename: &'a str,
    wallet_web_sha256: &'a str,
    base: BaseTemplate<'a>,
}

fn disclosure_based_issuance_universal_links(
    issuance_server_url: &BaseUrl,
    usecase: &str,
    universal_link_base: &BaseUrl,
    client_id: &str,
) -> HashMap<SessionType, Url> {
    SessionType::iter()
        .map(|session_type| {
            let params = serde_urlencoded::to_string(VerifierUrlParameters {
                session_type,
                ephemeral_id_params: None,
            })
            .unwrap();

            let mut issuance_server_url = issuance_server_url.join(&format!("/disclosure/{usecase}/request_uri"));
            issuance_server_url.set_query(Some(&params));

            let query = serde_urlencoded::to_string(VpRequestUriObject {
                request_uri: issuance_server_url.try_into().unwrap(),
                request_uri_method: Some(RequestUriMethod::POST),
                client_id: client_id.to_owned(),
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
) -> Response {
    let Some(usecase) = state.usecases.get(&usecase_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let universal_links = disclosure_based_issuance_universal_links(
        &state.issuance_server_url,
        &usecase_id,
        &state.universal_link_base_url,
        &usecase.client_id,
    );
    UsecaseTemplate {
        usecase: &usecase_id,
        same_device_ul: universal_links.get(&SessionType::SameDevice).unwrap().to_owned(),
        cross_device_ul: universal_links.get(&SessionType::CrossDevice).unwrap().to_owned(),
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
    Json(disclosed): Json<IndexMap<String, DocumentDisclosedAttributes>>,
) -> Result<Response> {
    let Some(usecase) = state.usecases.get(&usecase) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    // get the bsn from the disclosed attributes, ignore everything else as we trust the issuance_server blindly
    let bsn = disclosed
        .get(&usecase.disclosed.0)
        .and_then(|document| document.attributes.get(&usecase.disclosed.1))
        .and_then(|attributes| attributes.get(&usecase.disclosed.2))
        .ok_or(anyhow::Error::msg("invalid disclosure result"))?
        .as_text()
        .unwrap();

    let documents: Vec<IssuableDocument> = usecase
        .data
        .get(bsn)
        .map(|docs| docs.clone().into_inner())
        .unwrap_or_default();

    Ok(Json(documents).into_response())
}
