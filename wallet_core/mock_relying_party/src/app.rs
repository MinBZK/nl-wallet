use std::{
    env,
    path::PathBuf,
    result::Result as StdResult,
    str::FromStr,
    sync::{Arc, LazyLock},
};

use askama::Template;
use axum::{
    async_trait,
    extract::{FromRequestParts, Path, Query, Request, State},
    handler::HandlerWithoutStateExt,
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::prelude::*;
use http::{
    header::{ACCEPT_LANGUAGE, CACHE_CONTROL},
    request::Parts,
    HeaderMap, HeaderValue,
};
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

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
)]
pub enum Language {
    #[default]
    #[serde(rename = "nl")]
    #[strum(to_string = "nl", serialize = "nl-BE", serialize = "nl-NL")]
    Nl,
    #[serde(rename = "en")]
    #[strum(
        to_string = "en",
        serialize = "en-AU",
        serialize = "en-CA",
        serialize = "en-GB",
        serialize = "en-US"
    )]
    En,
}

impl Language {
    fn match_accept_language(headers: &HeaderMap) -> Option<Self> {
        let accept_language = headers.get(ACCEPT_LANGUAGE)?;
        let languages = accept_language::parse(accept_language.to_str().ok()?);

        // applies function to the elements of iterator and returns the first non-None result
        languages.into_iter().find_map(|l| Language::from_str(&l).ok())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageParam {
    pub lang: Language,
}

#[async_trait]
impl<S> FromRequestParts<S> for Language
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> std::result::Result<Self, Self::Rejection> {
        let query = Query::<LanguageParam>::from_request_parts(parts, state).await;
        if let Ok(params) = query {
            Ok(params.lang)
        } else if let Some(lang) = Language::match_accept_language(&parts.headers) {
            Ok(lang)
        } else {
            Ok(Language::default())
        }
    }
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

#[derive(Template, Serialize)]
#[template(path = "index.askama", escape = "html", ext = "html")]
struct IndexTemplate<'a> {
    usecases: &'a [&'a str],
    session_token: Option<SessionToken>,
    nonce: Option<String>,
    language: Language,
    t: &'a Words<'a>,
}

async fn index(State(state): State<Arc<ApplicationState>>, language: Language) -> Result<Response> {
    let t = TRANSLATIONS
        .get(&language)
        .ok_or(anyhow::Error::msg("translations for language not found"))?;
    let result = IndexTemplate {
        usecases: &state.usecases.keys().map(|s| s.as_str()).collect::<Vec<_>>(),
        session_token: None,
        nonce: None,
        language,
        t,
    };

    Ok(askama_axum::into_response(&result))
}

#[derive(Template, Serialize)]
#[template(path = "usecase/usecase.askama", escape = "html", ext = "html")]
struct UsecaseTemplate<'a> {
    usecase: &'a str,
    start_url: Url,
    usecase_js_sha256: &'a str,
    wallet_web_filename: &'a str,
    wallet_web_sha256: &'a str,
    session_token: Option<SessionToken>,
    nonce: Option<String>,
    language: Language,
    t: &'a Words<'a>,
}

static USECASE_JS_SHA256: LazyLock<String> =
    LazyLock::new(|| BASE64_STANDARD.encode(sha256(include_bytes!("../assets/usecase.js"))));

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
    let t = TRANSLATIONS
        .get(&language)
        .ok_or(anyhow::Error::msg("translations for language not found"))?;
    let result = UsecaseTemplate {
        usecase: &usecase,
        start_url,
        usecase_js_sha256: &USECASE_JS_SHA256,
        wallet_web_filename: &state.wallet_web.filename.to_string_lossy(),
        wallet_web_sha256: &state.wallet_web.sha256,
        session_token: None,
        nonce: None,
        language,
        t,
    };

    Ok(askama_axum::into_response(&result))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisclosedAttributesParams {
    pub nonce: Option<String>,
    pub session_token: SessionToken,
}

#[derive(Template, Serialize)]
#[template(path = "disclosed/attributes.askama", escape = "html", ext = "html")]
struct DisclosedAttributesTemplate<'a> {
    usecase: &'a str,
    attributes: DisclosedAttributes,
    session_token: SessionToken,
    nonce: Option<String>,
    language: Language,
    t: &'a Words<'a>,
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
    let t = TRANSLATIONS
        .get(&language)
        .ok_or(anyhow::Error::msg("translations for language not found"))?;
    match attributes {
        Ok(attributes) => {
            let result = DisclosedAttributesTemplate {
                usecase: &usecase,
                attributes,
                session_token: params.session_token,
                nonce: params.nonce,
                language,
                t,
            };
            Ok(askama_axum::into_response(&result))
        }
        Err(_) => {
            let result = UsecaseTemplate {
                usecase: &usecase,
                start_url,
                usecase_js_sha256: &USECASE_JS_SHA256,
                wallet_web_filename: &state.wallet_web.filename.to_string_lossy(),
                wallet_web_sha256: &state.wallet_web.sha256,
                session_token: Some(params.session_token),
                nonce: params.nonce,
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
