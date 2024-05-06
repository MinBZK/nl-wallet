use std::{collections::HashMap, result::Result as StdResult, sync::Arc};

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Form, Json, Router,
};
use axum_extra::response::{Css, JavaScript, Wasm};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use tower_http::trace::TraceLayer;
use tracing::warn;
use url::Url;
use wallet_common::config::wallet_config::BaseUrl;

use crate::{askama_axum, client::WalletServerClient, settings::Settings};
use nl_wallet_mdoc::{
    server_state::SessionToken,
    verifier::{DisclosedAttributes, ItemsRequests, SessionType},
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

pub async fn create_router(settings: Settings) -> anyhow::Result<Router> {
    let application_state = Arc::new(ApplicationState {
        client: WalletServerClient::new(settings.wallet_server_url.clone()),
        public_url: settings.public_url,
        usecases: settings.usecases,
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/", post(engage))
        .route(
            "/disclosure/sessions/:session_id/disclosed_attributes",
            get(disclosed_attributes),
        )
        .route("/marx.min.css", get(marxcss))
        .route("/qrcodegen.min.js", get(qrcodegenjs))
        .route("/qrcodegen.wasm", get(qrcodegenwasm))
        .layer(TraceLayer::new_for_http())
        .with_state(application_state);

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
    engagement: Option<(String, String, String)>,
    selected: Option<SelectForm>,
    usecases: Vec<String>,
}

async fn index(State(state): State<Arc<ApplicationState>>) -> Result<Response> {
    Ok(askama_axum::into_response(&DisclosureTemplate {
        engagement: None,
        selected: None,
        usecases: state.usecases.keys().cloned().collect(),
    }))
}

async fn engage(State(state): State<Arc<ApplicationState>>, Form(selected): Form<SelectForm>) -> Result<Response> {
    // return URL is just http://public.url/#{session_id}
    let return_url_template = match selected.session_type {
        MrpSessionType::CrossDeviceWithReturn | MrpSessionType::SameDevice => Some(
            format!("{}#{{session_id}}", state.public_url)
                .parse()
                .expect("should always be a valid ReturnUrlTemplate"),
        ),
        _ => None,
    };

    let (session_url, engagement_url, disclosed_attributes_url) = state
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
        engagement: Some((
            engagement_url.to_string(),
            session_url.to_string(),
            mrp_disclosed_attributes_url.to_string(),
        )),
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
    Path(session_id): Path<SessionToken>,
    Query(params): Query<DisclosedAttributesParams>,
) -> Result<Json<DisclosedAttributes>> {
    let attributes = state
        .client
        .disclosed_attributes(session_id, params.transcript_hash)
        .await?;
    Ok(Json(attributes))
}

// static files to not depend on external resources
async fn qrcodegenjs() -> Result<JavaScript<&'static str>> {
    Ok(JavaScript(include_str!("../templates/qrcodegen.min.js")))
}

async fn qrcodegenwasm() -> Result<Wasm<&'static [u8; 28041]>> {
    Ok(Wasm(include_bytes!("../templates/qrcodegen.wasm")))
}

async fn marxcss() -> Result<Css<&'static str>> {
    Ok(Css(include_str!("../templates/marx.min.css")))
}
