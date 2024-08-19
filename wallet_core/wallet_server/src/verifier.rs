use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Form, Json, Router,
};
use http::{header, HeaderMap, HeaderValue, Method, StatusCode, Uri};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

use nl_wallet_mdoc::verifier::{DisclosedAttributes, ItemsRequests};
use openid4vc::{
    disclosure_session::APPLICATION_OAUTH_AUTHZ_REQ_JWT,
    openid4vp::{VpResponse, WalletRequest},
    return_url::ReturnUrlTemplate,
    server_state::{SessionStore, SessionToken},
    verifier::{DisclosureData, SessionType, StatusResponse, Verifier, WalletAuthResponse},
    DisclosureErrorResponse, GetRequestErrorCode, PostAuthResponseErrorCode, VerificationErrorCode,
};
use wallet_common::{config::wallet_config::BaseUrl, generator::TimeGenerator, http_error::HttpJsonError};

use crate::settings::{self, Urls};

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

    let wallet_web = Router::new()
        .route("/:session_token", get(status::<S>))
        .route("/:session_token", delete(cancel::<S>))
        // The CORS headers should be set for these routes, so that any web browser may call them.
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::DELETE])
                .allow_origin(Any),
        );

    // RFC 9101 defines just `GET` for the `request_uri` endpoint, but OpenID4VP extends that with `POST`.
    // Note that since `retrieve_request()` uses the `Form` extractor, it requires the
    // `Content-Type: application/x-www-form-urlencoded` header to be set on POST requests (but not GET requests).
    let wallet_router = Router::new()
        .route("/:session_token/request_uri", get(retrieve_request::<S>))
        .route("/:session_token/request_uri", post(retrieve_request::<S>))
        .route("/:session_token/response_uri", post(post_response::<S>))
        .merge(wallet_web)
        .with_state(Arc::clone(&application_state));

    let requester_router = Router::new()
        .route("/", post(start::<S>))
        .route("/:session_token/disclosed_attributes", get(disclosed_attributes::<S>))
        .with_state(application_state);

    Ok((
        Router::new().nest("/sessions", wallet_router),
        Router::new().nest("/sessions", requester_router),
    ))
}

async fn retrieve_request<S>(
    uri: Uri,
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_token): Path<SessionToken>,
    Form(wallet_request): Form<WalletRequest>,
) -> Result<(HeaderMap, String), DisclosureErrorResponse<GetRequestErrorCode>>
where
    S: SessionStore<DisclosureData>,
{
    info!("process request for Authorization Request JWT");

    let response = state
        .verifier
        .process_get_request(
            &session_token,
            state
                .public_url
                .join_base_url(&format!("disclosure/sessions/{session_token}/response_uri")),
            uri.query(),
            wallet_request.wallet_nonce,
        )
        .await
        .inspect_err(|error| {
            warn!("processing request for Authorization Request JWT failed, returning error: {error}")
        })?;

    info!("processing request for Authorization Request JWT successful, returning response");

    let headers = HeaderMap::from_iter([(
        header::CONTENT_TYPE,
        HeaderValue::from_static(APPLICATION_OAUTH_AUTHZ_REQ_JWT.as_ref()),
    )]);
    Ok((headers, response.0))
}

async fn post_response<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_token): Path<SessionToken>,
    Form(wallet_response): Form<WalletAuthResponse>,
) -> Result<Json<VpResponse>, DisclosureErrorResponse<PostAuthResponseErrorCode>>
where
    S: SessionStore<DisclosureData>,
{
    info!("process Verifiable Presentation");

    let response = state
        .verifier
        .process_authorization_response(&session_token, wallet_response, &TimeGenerator)
        .await
        .inspect_err(|error| warn!("processing Verifiable Presentation failed, returning error: {error}"))?;

    info!("Verifiable Presentation processed successfully, returning response");

    Ok(Json(response))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StatusParams {
    pub session_type: SessionType,
}

async fn status<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_token): Path<SessionToken>,
    query: Option<Query<StatusParams>>,
) -> Result<Json<StatusResponse>, HttpJsonError<VerificationErrorCode>>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let response = state
        .verifier
        .status_response(
            &session_token,
            query.map(|Query(params)| params.session_type),
            &state.universal_link_base_url.join_base_url("disclosure/sessions"),
            state
                .public_url
                .join_base_url(&format!("disclosure/sessions/{session_token}/request_uri")),
            &TimeGenerator,
        )
        .await
        .inspect_err(|error| warn!("querying session status failed: {error}"))?;

    Ok(Json(response))
}

async fn cancel<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_token): Path<SessionToken>,
) -> Result<StatusCode, HttpJsonError<VerificationErrorCode>>
where
    S: SessionStore<DisclosureData>,
{
    state
        .verifier
        .cancel(&session_token)
        .await
        .inspect_err(|error| warn!("cancelling session failed: {error}"))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartDisclosureRequest {
    pub usecase: String,
    pub items_requests: ItemsRequests,
    pub return_url_template: Option<ReturnUrlTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartDisclosureResponse {
    pub session_token: SessionToken,
}

async fn start<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Json(start_request): Json<StartDisclosureRequest>,
) -> Result<Json<StartDisclosureResponse>, HttpJsonError<VerificationErrorCode>>
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
        .inspect_err(|error| warn!("starting new session failed: {error}"))?;

    Ok(Json(StartDisclosureResponse { session_token }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosedAttributesParams {
    pub nonce: Option<String>,
}

async fn disclosed_attributes<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_token): Path<SessionToken>,
    Query(params): Query<DisclosedAttributesParams>,
) -> Result<Json<DisclosedAttributes>, HttpJsonError<VerificationErrorCode>>
where
    S: SessionStore<DisclosureData>,
{
    let disclosed_attributes = state
        .verifier
        .disclosed_attributes(&session_token, params.nonce)
        .await
        .inspect_err(|error| warn!("fetching disclosed attributes failed: {error}"))?;

    Ok(Json(disclosed_attributes))
}
