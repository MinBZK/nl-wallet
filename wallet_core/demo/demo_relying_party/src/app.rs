use std::result::Result as StdResult;
use std::sync::Arc;
use std::sync::LazyLock;

use askama::Template;
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
use demo_utils::headers::cors_layer;
use demo_utils::headers::set_static_cache_control;
use demo_utils::language::LanguageParam;
use indexmap::IndexMap;
use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;
use strum::IntoEnumIterator;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::warn;
use url::Url;

use demo_utils::askama_axum;
use demo_utils::language::Language;
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

#[nutype(derive(Debug, From, AsRef))]
pub struct Error(anyhow::Error);

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        warn!("error result: {:?}", self);
        (StatusCode::INTERNAL_SERVER_ERROR, self.as_ref().to_string()).into_response()
    }
}

type Result<T> = StdResult<T, Error>;

const RETURN_URL_SEGMENT: &str = "return";

struct ApplicationState {
    client: WalletServerClient,
    public_wallet_server_url: BaseUrl,
    public_url: BaseUrl,
    usecases: IndexMap<String, Usecase>,
    wallet_web: WalletWeb,
}

pub fn create_router(settings: Settings) -> Router {
    let application_state = Arc::new(ApplicationState {
        client: WalletServerClient::new(settings.internal_wallet_server_url.clone()),
        public_wallet_server_url: settings.public_wallet_server_url,
        public_url: settings.public_url,
        usecases: settings.usecases,
        wallet_web: settings.wallet_web,
    });

    let mut app = Router::new()
        .route("/", get(index))
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
    nonce: Option<String>,
    selected_lang: Language,
    trans: &'a Words<'a>,
    available_languages: &'a Vec<Language>,
}

#[derive(Template)]
#[template(path = "index.askama", escape = "html", ext = "html")]
struct IndexTemplate<'a> {
    usecases: &'a [&'a str],
    base: BaseTemplate<'a>,
}

async fn index(State(state): State<Arc<ApplicationState>>, language: Language) -> Result<Response> {
    let result = IndexTemplate {
        usecases: &state.usecases.keys().map(|s| s.as_str()).collect::<Vec<_>>(),
        base: BaseTemplate {
            session_token: None,
            nonce: None,
            selected_lang: language,
            trans: &TRANSLATIONS[language],
            available_languages: &Language::iter().collect(),
        },
    };

    Ok(askama_axum::into_response(&result))
}

#[derive(Template)]
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
) -> Result<Response> {
    if !state.usecases.contains_key(&usecase) {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let start_url = format_start_url(&state.public_url, language);
    let result = UsecaseTemplate {
        usecase: &usecase,
        start_url,
        usecase_js_sha256: &USECASE_JS_SHA256,
        wallet_web_filename: &state.wallet_web.filename.to_string_lossy(),
        wallet_web_sha256: &state.wallet_web.sha256,
        base: BaseTemplate {
            session_token: None,
            nonce: None,
            selected_lang: language,
            trans: &TRANSLATIONS[language],
            available_languages: &Language::iter().collect(),
        },
    };

    Ok(askama_axum::into_response(&result))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisclosedAttributesParams {
    pub nonce: Option<String>,
    pub session_token: SessionToken,
}

#[derive(Template)]
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
) -> Result<Response> {
    if !state.usecases.contains_key(&usecase) {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let attributes = state
        .client
        .disclosed_attributes(params.session_token.clone(), params.nonce.clone())
        .await;

    let start_url = format_start_url(&state.public_url, language);
    let base = BaseTemplate {
        session_token: Some(params.session_token),
        nonce: params.nonce,
        selected_lang: language,
        trans: &TRANSLATIONS[language],
        available_languages: &Language::iter().collect(),
    };

    match attributes {
        Ok(attributes) => {
            let result = DisclosedAttributesTemplate {
                usecase: &usecase,
                attributes,
                base,
            };
            Ok(askama_axum::into_response(&result))
        }
        Err(err) => {
            warn!("Error getting disclosed attributes: {err}");
            let result = UsecaseTemplate {
                usecase: &usecase,
                start_url,
                usecase_js_sha256: &USECASE_JS_SHA256,
                wallet_web_filename: &state.wallet_web.filename.to_string_lossy(),
                wallet_web_sha256: &state.wallet_web.sha256,
                base,
            };
            Ok(askama_axum::into_response(&result))
        }
    }
}

mod filters {
    use mdoc::verifier::DisclosedAttributes;

    #[allow(clippy::unnecessary_wraps)]
    pub fn attribute(attributes: &DisclosedAttributes, name: &str) -> ::askama::Result<String> {
        for doctype in attributes {
            for namespace in &doctype.1.attributes {
                for (attribute_name, attribute_value) in namespace.1 {
                    if attribute_name == name {
                        return Ok(attribute_value.as_text().unwrap().to_owned());
                    }
                }
            }
        }

        Ok(format!("attribute '{name}' cannot be found"))
    }
}
