use std::result::Result as StdResult;
use std::sync::Arc;
use std::sync::LazyLock;

use askama::Template;
use axum::extract::FromRequestParts;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::Request;
use axum::extract::State;
use axum::handler::HandlerWithoutStateExt;
use axum::http::Method;
use axum::http::StatusCode;
use axum::middleware;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;
use base64::prelude::*;
use http::header::ACCEPT_LANGUAGE;
use http::header::CACHE_CONTROL;
use http::request::Parts;
use http::HeaderMap;
use http::HeaderValue;
use indexmap::IndexMap;
use nutype::nutype;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use strum::IntoEnumIterator;
use tower::ServiceBuilder;
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::warn;
use url::Url;

use mdoc::verifier::DisclosedAttributes;
use openid4vc::server_state::SessionToken;
use wallet_common::urls::BaseUrl;
use wallet_common::urls::CorsOrigin;
use wallet_common::utils;

use crate::askama_axum;
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

fn cors_layer(allow_origins: CorsOrigin) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(allow_origins)
        .allow_headers(Any)
        .allow_methods([Method::GET, Method::POST])
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
                    ServeDir::new(utils::prefix_local_path("assets".as_ref()).as_ref())
                        .not_found_service({ StatusCode::NOT_FOUND }.into_service()),
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

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    SerializeDisplay,
    DeserializeFromStr,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
)]
pub enum Language {
    #[default]
    #[strum(to_string = "nl")]
    Nl,
    #[strum(to_string = "en")]
    En,
}

impl Language {
    fn parse(s: &str) -> Option<Self> {
        match s.split('-').next() {
            Some("en") => Some(Language::En),
            Some("nl") => Some(Language::Nl),
            _ => None,
        }
    }

    fn match_accept_language(headers: &HeaderMap) -> Option<Self> {
        let accept_language = headers.get(ACCEPT_LANGUAGE)?;
        let languages = accept_language::parse(accept_language.to_str().ok()?);

        // applies function to the elements of iterator and returns the first non-None result
        languages.into_iter().find_map(|l| Language::parse(&l))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageParam {
    pub lang: Language,
}

impl<S> FromRequestParts<S> for Language
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> std::result::Result<Self, Self::Rejection> {
        let lang = Query::<LanguageParam>::from_request_parts(parts, state)
            .await
            .map(|l| l.lang)
            .unwrap_or(Language::match_accept_language(&parts.headers).unwrap_or_default());
        Ok(lang)
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

#[cfg(test)]
mod test {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("en", Some(Language::En))]
    #[case("nl", Some(Language::Nl))]
    #[case("123", None)]
    #[case("en-GB", Some(Language::En))]
    #[case("nl-NL", Some(Language::Nl))]
    fn test_parse_language(#[case] s: &str, #[case] expected: Option<Language>) {
        assert_eq!(Language::parse(s), expected);
    }
}
