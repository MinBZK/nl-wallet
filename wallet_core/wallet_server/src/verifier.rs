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

use nl_wallet_mdoc::{
    server_state::{SessionStore, SessionToken},
    verifier::{
        DisclosedAttributes, DisclosureData, ItemsRequests, ReturnUrlTemplate, SessionType, StatusResponse,
        VerificationError, Verifier,
    },
    SessionData,
};
use wallet_common::{
    config::wallet_config::BaseUrl,
    generator::TimeGenerator,
    http_error::{HttpJsonError, HttpJsonErrorType},
};

use crate::{
    cbor::Cbor,
    settings::{self, Urls},
};

#[derive(Debug, thiserror::Error)]
#[error("process mdoc message error: {0}")]
pub struct ProcessMdocError(#[from] nl_wallet_mdoc::Error);

impl IntoResponse for ProcessMdocError {
    fn into_response(self) -> Response {
        match self.0 {
            nl_wallet_mdoc::Error::Verification(error) => match error {
                VerificationError::UnknownSessionId(_) => StatusCode::NOT_FOUND,
                VerificationError::SessionStore(_) => StatusCode::INTERNAL_SERVER_ERROR,
                _ => StatusCode::BAD_REQUEST,
            },
            _ => StatusCode::BAD_REQUEST,
        }
        .into_response()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum RequesterErrorType {
    Server,
    SessionParameters,
    UnknownSession,
    Nonce,
    SessionState,
}

#[derive(Debug, thiserror::Error)]
pub enum RequesterError {
    #[error("starting mdoc session failed: {0}")]
    StartSession(#[source] nl_wallet_mdoc::Error),
    #[error("retrieving status error: {0}")]
    SessionStatus(#[source] nl_wallet_mdoc::Error),
    #[error("retrieving disclosed attributes error: {0}")]
    DisclosedAttributes(#[source] nl_wallet_mdoc::Error),
}

impl HttpJsonErrorType for RequesterErrorType {
    fn title(&self) -> String {
        match self {
            Self::Server => "A server error occurred.".to_string(),
            Self::SessionParameters => "Incorrect session parameters provided".to_string(),
            Self::UnknownSession => "Unkown session for provided ID.".to_string(),
            Self::Nonce => "Nonce is missing or incorrect.".to_string(),
            Self::SessionState => "Session is not in the required state.".to_string(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Server => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SessionParameters => StatusCode::BAD_REQUEST,
            Self::UnknownSession => StatusCode::NOT_FOUND,
            Self::Nonce => StatusCode::UNAUTHORIZED,
            Self::SessionState => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<RequesterError> for RequesterErrorType {
    fn from(value: RequesterError) -> Self {
        match value {
            RequesterError::StartSession(nl_wallet_mdoc::Error::Verification(error)) => match error {
                VerificationError::UnknownUseCase(_)
                | VerificationError::NoItemsRequests
                | VerificationError::ReturnUrlConfigurationMismatch => Self::SessionParameters,
                _ => Self::Server,
            },
            RequesterError::SessionStatus(nl_wallet_mdoc::Error::Verification(
                VerificationError::UnknownSessionId(_),
            )) => Self::UnknownSession,
            RequesterError::DisclosedAttributes(nl_wallet_mdoc::Error::Verification(error)) => match error {
                VerificationError::UnknownSessionId(_) => Self::UnknownSession,
                VerificationError::ReturnUrlNonceMissing | VerificationError::ReturnUrlNonceMismatch(_) => Self::Nonce,
                VerificationError::SessionNotDone => Self::SessionState,
                _ => Self::Server,
            },
            _ => Self::Server,
        }
    }
}

impl From<RequesterError> for HttpJsonError<RequesterErrorType> {
    fn from(value: RequesterError) -> Self {
        Self::from_error(value)
    }
}

struct ApplicationState<S> {
    verifier: Verifier<S>,
    public_url: BaseUrl,
    universal_link_base_url: BaseUrl,
}

fn create_application_state<S>(
    urls: Urls,
    verifier: settings::Verifier,
    sessions: S,
) -> anyhow::Result<ApplicationState<S>>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let application_state = ApplicationState {
        verifier: Verifier::new(
            verifier.usecases.try_into()?,
            sessions,
            verifier
                .trust_anchors
                .into_iter()
                .map(|ta| ta.owned_trust_anchor)
                .collect::<Vec<_>>(),
            (&verifier.ephemeral_id_secret).into(),
        ),
        public_url: urls.public_url,
        universal_link_base_url: urls.universal_link_base_url,
    };
    Ok(application_state)
}

pub fn create_routers<S>(urls: Urls, verifier: settings::Verifier, sessions: S) -> anyhow::Result<(Router, Router)>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let application_state = Arc::new(create_application_state(urls, verifier, sessions)?);

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
) -> Result<Cbor<SessionData>, ProcessMdocError>
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
        .inspect_err(|error| {
            warn!("processing message failed: {}", error);
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
) -> Result<Json<StatusResponse>, HttpJsonError<RequesterErrorType>>
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
        .inspect_err(|error| {
            warn!("querying session status failed: {}", error);
        })
        .map_err(RequesterError::SessionStatus)?;

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
    pub session_token: SessionToken,
}

async fn start<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Json(start_request): Json<StartDisclosureRequest>,
) -> Result<Json<StartDisclosureResponse>, HttpJsonError<RequesterErrorType>>
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
        .inspect_err(|error| {
            warn!("starting new session failed: {}", error);
        })
        .map_err(RequesterError::StartSession)?;

    Ok(Json(StartDisclosureResponse { session_token }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisclosedAttributesParams {
    pub nonce: Option<String>,
}

async fn disclosed_attributes<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_token): Path<SessionToken>,
    Query(params): Query<DisclosedAttributesParams>,
) -> Result<Json<DisclosedAttributes>, HttpJsonError<RequesterErrorType>>
where
    S: SessionStore<DisclosureData>,
{
    let disclosed_attributes = state
        .verifier
        .disclosed_attributes(&session_token, params.nonce)
        .await
        .inspect_err(|error| {
            warn!("fetching disclosed attributes failed: {}", error);
        })
        .map_err(RequesterError::DisclosedAttributes)?;

    Ok(Json(disclosed_attributes))
}
