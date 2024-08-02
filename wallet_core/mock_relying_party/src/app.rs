use std::{
    env,
    path::PathBuf,
    result::Result as StdResult,
    sync::{Arc, LazyLock},
};

use askama::Template;
use axum::{
    extract::{Path, Query, Request, State},
    handler::HandlerWithoutStateExt,
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::prelude::*;
use http::{header::CACHE_CONTROL, HeaderValue};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::warn;
use url::Url;

use nl_wallet_mdoc::{server_state::SessionToken, verifier::DisclosedAttributes};
use wallet_common::{config::wallet_config::BaseUrl, utils::sha256};

use crate::{
    askama_axum,
    client::WalletServerClient,
    settings::{Origin, ReturnUrlMode, Settings, Usecase, WalletWeb},
    translations::{Words, TRANSLATIONS},
};

#[derive(Debug)]
pub struct Error(anyhow::Error);

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self(error)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        warn!("error result: {:?}", self);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
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

fn cors_layer(allow_origins: Vec<Origin>) -> Option<CorsLayer> {
    if allow_origins.is_empty() {
        return None;
    }

    let layer = CorsLayer::new()
        .allow_origin(
            allow_origins
                .into_iter()
                .map(|url| {
                    url.try_into()
                        .expect("cross_origin base_url should be parseable to header value")
                })
                .collect::<Vec<_>>(),
        )
        .allow_headers(Any)
        .allow_methods([Method::GET, Method::POST]);

    Some(layer)
}

async fn set_static_cache_control(request: Request, next: Next) -> Response {
    // only cache images and fonts, not CSS and JS (except wallet_web, as that is suffixed with a hash)
    let set_no_store = !request.uri().path().ends_with(".iife.js")
        && [".css", ".js"].iter().any(|ext| request.uri().path().ends_with(ext));
    let mut response = next.run(request).await;
    if set_no_store {
        response
            .headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
    response
}

pub fn create_router(settings: Settings) -> Router {
    let application_state = Arc::new(ApplicationState {
        client: WalletServerClient::new(settings.internal_wallet_server_url.clone()),
        public_wallet_server_url: settings.public_wallet_server_url,
        public_url: settings.public_url,
        usecases: settings.usecases,
        wallet_web: settings.wallet_web,
    });

    let root_dir = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

    let mut app = Router::new()
        .route("/", get(index))
        .route("/sessions", post(create_session))
        .route("/:usecase/", get(usecase))
        .route(&format!("/:usecase/{}", RETURN_URL_SEGMENT), get(disclosed_attributes))
        .fallback_service(
            ServiceBuilder::new()
                .layer(middleware::from_fn(set_static_cache_control))
                .service(
                    ServeDir::new(root_dir.join("assets")).not_found_service({ StatusCode::NOT_FOUND }.into_service()),
                ),
        )
        .with_state(application_state)
        .layer(TraceLayer::new_for_http());

    if let Some(cors) = cors_layer(settings.allow_origins) {
        app = app.layer(cors)
    }

    app
}

#[derive(Deserialize, Serialize)]
struct SessionOptions {
    usecase: String,
}

#[derive(Serialize)]
struct SessionResponse {
    status_url: Url,
    session_token: SessionToken,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash, strum::Display, strum::EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Language {
    #[default]
    Nl,
    En,
}

async fn create_session(
    State(state): State<Arc<ApplicationState>>,
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
                "{}/{}?session_token={{session_token}}",
                state.public_url.join(&options.usecase),
                RETURN_URL_SEGMENT
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

#[derive(Debug, Deserialize)]
pub struct IndexParams {
    pub lang: Option<Language>,
}

#[derive(Template, Serialize)]
#[template(path = "index.askama", escape = "html", ext = "html")]
struct IndexTemplate<'a> {
    usecases: &'a [&'a str],
    language: Language,
    t: &'a Words<'a>,
}

async fn index(State(state): State<Arc<ApplicationState>>, Query(params): Query<IndexParams>) -> Result<Response> {
    let language = params.lang.unwrap_or_default();
    let t = TRANSLATIONS.get(&language).unwrap(); // TODO unwrap?
    let result = IndexTemplate {
        usecases: &state.usecases.keys().map(|s| s.as_str()).collect::<Vec<_>>(),
        language,
        t,
    };

    Ok(askama_axum::into_response(&result))
}

#[derive(Debug, Deserialize)]
pub struct UsecaseParams {
    pub lang: Option<Language>,
}

#[derive(Template, Serialize)]
#[template(path = "usecase/usecase.askama", escape = "html", ext = "html")]
struct UsecaseTemplate<'a> {
    usecase: &'a str,
    usecase_js_sha256: &'a str,
    wallet_web_filename: &'a str,
    wallet_web_sha256: &'a str,
    error: Option<&'a str>,
    language: Language,
    t: &'a Words<'a>,
}

static USECASE_JS_SHA256: LazyLock<String> =
    LazyLock::new(|| BASE64_STANDARD.encode(sha256(include_bytes!("../assets/usecase.js"))));

async fn usecase(
    State(state): State<Arc<ApplicationState>>,
    Path(usecase): Path<String>,
    Query(params): Query<UsecaseParams>,
) -> Result<Response> {
    if !state.usecases.contains_key(&usecase) {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let language = params.lang.unwrap_or_default();
    let t = TRANSLATIONS.get(&language).unwrap(); // TODO unwrap?
    let result = UsecaseTemplate {
        usecase: &usecase,
        usecase_js_sha256: &USECASE_JS_SHA256,
        wallet_web_filename: &state.wallet_web.filename.to_string_lossy(),
        wallet_web_sha256: &state.wallet_web.sha256,
        error: None,
        language,
        t,
    };

    Ok(askama_axum::into_response(&result))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisclosedAttributesParams {
    pub nonce: Option<String>,
    pub session_token: SessionToken,
    pub lang: Option<Language>,
}

#[derive(Template, Serialize)]
#[template(path = "disclosed/attributes.askama", escape = "html", ext = "html")]
struct DisclosedAttributesTemplate<'a> {
    usecase: &'a str,
    attributes: DisclosedAttributes,
    language: Language,
    t: &'a Words<'a>,
}

async fn disclosed_attributes(
    State(state): State<Arc<ApplicationState>>,
    Path(usecase): Path<String>,
    Query(params): Query<DisclosedAttributesParams>,
) -> Result<Response> {
    if !state.usecases.contains_key(&usecase) {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let attributes = state
        .client
        .disclosed_attributes(params.session_token, params.nonce)
        .await;

    let language = params.lang.unwrap_or_default();
    let t = TRANSLATIONS.get(&language).unwrap(); // TODO unwrap?
    match attributes {
        Ok(attributes) => {
            let result = DisclosedAttributesTemplate {
                usecase: &usecase,
                attributes,
                language,
                t,
            };
            Ok(askama_axum::into_response(&result))
        }
        Err(err) => {
            let err = err.to_string();
            let result = UsecaseTemplate {
                usecase: &usecase,
                usecase_js_sha256: &USECASE_JS_SHA256,
                wallet_web_filename: &state.wallet_web.filename.to_string_lossy(),
                wallet_web_sha256: &state.wallet_web.sha256,
                error: Some(&err),
                language,
                t,
            };
            Ok(askama_axum::into_response(&result))
        }
    }
}

mod filters {
    use nl_wallet_mdoc::verifier::DisclosedAttributes;

    pub fn attribute(attributes: &DisclosedAttributes, name: &str) -> ::askama::Result<String> {
        for doctype in attributes {
            for namespace in doctype.1.attributes.iter() {
                for attribute in namespace.1 {
                    if attribute.name == name {
                        return Ok(attribute.value.as_text().unwrap().to_owned());
                    }
                }
            }
        }

        Ok(format!("attribute '{name}' cannot be found"))
    }
}
