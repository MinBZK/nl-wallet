use std::path::Path;
use std::sync::Arc;
use std::sync::LazyLock;

use askama::Template;
use askama_web::WebTemplate;
use axum::Form;
use axum::Router;
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
use server_utils::log_requests::log_request_response;
use strum::IntoEnumIterator;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use http_utils::health::create_health_router;
use utils::path::prefix_local_path;
use web_utils::LANGUAGE_JS_SHA256;
use web_utils::headers::set_content_security_policy;
use web_utils::headers::set_static_cache_control;
use web_utils::language::Language;

use crate::settings::Settings;
use crate::translations::TRANSLATIONS;
use crate::translations::Words;

struct ApplicationState {}

static CSP_HEADER: LazyLock<String> = LazyLock::new(|| {
    let script_src = format!("'sha256-{}'", *LANGUAGE_JS_SHA256);

    format!(
        "default-src 'self'; script-src 'self' {script_src}; img-src 'self' data:; font-src 'self' data:; form-action \
         'self'; frame-ancestors 'none'; object-src 'none'; base-uri 'none';"
    )
});

#[derive(Deserialize)]
struct DeleteForm {
    _revocation_code: String,
}

pub fn create_router(settings: &Settings) -> Router {
    let application_state = Arc::new(ApplicationState {});

    let mut app = Router::new()
        .route("/", get(index))
        .route("/", post(delete_wallet))
        .fallback_service(
            ServiceBuilder::new()
                .layer(middleware::from_fn(set_static_cache_control))
                .service(
                    ServeDir::new(prefix_local_path(Path::new("assets"))).fallback(
                        ServiceBuilder::new()
                            .service(ServeDir::new(prefix_local_path(Path::new("../lib/web_utils/assets"))))
                            .not_found_service({ StatusCode::NOT_FOUND }.into_service()),
                    ),
                ),
        )
        .with_state(Arc::clone(&application_state))
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(|req, next| {
            set_content_security_policy(req, next, &CSP_HEADER)
        }));

    if settings.log_requests {
        app = app.layer(axum::middleware::from_fn(log_request_response));
    }

    app.merge(create_health_router([]))
}

struct BaseTemplate<'a> {
    selected_lang: Language,
    trans: &'a Words<'a>,
    available_languages: &'a [Language],
    language_js_sha256: &'a str,
}

#[derive(Template, WebTemplate)]
#[template(path = "index.askama", escape = "html", ext = "html")]
struct IndexTemplate<'a> {
    base: BaseTemplate<'a>,
    failed_reason: Option<&'a str>,
}

async fn index(State(_state): State<Arc<ApplicationState>>, language: Language) -> Response {
    IndexTemplate {
        base: BaseTemplate {
            selected_lang: language,
            trans: &TRANSLATIONS[language],
            available_languages: &Language::iter().collect_vec(),
            language_js_sha256: &LANGUAGE_JS_SHA256,
        },
        failed_reason: None,
    }
    .into_response()
}

async fn delete_wallet(
    State(_state): State<Arc<ApplicationState>>,
    language: Language,
    Form(_delete_form): Form<DeleteForm>,
) -> Response {
    IndexTemplate {
        base: BaseTemplate {
            selected_lang: language,
            trans: &TRANSLATIONS[language],
            available_languages: &Language::iter().collect_vec(),
            language_js_sha256: &LANGUAGE_JS_SHA256,
        },
        failed_reason: Some("Wallet deletion is not supported yet."),
    }
    .into_response()
}
