use std::sync::Arc;

use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use axum::handler::HandlerWithoutStateExt;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use itertools::Itertools;
use strum::IntoEnumIterator;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use demo_utils::headers::set_static_cache_control;
use demo_utils::language::Language;
use demo_utils::LANGUAGE_JS_SHA256;
use utils::path::prefix_local_path;

use crate::settings::DemoService;
use crate::settings::Settings;
use crate::translations::Words;
use crate::translations::TRANSLATIONS;

struct ApplicationState {
    demo_services: Vec<DemoService>,
}

pub fn create_router(settings: Settings) -> Router {
    let application_state = Arc::new(ApplicationState {
        demo_services: settings.demo_services,
    });

    let app = Router::new()
        .route("/", get(index))
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

    app
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
    demo_services: &'a [DemoService],
    base: BaseTemplate<'a>,
}

#[axum::debug_handler]
async fn index(State(state): State<Arc<ApplicationState>>, language: Language) -> Response {
    IndexTemplate {
        demo_services: &state.demo_services,
        base: BaseTemplate {
            selected_lang: language,
            trans: &TRANSLATIONS[language],
            available_languages: &Language::iter().collect_vec(),
            language_js_sha256: &LANGUAGE_JS_SHA256,
        },
    }
    .into_response()
}
