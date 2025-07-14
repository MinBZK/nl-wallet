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

use attestation_data::disclosure::DisclosedAttestations;
use attestation_types::request::NormalizedCredentialRequests;
use crypto::keys::EcdsaKeySend;
use http_utils::error::HttpJsonError;
use http_utils::urls;
use http_utils::urls::BaseUrl;
use http_utils::urls::CorsOrigin;
use openid4vc::disclosure_session::APPLICATION_OAUTH_AUTHZ_REQ_JWT;
use openid4vc::openid4vp::VpResponse;
use openid4vc::openid4vp::WalletRequest;
use openid4vc::return_url::ReturnUrlTemplate;
use openid4vc::server_state::SessionStore;
use openid4vc::server_state::SessionToken;
use openid4vc::verifier::DisclosureData;
use openid4vc::verifier::DisclosureResultHandler;
use openid4vc::verifier::SessionType;
use openid4vc::verifier::StatusResponse;
use openid4vc::verifier::UseCase;
use openid4vc::verifier::UseCases;
use openid4vc::verifier::Verifier;
use openid4vc::verifier::WalletAuthResponse;
use openid4vc::DisclosureErrorResponse;
use openid4vc::GetRequestErrorCode;
use openid4vc::PostAuthResponseErrorCode;
use openid4vc::VerificationErrorCode;
use utils::generator::TimeGenerator;

struct ApplicationState<S, US> {
    verifier: Verifier<S, US>,
    public_url: BaseUrl,
    universal_link_base_url: BaseUrl,
}

pub struct VerifierFactory<US> {
    public_url: BaseUrl,
    universal_link_base_url: BaseUrl,
    use_cases: US,
    issuer_trust_anchors: Vec<TrustAnchor<'static>>,
    accepted_wallet_client_ids: Vec<String>,
}

struct WalletRouterAndState<S, US> {
    wallet_router: Router<Arc<ApplicationState<S, US>>>,
    application_state: Arc<ApplicationState<S, US>>,
}

impl<K, US, UC> VerifierFactory<US>
where
    US: UseCases<UseCase = UC, Key = K> + Send + Sync + 'static,
    UC: UseCase<Key = K> + Send + Sync + 'static,
    K: EcdsaKeySend + Sync + 'static,
{
    pub fn new(
        public_url: BaseUrl,
        universal_link_base_url: BaseUrl,
        use_cases: US,
        issuer_trust_anchors: Vec<TrustAnchor<'static>>,
        accepted_wallet_client_ids: Vec<String>,
    ) -> Self {
        Self {
            public_url,
            universal_link_base_url,
            use_cases,
            issuer_trust_anchors,
            accepted_wallet_client_ids,
        }
    }

    fn wallet_router_and_state<S>(
        self,
        sessions: Arc<S>,
        result_handler: Option<Box<dyn DisclosureResultHandler + Send + Sync>>,
    ) -> WalletRouterAndState<S, US>
    where
        S: SessionStore<DisclosureData> + Send + Sync + 'static,
    {
        let application_state = Arc::new(ApplicationState {
            verifier: Verifier::new(
                self.use_cases,
                sessions,
                self.issuer_trust_anchors,
                result_handler,
                self.accepted_wallet_client_ids.clone(),
            ),
            public_url: self.public_url,
            universal_link_base_url: self.universal_link_base_url,
        });

        // RFC 9101 defines just `GET` for the `request_uri` endpoint, but OpenID4VP extends that with `POST`.
        // Note that since `retrieve_request()` uses the `Form` extractor, it requires the
        // `Content-Type: application/x-www-form-urlencoded` header to be set on POST requests (but not GET requests).
        let wallet_router = Router::new()
            .route("/{identifier}/request_uri", get(retrieve_request::<S, US, UC, K>))
            .route("/{identifier}/request_uri", post(retrieve_request::<S, US, UC, K>))
            .route("/{session_token}/response_uri", post(post_response::<S, US, UC, K>));

        WalletRouterAndState {
            wallet_router,
            application_state,
        }
    }

    pub fn create_wallet_router<S>(
        self,
        sessions: Arc<S>,
        result_handler: Option<Box<dyn DisclosureResultHandler + Send + Sync>>,
    ) -> Router
    where
        S: SessionStore<DisclosureData> + Send + Sync + 'static,
    {
        let WalletRouterAndState {
            wallet_router,
            application_state,
        } = self.wallet_router_and_state(sessions, result_handler);

        wallet_router.with_state(application_state)
    }

    pub fn create_routers<S>(
        self,
        allow_origins: Option<CorsOrigin>,
        sessions: Arc<S>,
        result_handler: Option<Box<dyn DisclosureResultHandler + Send + Sync>>,
    ) -> (Router, Router)
    where
        S: SessionStore<DisclosureData> + Send + Sync + 'static,
    {
        let WalletRouterAndState {
            wallet_router,
            application_state,
        } = self.wallet_router_and_state(sessions, result_handler);

        let mut wallet_web = Router::new()
            .route("/{session_token}", get(status::<S, US, UC, K>))
            .route("/{session_token}", delete(cancel::<S, US, UC, K>));

        if let Some(cors_origin) = allow_origins {
            // The CORS headers should be set for these routes, so that any web browser may call them.
            wallet_web = wallet_web.layer(cors_layer(cors_origin));
        }

        let wallet_router = wallet_router
            .merge(wallet_web)
            .with_state(Arc::clone(&application_state));

        let requester_router = Router::new()
            .route("/", post(start::<S, US, UC, K>))
            .route(
                "/{session_token}/disclosed_attributes",
                get(disclosed_attributes::<S, US, UC, K>),
            )
            .with_state(application_state);

        (wallet_router, requester_router)
    }
}

