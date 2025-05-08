use std::sync::Arc;
use std::sync::LazyLock;

use askama::Template;
use askama_web::WebTemplate;
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
use axum::Json;
use axum::Router;
use base64::prelude::*;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use strum::IntoEnumIterator;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::warn;
use url::Url;

use demo_utils::error::Result;
use demo_utils::headers::cors_layer;
use demo_utils::headers::set_static_cache_control;
use demo_utils::language::Language;
use demo_utils::language::LanguageParam;
use http_utils::urls::BaseUrl;
use mdoc::verifier::DisclosedAttributes;
use openid4vc::server_state::SessionToken;
use utils::path::prefix_local_path;

use crate::client::WalletServerClient;
use crate::settings::ReturnUrlMode;
use crate::settings::Settings;
use crate::settings::Usecase;
use crate::settings::WalletWeb;
use crate::translations::Words;
use crate::translations::TRANSLATIONS;

const RETURN_URL_SEGMENT: &str = "return";

struct ApplicationState {
    client: WalletServerClient,
    public_wallet_server_url: BaseUrl,
    public_url: BaseUrl,
    help_base_url: BaseUrl,
    usecases: IndexMap<String, Usecase>,
    wallet_web: WalletWeb,
}

pub fn create_router(settings: Settings) -> Router {
    let application_state = Arc::new(ApplicationState {
        client: WalletServerClient::new(settings.internal_wallet_server_url.clone()),
        public_wallet_server_url: settings.public_wallet_server_url,
        public_url: settings.public_url,
        help_base_url: settings.help_base_url,
        usecases: settings.usecases,
        wallet_web: settings.wallet_web,
    });

    let mut app = Router::new()
        .route("/sessions", post(create_session))
        .route("/{usecase}/", get(usecase))
        .route(
            &format!("/{{usecase}}/{}", RETURN_URL_SEGMENT),
            get(disclosed_attributes),
        )
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
        .with_state(application_state)
        .layer(TraceLayer::new_for_http());

    if let Some(cors_origin) = settings.allow_origins {
        app = app.layer(cors_layer(cors_origin));
    }

    app
}

#[derive(Serialize, Deserialize)]
struct SessionOptions {
    usecase: String,
}

#[derive(Serialize)]
struct SessionResponse {
    status_url: Url,
    session_token: SessionToken,
}

async fn create_session(
    State(state): State<Arc<ApplicationState>>,
    language: Language,
    Json(options): Json<SessionOptions>,
) -> Result<Json<SessionResponse>> {
    let usecase = state
        .usecases
        .get(&options.usecase)
        .ok_or(anyhow::Error::msg("usecase not found"))?;

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
        .start(
            options.usecase.clone(),
            usecase.items_requests.clone(),
            return_url_template,
        )
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
}

#[derive(Template, WebTemplate)]
#[template(path = "usecase/usecase.askama", escape = "html", ext = "html")]
struct UsecaseTemplate<'a> {
    usecase: &'a str,
    start_url: Url,
    help_base_url: Url,
    usecase_js_sha256: &'a str,
    wallet_web_filename: &'a str,
    wallet_web_sha256: &'a str,
    base: BaseTemplate<'a>,
}

static USECASE_JS_SHA256: LazyLock<String> =
    LazyLock::new(|| BASE64_STANDARD.encode(crypto::utils::sha256(include_bytes!("../assets/usecase.js"))));

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
    if !state.usecases.contains_key(&usecase) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let start_url = format_start_url(&state.public_url, language);
    UsecaseTemplate {
        usecase: &usecase,
        start_url,
        help_base_url: state.help_base_url.clone().into_inner(),
        usecase_js_sha256: &USECASE_JS_SHA256,
        wallet_web_filename: &state.wallet_web.filename.to_string_lossy(),
        wallet_web_sha256: &state.wallet_web.sha256,
        base: BaseTemplate {
            session_token: None,
            nonce: None,
            selected_lang: language,
            trans: &TRANSLATIONS[language],
            available_languages: &Language::iter().collect_vec(),
        },
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
    attributes: DisclosedAttributes,
    base: BaseTemplate<'a>,
}

async fn disclosed_attributes(
    State(state): State<Arc<ApplicationState>>,
    Path(usecase): Path<String>,
    Query(params): Query<DisclosedAttributesParams>,
    language: Language,
) -> Response {
    if !state.usecases.contains_key(&usecase) {
        return StatusCode::NOT_FOUND.into_response();
    }

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
    };

    match attributes {
        Ok(attributes) => DisclosedAttributesTemplate {
            usecase: &usecase,
            attributes,
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
                wallet_web_filename: &state.wallet_web.filename.to_string_lossy(),
                wallet_web_sha256: &state.wallet_web.sha256,
                base,
            }
            .into_response()
        }
    }
}

mod filters {
    use mdoc::verifier::DisclosedAttributes;

    pub fn attribute(attributes: &DisclosedAttributes, _: &dyn askama::Values, name: &str) -> askama::Result<String> {
        for doctype in attributes {
            for namespace in &doctype.1.attributes {
                for (attribute_name, attribute_value) in namespace.1 {
                    if attribute_name == name {
                        return Ok(attribute_value
                            .as_text()
                            .ok_or(askama::Error::custom("could not format attribute_value as text"))?
                            .to_owned());
                    }
                }
            }
        }

        Err(askama::Error::custom("attribute '{name}' cannot be found"))
    }
}
