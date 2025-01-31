use std::sync::Arc;

use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::post;
use axum::Form;
use axum::Json;
use axum::Router;
use http::header;
use http::HeaderMap;
use http::HeaderValue;
use http::Method;
use http::StatusCode;
use http::Uri;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing::warn;

use nl_wallet_mdoc::verifier::DisclosedAttributes;
use nl_wallet_mdoc::verifier::ItemsRequests;
use openid4vc::disclosure_session::APPLICATION_OAUTH_AUTHZ_REQ_JWT;
use openid4vc::openid4vp::VpResponse;
use openid4vc::openid4vp::WalletRequest;
use openid4vc::return_url::ReturnUrlTemplate;
use openid4vc::server_state::SessionStore;
use openid4vc::server_state::SessionToken;
use openid4vc::verifier::DisclosureData;
use openid4vc::verifier::SessionType;
use openid4vc::verifier::StatusResponse;
use openid4vc::verifier::Verifier;
use openid4vc::verifier::WalletAuthResponse;
use openid4vc::DisclosureErrorResponse;
use openid4vc::GetRequestErrorCode;
use openid4vc::PostAuthResponseErrorCode;
use openid4vc::VerificationErrorCode;
use wallet_common::generator::TimeGenerator;
use wallet_common::http_error::HttpJsonError;
use wallet_common::urls;
use wallet_common::urls::BaseUrl;
use wallet_common::urls::CorsOrigin;

use crate::settings;
use crate::settings::Urls;

struct ApplicationState<S> {
    verifier: Verifier<S>,
    public_url: BaseUrl,
    universal_link_base_url: BaseUrl,
}

fn create_application_state<S>(
    urls: Urls,
    verifier: settings::Verifier,
    issuer_trust_anchors: Vec<TrustAnchor<'static>>,
    sessions: S,
) -> anyhow::Result<ApplicationState<S>>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let application_state = ApplicationState {
        verifier: Verifier::new(
            verifier.usecases.try_into()?,
            sessions,
            issuer_trust_anchors,
            (&verifier.ephemeral_id_secret).into(),
        ),
        public_url: urls.public_url,
        universal_link_base_url: urls.universal_link_base_url,
    };
    Ok(application_state)
}

fn cors_layer(allow_origins: CorsOrigin) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(allow_origins)
        .allow_methods([Method::GET, Method::DELETE])
}

pub fn create_routers<S>(
    urls: Urls,
    verifier: settings::Verifier,
    issuer_trust_anchors: Vec<TrustAnchor<'static>>,
    sessions: S,
) -> anyhow::Result<(Router, Router)>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let allow_origins = verifier.allow_origins.clone();
    let application_state = Arc::new(create_application_state(
        urls,
        verifier,
        issuer_trust_anchors,
        sessions,
    )?);

    let mut wallet_web = Router::new()
        .route("/{session_token}", get(status::<S>))
        .route("/{session_token}", delete(cancel::<S>));

    if let Some(cors_origin) = allow_origins {
        // The CORS headers should be set for these routes, so that any web browser may call them.
        wallet_web = wallet_web.layer(cors_layer(cors_origin));
    }

    // RFC 9101 defines just `GET` for the `request_uri` endpoint, but OpenID4VP extends that with `POST`.
    // Note that since `retrieve_request()` uses the `Form` extractor, it requires the
    // `Content-Type: application/x-www-form-urlencoded` header to be set on POST requests (but not GET requests).
    let wallet_router = Router::new()
        .route("/{session_token}/request_uri", get(retrieve_request::<S>))
        .route("/{session_token}/request_uri", post(retrieve_request::<S>))
        .route("/{session_token}/response_uri", post(post_response::<S>))
        .merge(wallet_web)
        .with_state(Arc::clone(&application_state));

    let requester_router = Router::new()
        .route("/", post(start::<S>))
        .route("/{session_token}/disclosed_attributes", get(disclosed_attributes::<S>))
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
            warn!("processing request for Authorization Request JWT failed, returning error: {error}");
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
    pub session_type: Option<SessionType>,
}

async fn status<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_token): Path<SessionToken>,
    Query(query): Query<StatusParams>,
) -> Result<Json<StatusResponse>, HttpJsonError<VerificationErrorCode>>
where
    S: SessionStore<DisclosureData> + Send + Sync + 'static,
{
    let response = state
        .verifier
        .status_response(
            &session_token,
            query.session_type,
            &urls::disclosure_base_uri(&state.universal_link_base_url),
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