fn cors_layer(allow_origins: CorsOrigin) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(allow_origins)
        .allow_methods([Method::GET, Method::DELETE])
}

async fn retrieve_request<S, US, UC, K>(
    uri: Uri,
    State(state): State<Arc<ApplicationState<S, US>>>,
    Path(identifier): Path<String>,
    Form(wallet_request): Form<WalletRequest>,
) -> Result<(HeaderMap, String), DisclosureErrorResponse<GetRequestErrorCode>>
where
    S: SessionStore<DisclosureData>,
    US: UseCases<Key = K, UseCase = UC>,
    UC: UseCase<Key = K>,
    K: EcdsaKeySend,
{
    info!("process request for Authorization Request JWT");

    let response = state
        .verifier
        .process_get_request(&identifier, &state.public_url, uri.query(), wallet_request.wallet_nonce)
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

async fn post_response<S, US, UC, K>(
    State(state): State<Arc<ApplicationState<S, US>>>,
    Path(session_token): Path<SessionToken>,
    Form(wallet_response): Form<WalletAuthResponse>,
) -> Result<Json<VpResponse>, DisclosureErrorResponse<PostAuthResponseErrorCode>>
where
    S: SessionStore<DisclosureData>,
    US: UseCases<Key = K, UseCase = UC>,
    UC: UseCase<Key = K>,
    K: EcdsaKeySend,
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

async fn status<S, US, UC, K>(
    State(state): State<Arc<ApplicationState<S, US>>>,
    Path(session_token): Path<SessionToken>,
    Query(query): Query<StatusParams>,
) -> Result<Json<StatusResponse>, HttpJsonError<VerificationErrorCode>>
where
    S: SessionStore<DisclosureData>,
    US: UseCases<Key = K, UseCase = UC>,
    UC: UseCase<Key = K>,
    K: EcdsaKeySend,
{
    let response = state
        .verifier
        .status_response(
            &session_token,
            query.session_type,
            &urls::disclosure_base_uri(&state.universal_link_base_url),
            state.public_url.join_base_url(&format!("{session_token}/request_uri")),
            &TimeGenerator,
        )
        .await
        .inspect_err(|error| warn!("querying session status failed: {error}"))?;

    Ok(Json(response))
}

async fn cancel<S, US, UC, K>(
    State(state): State<Arc<ApplicationState<S, US>>>,
    Path(session_token): Path<SessionToken>,
) -> Result<StatusCode, HttpJsonError<VerificationErrorCode>>
where
    S: SessionStore<DisclosureData>,
    US: UseCases<Key = K, UseCase = UC>,
    UC: UseCase<Key = K>,
    K: EcdsaKeySend,
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
    // TODO: replace with dcql::Query (PVW-4530)
    pub credential_requests: Option<NormalizedCredentialRequests>,
    pub return_url_template: Option<ReturnUrlTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartDisclosureResponse {
    pub session_token: SessionToken,
}

async fn start<S, US, UC, K>(
    State(state): State<Arc<ApplicationState<S, US>>>,
    Json(start_request): Json<StartDisclosureRequest>,
) -> Result<Json<StartDisclosureResponse>, HttpJsonError<VerificationErrorCode>>
where
    S: SessionStore<DisclosureData>,
    US: UseCases<Key = K, UseCase = UC>,
    UC: UseCase<Key = K>,
    K: EcdsaKeySend,
{
    let session_token = state
        .verifier
        .new_session(
            start_request.usecase,
            start_request.credential_requests,
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

async fn disclosed_attributes<S, US, UC, K>(
    State(state): State<Arc<ApplicationState<S, US>>>,
    Path(session_token): Path<SessionToken>,
    Query(params): Query<DisclosedAttributesParams>,
) -> Result<Json<DisclosedAttestations>, HttpJsonError<VerificationErrorCode>>
where
    S: SessionStore<DisclosureData>,
    US: UseCases<Key = K, UseCase = UC>,
    UC: UseCase<Key = K>,
    K: EcdsaKeySend,
{
    let disclosed_attributes = state
        .verifier
        .disclosed_attributes(&session_token, params.nonce)
        .await
        .inspect_err(|error| warn!("fetching disclosed attributes failed: {error}"))?;

    Ok(Json(disclosed_attributes))
}
