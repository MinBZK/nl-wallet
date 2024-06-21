use std::{collections::HashMap, env, path::PathBuf, result::Result as StdResult, sync::Arc};

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    handler::HandlerWithoutStateExt,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::warn;

use nl_wallet_mdoc::{server_state::SessionToken, verifier::DisclosedAttributes};
use wallet_common::config::wallet_config::BaseUrl;

use crate::{
    askama_axum,
    client::WalletServerClient,
    settings::{Origin, ReturnUrlMode, Settings, Usecase},
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
    usecases: HashMap<String, Usecase>,
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

pub fn create_router(settings: Settings) -> Router {
    let application_state = Arc::new(ApplicationState {
        client: WalletServerClient::new(settings.internal_wallet_server_url.clone()),
        public_wallet_server_url: settings.public_wallet_server_url,
        public_url: settings.public_url,
        usecases: settings.usecases,
    });

    let root_dir = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

    let mut app = Router::new()
        .route("/sessions", post(create_session))
        .route("/:usecase/", get(usecase))
        .route(&format!("/:usecase/{}", RETURN_URL_SEGMENT), get(disclosed_attributes))
        .fallback_service(
            ServeDir::new(root_dir.join("assets")).not_found_service({ StatusCode::NOT_FOUND }.into_service()),
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
    status_url: String,
    session_token: SessionToken,
}

#[derive(Template, Serialize)]
#[template(path = "disclosed/attributes.askama", escape = "html", ext = "html")]
struct DisclosureTemplate<'a> {
    usecase: &'a str,
    attributes: DisclosedAttributes,
}

#[derive(Template, Serialize)]
#[template(path = "usecase/usecase.askama", escape = "html", ext = "html")]
struct UsecaseTemplate<'a> {
    usecase: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisclosedAttributesParams {
    pub nonce: Option<String>,
    pub session_token: SessionToken,
}

#[derive(Deserialize, Serialize)]
struct EngagementUrls {
    status_url: String,
    disclosed_attributes_url: String,
}

async fn create_session(
    State(state): State<Arc<ApplicationState>>,
    Json(options): Json<SessionOptions>,
) -> Result<Json<SessionResponse>> {
    let usecase = state
        .usecases
        .get(&options.usecase)
        .ok_or(anyhow::Error::msg("usecase not found"))?;

    let session_token = state
        .client
        .start(
            options.usecase.clone(),
            usecase.items_requests.clone(),
            if usecase.return_url == ReturnUrlMode::None {
                None
            } else {
                Some(
                    format!(
                        "{}/{}?session_token={{session_token}}",
                        state.public_url.join(&options.usecase),
                        RETURN_URL_SEGMENT
                    )
                    .parse()
                    .expect("should always be a valid ReturnUrlTemplate"),
                )
            },
        )
        .await?;

    let result = SessionResponse {
        status_url: state
            .public_wallet_server_url
            .join(&format!("disclosure/{session_token}/status"))
            .to_string(),
        session_token,
    };
    Ok(result.into())
}

async fn usecase(Path(usecase): Path<String>) -> Result<Response> {
    let result = UsecaseTemplate { usecase: &usecase };

    Ok(askama_axum::into_response(&result))
}

async fn disclosed_attributes(
    State(state): State<Arc<ApplicationState>>,
    Path(usecase): Path<String>,
    Query(params): Query<DisclosedAttributesParams>,
) -> Result<Response> {
    let attributes = state
        .client
        .disclosed_attributes(params.session_token, params.nonce)
        .await?;

    let result = DisclosureTemplate {
        usecase: &usecase,
        attributes,
    };

    Ok(askama_axum::into_response(&result))
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
