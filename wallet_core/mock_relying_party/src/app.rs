use std::{collections::HashMap, env, path::PathBuf, result::Result as StdResult, sync::Arc};

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Form, Json, Router,
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::warn;
use url::Url;

use nl_wallet_mdoc::{
    server_state::SessionToken,
    verifier::{DisclosedAttributes, ItemsRequests, SessionType},
};
use wallet_common::config::wallet_config::BaseUrl;

use crate::{askama_axum, client::WalletServerClient, settings::Settings};

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

pub async fn create_router(settings: Settings) -> anyhow::Result<Router> {
    let application_state = Arc::new(ApplicationState {
        client: WalletServerClient::new(settings.wallet_server_url.clone()),
        public_url: settings.public_url,
        usecases: settings.usecases,
    });

    let root_dir = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

    let app = Router::new()
        .route("/", get(index))
        .route("/", post(engage))
        .route(
            "/disclosure/sessions/:session_token/disclosed_attributes",
            get(disclosed_attributes),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(application_state)
        .fallback_service(
            ServeDir::new(root_dir.join("assets")).not_found_service({ StatusCode::NOT_FOUND }.into_service()),
        );

    Ok(app)
}

#[derive(Deserialize, Serialize)]
struct SelectForm {
    session_type: MrpSessionType,
    usecase: String,
}

#[derive(Deserialize, Serialize, PartialEq, strum::Display, strum::EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
enum MrpSessionType {
    SameDevice,
    SameDeviceNoReturn,
    CrossDevice,
    CrossDeviceWithReturn,
}

#[derive(Template)]
#[template(path = "disclosure.html")]
struct DisclosureTemplate {
    urls: Option<(String, String)>,
    selected: Option<SelectForm>,
    usecases: Vec<String>,
}

async fn index(State(state): State<Arc<ApplicationState>>) -> Result<Response> {
    Ok(askama_axum::into_response(&DisclosureTemplate {
        urls: None,
        selected: None,
        usecases: state.usecases.keys().cloned().collect(),
    }))
}

async fn engage(State(state): State<Arc<ApplicationState>>, Form(selected): Form<SelectForm>) -> Result<Response> {
    // return URL is just http://public.url/#{session_token}
    let return_url_template = match selected.session_type {
        MrpSessionType::CrossDeviceWithReturn | MrpSessionType::SameDevice => Some(
            format!("{}#{{session_token}}", state.public_url)
                .parse()
                .expect("should always be a valid ReturnUrlTemplate"),
        ),
        _ => None,
    };

    let (status_url, disclosed_attributes_url) = state
        .client
        .start(
            selected.usecase.clone(),
            state
                .usecases
                .get(&selected.usecase)
                .ok_or(anyhow::Error::msg("usecase not found"))?
                .clone(),
            match selected.session_type {
                MrpSessionType::SameDevice | MrpSessionType::SameDeviceNoReturn => SessionType::SameDevice,
                _ => SessionType::CrossDevice,
            },
            return_url_template,
        )
        .await?;

    let mrp_disclosed_attributes_url: Url = state.public_url.join(&disclosed_attributes_url.path()[1..]); // `.path()` always starts with a `/`

    Ok(askama_axum::into_response(&DisclosureTemplate {
        urls: Some((status_url.to_string(), mrp_disclosed_attributes_url.to_string())),
        selected: Some(SelectForm {
            usecase: selected.usecase,
            session_type: selected.session_type,
        }),
        usecases: state.usecases.keys().cloned().collect(),
    }))
}

#[derive(Deserialize)]
struct DisclosedAttributesParams {
    transcript_hash: Option<String>,
}

// for now this just passes the disclosed attributes on as they are received
async fn disclosed_attributes(
    State(state): State<Arc<ApplicationState>>,
    Path(session_token): Path<SessionToken>,
    Query(params): Query<DisclosedAttributesParams>,
) -> Result<Json<DisclosedAttributes>> {
    let attributes = state
        .client
        .disclosed_attributes(session_token, params.transcript_hash)
        .await?;
    Ok(Json(attributes))
}
