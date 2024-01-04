use std::{collections::HashMap, result::Result as StdResult, sync::Arc};

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Form, Json, Router,
};
use axum_extra::response::{Css, JavaScript, Wasm};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing::warn;
use url::Url;

use crate::{askama_axum, client::WalletServerClient, settings::Settings};
use nl_wallet_mdoc::{
    server_state::SessionToken,
    verifier::{ItemsRequests, SessionType, StatusResponse},
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
    public_url: Url,
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
        .route("/sessions/:session_id/status", get(status))
        .route("/marx.min.css", get(marxcss))
        .route("/qrcodegen.min.js", get(qrcodegenjs))
        .route("/qrcodegen.wasm", get(qrcodegenwasm))
        .layer(TraceLayer::new_for_http())
        .with_state(application_state);

    Ok(app)
}

#[derive(Deserialize, Serialize)]
struct SelectForm {
    session_type: SessionType,
    usecase: String,
}

#[derive(Template)]
#[template(path = "disclosure.html")]
struct DisclosureTemplate {
    engagement: Option<(String, String)>,
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

#[derive(Serialize)]
struct EngageUrlparams {
    session_type: SessionType,
    return_url: Url,
}

async fn engage(State(state): State<Arc<ApplicationState>>, Form(selected): Form<SelectForm>) -> Result<Response> {
    let mut return_url_template = None;
    // TODO make a third selection option (CrossDevice + return URL)
    if selected.session_type == SessionType::SameDevice {
        let status_url = format!("{}{}", state.public_url, "/sessions/{session_id}/status");
        return_url_template = Some(
            format!("{}#{}", state.public_url, status_url)
                .parse()
                .expect("should always be a valid ReturnUrlTemplate"),
        );
    }

    let (session_url, engagement_url) = state
        .client
        .start(
            selected.usecase.clone(),
            state
                .usecases
                .get(&selected.usecase)
                .ok_or(anyhow::Error::msg("usecase not found"))?
                .clone(),
            selected.session_type,
            return_url_template,
        )
        .await?;

    let mut session_url = session_url.path().to_owned();
    session_url.remove(0); // remove initial '/'

    Ok(askama_axum::into_response(&DisclosureTemplate {
        engagement: Some((engagement_url.to_string(), session_url.to_string())),
        selected: Some(SelectForm {
            usecase: selected.usecase,
            session_type: selected.session_type,
        }),
        usecases: state.usecases.keys().cloned().collect(),
    }))
}

// for now this just passes the status incl. the attributes on as it is received
async fn status(
    State(state): State<Arc<ApplicationState>>,
    Path(session_id): Path<SessionToken>,
) -> Result<Json<StatusResponse>> {
    let status = state.client.status(session_id).await?;

    Ok(Json(status))
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
