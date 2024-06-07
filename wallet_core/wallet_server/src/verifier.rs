use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{OriginalUri, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use http::Method;
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};
use url::Url;

use nl_wallet_mdoc::{
    server_state::{SessionStore, SessionToken},
    verifier::{
        DisclosedAttributes, DisclosureData, ItemsRequests, ReturnUrlTemplate, SessionType, StatusResponse,
        VerificationError, Verifier,
    },
    SessionData,
};
use wallet_common::{config::wallet_config::BaseUrl, generator::TimeGenerator};

use crate::{cbor::Cbor, settings::Settings};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("starting mdoc session failed: {0}")]
    StartSession(#[source] nl_wallet_mdoc::Error),
    #[error("process mdoc message error: {0}")]
    ProcessMdoc(#[source] nl_wallet_mdoc::Error),
    #[error("retrieving status error: {0}")]
    SessionStatus(#[source] nl_wallet_mdoc::Error),
    #[error("retrieving disclosed attributes error: {0}")]
    DisclosedAttributes(#[source] nl_wallet_mdoc::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::StartSession(nl_wallet_mdoc::Error::Verification(_)) => StatusCode::BAD_REQUEST,
            Error::StartSession(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::ProcessMdoc(nl_wallet_mdoc::Error::Verification(verification_error))
            | Error::SessionStatus(nl_wallet_mdoc::Error::Verification(verification_error))
            | Error::DisclosedAttributes(nl_wallet_mdoc::Error::Verification(verification_error)) => {
                match verification_error {
                    VerificationError::UnknownSessionId(_) => StatusCode::NOT_FOUND,
                    VerificationError::SessionStore(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    _ => StatusCode::BAD_REQUEST,
                }
            }
            Error::ProcessMdoc(_) => StatusCode::BAD_REQUEST,
            Error::SessionStatus(_) => StatusCode::BAD_REQUEST,
            Error::DisclosedAttributes(_) => StatusCode::BAD_REQUEST,
        }
        .into_response()
    }
}

struct ApplicationState<S> {
    verifier: Verifier<S>,
    internal_url: BaseUrl,
    public_url: BaseUrl,
    universal_link_base_url: BaseUrl,
}

fn create_application_state<S>(settings: &Settings, sessions: S) -> anyhow::Result<ApplicationState<S>>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let application_state = ApplicationState {
        verifier: Verifier::new(
            (&settings.verifier.usecases).try_into()?,
            sessions,
            settings
                .verifier
                .trust_anchors
                .iter()
                .map(|ta| ta.owned_trust_anchor.to_owned())
                .collect::<Vec<_>>(),
            (&settings.verifier.ephemeral_id_secret).into(),
        ),
        internal_url: settings.internal_url.clone(),
        public_url: settings.public_url.clone(),
        universal_link_base_url: settings.universal_link_base_url.clone(),
    };
    Ok(application_state)
}

pub fn create_routers<S>(settings: &Settings, sessions: S) -> anyhow::Result<(Router, Router)>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let application_state = Arc::new(create_application_state(settings, sessions)?);

    let wallet_router = Router::new()
        .route("/:session_token", post(session::<S>))
        .route(
            "/:session_token/status",
            get(status::<S>)
                // to be able to request the status from a browser, the cors headers should be set
                // but only on this endpoint
                .layer(CorsLayer::new().allow_methods([Method::GET]).allow_origin(Any)),
        )
        .with_state(application_state.clone());

    let requester_router = Router::new()
        .route("/", post(start::<S>))
        .route("/:session_token/disclosed_attributes", get(disclosed_attributes::<S>))
        .with_state(application_state);

    Ok((wallet_router, requester_router))
}

async fn session<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    OriginalUri(uri): OriginalUri,
    Path(session_token): Path<SessionToken>,
    msg: Bytes,
) -> Result<Cbor<SessionData>, Error>
where
    S: SessionStore<DisclosureData>,
{
    // Since axum does not include the scheme and authority in the original URI, we need
    // to rebuild the full URI by combining it with the configured public base URL.
    let verifier_url = state.public_url.join(&uri.to_string());

    info!("process received message");

    let response = state
        .verifier
        .process_message(&msg, &session_token, verifier_url)
        .await
        .map_err(|e| {
            warn!("processing message failed, returning ProcessMdoc error");
            Error::ProcessMdoc(e)
        })?;

    info!("message processed successfully, returning response");

    Ok(Cbor(response))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusParams {
    pub session_type: SessionType,
}

async fn status<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_token): Path<SessionToken>,
    Query(params): Query<StatusParams>,
) -> Result<Json<StatusResponse>, Error>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let response = state
        .verifier
        .status_response(
            &session_token,
            params.session_type,
            &state.universal_link_base_url.join_base_url("disclosure"),
            &state.public_url.join_base_url("disclosure"),
            &TimeGenerator,
        )
        .await
        .map_err(Error::SessionStatus)?;

    Ok(Json(response))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartDisclosureRequest {
    pub usecase: String,
    pub items_requests: ItemsRequests,
    pub return_url_template: Option<ReturnUrlTemplate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartDisclosureResponse {
    pub status_url: Url,
    pub disclosed_attributes_url: Url,
}

async fn start<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Json(start_request): Json<StartDisclosureRequest>,
) -> Result<Json<StartDisclosureResponse>, Error>
where
    S: SessionStore<DisclosureData>,
{
    let session_token = state
        .verifier
        .new_session(
            start_request.items_requests,
            start_request.usecase,
            start_request.return_url_template,
        )
        .await
        .map_err(Error::StartSession)?;

    let status_url = state.public_url.join(&format!("disclosure/{session_token}/status"));
    let disclosed_attributes_url = state
        .internal_url
        .join(&format!("disclosure/sessions/{session_token}/disclosed_attributes"));

    Ok(Json(StartDisclosureResponse {
        status_url,
        disclosed_attributes_url,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisclosedAttributesParams {
    pub nonce: Option<String>,
}

async fn disclosed_attributes<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_token): Path<SessionToken>,
    Query(params): Query<DisclosedAttributesParams>,
) -> Result<Json<DisclosedAttributes>, Error>
where
    S: SessionStore<DisclosureData>,
{
    let disclosed_attributes = state
        .verifier
        .disclosed_attributes(&session_token, params.nonce)
        .await
        .map_err(Error::DisclosedAttributes)?;
    Ok(Json(disclosed_attributes))
}
