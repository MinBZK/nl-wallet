use std::{collections::HashMap, env, path::PathBuf, result::Result as StdResult, sync::Arc};

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    handler::HandlerWithoutStateExt,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Form, Json, Router,
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::warn;
use url::Url;

use nl_wallet_mdoc::{
    server_state::SessionToken,
    verifier::{DisclosedAttributes, ItemsRequests, SessionType},
};
use wallet_common::config::wallet_config::BaseUrl;
use wallet_server::verifier::StatusParams;

use crate::{
    askama_axum,
    client::WalletServerClient,
    settings::{Origin, Settings},
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

struct ApplicationState {
    client: WalletServerClient,
    public_url: BaseUrl,
    usecases: HashMap<String, ItemsRequests>,
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

pub async fn create_router(settings: Settings) -> anyhow::Result<Router> {
    let application_state = Arc::new(ApplicationState {
        client: WalletServerClient::new(settings.wallet_server_url.clone()),
        public_url: settings.public_url,
        usecases: settings.usecases,
    });

    let root_dir = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

    let mut app = Router::new()
        .route("/", get(index))
        .route("/", post(engage))
        .route("/engagement", post(create_engagement))
        .route(
            "/disclosure/sessions/:session_token/disclosed_attributes",
            get(disclosed_attributes),
        )
        .fallback_service(
            ServeDir::new(root_dir.join("assets")).not_found_service({ StatusCode::NOT_FOUND }.into_service()),
        )
        .with_state(application_state)
        .layer(TraceLayer::new_for_http());

    if let Some(cors) = cors_layer(settings.allow_origins) {
        app = app.layer(cors)
    }

    Ok(app)
}

#[derive(Deserialize, Serialize)]
struct SelectForm {
    session_type: MrpSessionType,
    usecase: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, strum::EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
enum MrpSessionType {
    SameDevice,
    SameDeviceNoReturn,
    CrossDevice,
    CrossDeviceWithReturn,
}

#[derive(Template, Serialize)]
#[template(path = "disclosure.html")]
struct DisclosureTemplate {
    urls: Option<EngagementUrls>,
    selected: Option<SelectForm>,
    usecases: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct EngagementUrls {
    status_url: String,
    disclosed_attributes_url: String,
}

impl From<MrpSessionType> for SessionType {
    fn from(value: MrpSessionType) -> Self {
        match value {
            MrpSessionType::SameDevice | MrpSessionType::SameDeviceNoReturn => Self::SameDevice,
            MrpSessionType::CrossDevice | MrpSessionType::CrossDeviceWithReturn => Self::CrossDevice,
        }
    }
}

async fn index(State(state): State<Arc<ApplicationState>>) -> Result<Response> {
    Ok(askama_axum::into_response(&DisclosureTemplate {
        urls: None,
        selected: None,
        usecases: state.usecases.keys().cloned().collect(),
    }))
}

async fn engage(State(state): State<Arc<ApplicationState>>, Form(selected): Form<SelectForm>) -> Result<Response> {
    let result = start_engagement(state, selected).await?;
    Ok(askama_axum::into_response(&result))
}

async fn create_engagement(
    State(state): State<Arc<ApplicationState>>,
    Json(selected): Json<SelectForm>,
) -> Result<Json<DisclosureTemplate>> {
    let result = start_engagement(state, selected).await?;
    Ok(result.into())
}

async fn start_engagement(state: Arc<ApplicationState>, selected: SelectForm) -> Result<DisclosureTemplate> {
    // return URL is just http://public.url/#{session_token}
    let return_url_template = match selected.session_type {
        MrpSessionType::CrossDeviceWithReturn | MrpSessionType::SameDevice => Some(
            format!("{}#{{session_token}}", state.public_url)
                .parse()
                .expect("should always be a valid ReturnUrlTemplate"),
        ),
        _ => None,
    };

    let (mut status_url, disclosed_attributes_url) = state
        .client
        .start(
            selected.usecase.clone(),
            state
                .usecases
                .get(&selected.usecase)
                .ok_or(anyhow::Error::msg("usecase not found"))?
                .clone(),
            return_url_template,
        )
        .await?;

    // For now just make the `session_type` part of the status URL that we send to the frontend.
    // In a later iteration the frontend should be able to choose this freely.
    let query = serde_urlencoded::to_string(StatusParams {
        session_type: selected.session_type.into(),
    })
    .expect("all variants of SessionType should convert into query parameters");
    status_url.set_query(query.as_str().into());

    let mrp_disclosed_attributes_url: Url = state.public_url.join(&disclosed_attributes_url.path()[1..]); // `.path()` always starts with a `/`

    let result = DisclosureTemplate {
        urls: Some(EngagementUrls {
            status_url: status_url.to_string(),
            disclosed_attributes_url: mrp_disclosed_attributes_url.to_string(),
        }),
        selected: Some(SelectForm {
            usecase: selected.usecase,
            session_type: selected.session_type,
        }),
        usecases: state.usecases.keys().cloned().collect(),
    };

    Ok(result)
}

#[derive(Deserialize)]
struct DisclosedAttributesParams {
    // Use the same query parameter as is present in the return_url here, as this is easier on the frontend.
    nonce: Option<String>,
}

// for now this just passes the disclosed attributes on as they are received
async fn disclosed_attributes(
    State(state): State<Arc<ApplicationState>>,
    Path(session_token): Path<SessionToken>,
    Query(params): Query<DisclosedAttributesParams>,
) -> Result<Json<DisclosedAttributes>> {
    let attributes = state.client.disclosed_attributes(session_token, params.nonce).await?;
    Ok(Json(attributes))
}
