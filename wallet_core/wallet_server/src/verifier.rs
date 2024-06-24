use std::sync::Arc;

use axum::{
    extract::{OriginalUri, Path, Query, State},
    routing::{get, on, post, MethodFilter},
    Form, Json, Router,
};
use http::{header, HeaderMap, HeaderValue, Method};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

use nl_wallet_mdoc::{
    server_state::{SessionStore, SessionToken},
    verifier::{DisclosedAttributes, ItemsRequests, ReturnUrlTemplate, SessionType},
};
use openid4vc::{
    disclosure_session::APPLICATION_OAUTH_AUTHZ_REQ_JWT,
    openid4vp::{VpResponse, WalletRequest},
    verifier::{DisclosureData, StatusResponse, Verifier, WalletAuthResponse},
    GetRequestErrorCode, PostAuthResponseErrorCode, VerificationErrorCode,
};
use wallet_common::{config::wallet_config::BaseUrl, generator::TimeGenerator};

use crate::{
    errors::ErrorResponse,
    settings::{self, Urls},
};

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

    // RFC 9101 defines just `GET` for the `request_uri` endpoint, but OpenID4VP extends that with `POST`.
    let wallet_router = Router::new()
        .route(
            "/:session_token/request_uri",
            on(MethodFilter::GET | MethodFilter::POST, retrieve_request::<S>),
        )
        .route("/:session_token/response_uri", post(post_response::<S>))
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

async fn retrieve_request<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_token): Path<SessionToken>,
    OriginalUri(uri): OriginalUri,
    Form(wallet_request): Form<WalletRequest>,
) -> Result<(HeaderMap, String), ErrorResponse<GetRequestErrorCode>>
where
    S: SessionStore<DisclosureData>,
{
    info!("process received message");

    let response = state
        .verifier
        .process_get_request(
            &session_token,
            state
                .public_url
                .join_base_url(&format!("disclosure/{session_token}/response_uri")),
            uri.query(),
            wallet_request.wallet_nonce,
        )
        .await
        .map_err(|err| {
            warn!("processing message failed, returning error");
            ErrorResponse::with_uri(err)
        })?;

    info!("message processed successfully, returning response");

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
) -> Result<Json<VpResponse>, ErrorResponse<PostAuthResponseErrorCode>>
where
    S: SessionStore<DisclosureData>,
{
    info!("process received message");

    let response = state
        .verifier
        .process_authorization_response(&session_token, wallet_response, &TimeGenerator)
        .await
        .map_err(|err| {
            warn!("processing message failed, returning error");
            ErrorResponse::with_uri(err)
        })?;

    info!("message processed successfully, returning response");

    Ok(Json(response))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusParams {
    pub session_type: SessionType,
}

async fn status<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_token): Path<SessionToken>,
    Query(params): Query<StatusParams>,
) -> Result<Json<StatusResponse>, ErrorResponse<VerificationErrorCode>>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let response = state
        .verifier
        .status_response(
            &session_token,
            params.session_type,
            &state.universal_link_base_url.join_base_url("disclosure"),
            state
                .public_url
                .join_base_url(&format!("disclosure/{session_token}/request_uri")),
            &TimeGenerator,
        )
        .await
        .map_err(ErrorResponse::new)?;

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
) -> Result<Json<StartDisclosureResponse>, ErrorResponse<VerificationErrorCode>>
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
        .map_err(ErrorResponse::new)?;

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
) -> Result<Json<DisclosedAttributes>, ErrorResponse<VerificationErrorCode>>
where
    S: SessionStore<DisclosureData>,
{
    let disclosed_attributes = state
        .verifier
        .disclosed_attributes(&session_token, params.nonce)
        .await
        .map_err(ErrorResponse::new)?;
    Ok(Json(disclosed_attributes))
}
