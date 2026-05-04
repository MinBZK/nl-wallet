use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use axum::Form;
use axum::Json;
use axum::Router;
use axum::extract::Query;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::HeaderName;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::http::Uri;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::post;
use axum_extra::TypedHeader;
use axum_extra::headers;
use axum_extra::headers::Authorization;
use axum_extra::headers::CacheControl;
use axum_extra::headers::Header;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::authorization::Credentials;
use chrono::Utc;
use crypto::keys::EcdsaKeySend;
use openid4vc::CredentialErrorCode;
use openid4vc::CredentialPreviewErrorCode;
use openid4vc::ErrorResponse;
use openid4vc::ErrorStatusCode;
use openid4vc::TokenErrorCode;
use openid4vc::authorization::AuthorizationRequest;
use openid4vc::authorization::PushedAuthorizationRequest;
use openid4vc::authorization::PushedAuthorizationResponse;
use openid4vc::credential::CredentialRequest;
use openid4vc::credential::CredentialRequests;
use openid4vc::credential::CredentialResponse;
use openid4vc::credential::CredentialResponses;
use openid4vc::dpop::DPOP_HEADER_NAME;
use openid4vc::dpop::DPOP_NONCE_HEADER_NAME;
use openid4vc::dpop::Dpop;
use openid4vc::issuer::AttributeService;
use openid4vc::issuer::IssuanceData;
use openid4vc::issuer::Issuer;
use openid4vc::metadata::issuer_metadata::IssuerMetadata;
use openid4vc::metadata::oauth_metadata::AuthorizationServerMetadata;
use openid4vc::nonce::response::NonceResponse;
use openid4vc::nonce::store::NonceStore;
use openid4vc::par;
use openid4vc::par::PAR_TTL;
use openid4vc::par::ParStore;
use openid4vc::preview::CredentialPreviewRequest;
use openid4vc::preview::CredentialPreviewResponse;
use openid4vc::server_state::SessionStore;
use openid4vc::token::AccessToken;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenResponse;
use serde::Serialize;
use token_status_list::status_list_service::StatusListServices;
use tracing::warn;
use url::Url;

/// Error returned by [`UpstreamAuthorizationAdapter::adapt`].
#[derive(Debug, thiserror::Error)]
pub enum UpstreamResolveError {
    #[error("upstream metadata discovery failed: {0}")]
    Discovery(Box<dyn std::error::Error + Send + Sync>),

    #[error("upstream metadata has no authorization_endpoint")]
    NoAuthorizationEndpoint,
}

/// Adapts the wallet's authorization request to what the upstream OIDC provider expects.
///
/// The implementer resolves the upstream authorization endpoint (e.g. via OIDC discovery)
/// and rewrites the request.
#[async_trait]
pub trait UpstreamAuthorizationAdapter: Send + Sync {
    async fn adapt(&self, request: AuthorizationRequest) -> Result<(Url, AuthorizationRequest), UpstreamResolveError>;
}

struct ApplicationState<K, A, S, N, L, P> {
    issuer: Arc<Issuer<K, A, S, N, L>>,
    par_store: Arc<P>,
    upstream_authorization_adapter: Option<Arc<dyn UpstreamAuthorizationAdapter>>,
    accepted_wallet_client_ids: Vec<String>,
}

// Implement `Clone` manually, because `#[derive(Clone)]` unnecessarily adds `Clone` bounds on its type parameters,
// which we don't want.
impl<K, A, S, N, L, P> Clone for ApplicationState<K, A, S, N, L, P> {
    fn clone(&self) -> Self {
        Self {
            issuer: self.issuer.clone(),
            par_store: self.par_store.clone(),
            upstream_authorization_adapter: self.upstream_authorization_adapter.clone(),
            accepted_wallet_client_ids: self.accepted_wallet_client_ids.clone(),
        }
    }
}

pub fn create_issuance_router<K, A, S, N, L, P>(
    issuer: Arc<Issuer<K, A, S, N, L>>,
    par_store: Arc<P>,
    upstream_authorization_adapter: Option<Arc<dyn UpstreamAuthorizationAdapter>>,
    accepted_wallet_client_ids: Vec<String>,
) -> Router
where
    K: EcdsaKeySend + Sync + 'static,
    A: AttributeService + Send + Sync + 'static,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
    N: NonceStore + Send + Sync + 'static,
    L: StatusListServices + Send + Sync + 'static,
    P: ParStore + Send + Sync + 'static,
{
    let application_state = ApplicationState {
        issuer,
        par_store,
        upstream_authorization_adapter,
        accepted_wallet_client_ids,
    };

    Router::new()
        .nest(
            "/.well-known",
            Router::new()
                .route("/openid-credential-issuer", get(metadata))
                .route("/oauth-authorization-server", get(oauth_metadata)),
        )
        .nest(
            "/issuance",
            Router::new()
                .route("/par", post(pushed_authorization_request::<K, A, S, N, L, P>))
                .route("/authorize", get(authorize::<K, A, S, N, L, P>))
                .route("/token", post(token))
                .route("/credential_preview", post(credential_preview))
                .route("/nonce", post(nonce))
                .route("/credential", post(credential))
                .route("/credential", delete(reject_issuance))
                .route("/batch_credential", post(batch_credential))
                .route("/batch_credential", delete(reject_issuance)),
        )
        .with_state(application_state)
}

// Although there is no standard here mandating what our error response looks like, we use `ErrorResponse`
// for consistency with the other endpoints.
async fn oauth_metadata<K, A, S, N, L, P>(
    State(state): State<ApplicationState<K, A, S, N, L, P>>,
) -> Json<AuthorizationServerMetadata>
where
    A: AttributeService,
{
    Json(state.issuer.oauth_metadata())
}

async fn metadata<K, A, S, N, L, P>(State(state): State<ApplicationState<K, A, S, N, L, P>>) -> Json<IssuerMetadata> {
    Json(state.issuer.metadata().clone())
}

async fn token<K, A, S, N, L, P>(
    State(state): State<ApplicationState<K, A, S, N, L, P>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Form(token_request): Form<TokenRequest>,
) -> Result<(HeaderMap, Json<TokenResponse>), ErrorResponse<TokenErrorCode>>
where
    K: EcdsaKeySend,
    A: AttributeService,
    S: SessionStore<IssuanceData>,
{
    let (response, dpop_nonce) = state
        .issuer
        .process_token_request(token_request, dpop)
        .await
        .inspect_err(|error| {
            warn!("processing token request failed: {}", error);
        })?;

    let headers = HeaderMap::from_iter([(
        HeaderName::from_str(DPOP_NONCE_HEADER_NAME).unwrap(),
        HeaderValue::from_str(&dpop_nonce).unwrap(),
    )]);
    Ok((headers, Json(response)))
}

async fn credential_preview<K, A, S, N, L, P>(
    State(state): State<ApplicationState<K, A, S, N, L, P>>,
    TypedHeader(Authorization(authorization_header)): TypedHeader<Authorization<Bearer>>,
    Json(preview_request): Json<CredentialPreviewRequest>,
) -> Result<Json<CredentialPreviewResponse>, ErrorResponse<CredentialPreviewErrorCode>>
where
    S: SessionStore<IssuanceData>,
{
    let access_token = AccessToken::from(authorization_header.token().to_string());
    let response = state
        .issuer
        .process_credential_preview(access_token, preview_request)
        .await
        .inspect_err(|error| warn!("processing credential preview failed: {}", error))?;

    Ok(Json(response))
}

async fn nonce<K, A, S, N, L, P>(
    State(state): State<ApplicationState<K, A, S, N, L, P>>,
) -> Result<(TypedHeader<CacheControl>, Json<NonceResponse>), StatusCode>
where
    N: NonceStore,
{
    let c_nonce = state.issuer.generate_proof_nonce().await.map_err(|error| {
        warn!("generating fresh c_nonce failed: {}", error);

        // Any error that occurs while generating the nonce is de facto a problem with the server.
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Including this header is mandated by the OpenID4VCI specification.
    // See: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-7.2-3>
    let header = TypedHeader(CacheControl::new().with_no_store());
    let body = Json(NonceResponse { c_nonce });

    Ok((header, body))
}

async fn credential<K, A, S, N, L, P>(
    State(state): State<ApplicationState<K, A, S, N, L, P>>,
    TypedHeader(Authorization(authorization_header)): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Json(credential_request): Json<CredentialRequest>,
) -> Result<Json<CredentialResponse>, ErrorResponse<CredentialErrorCode>>
where
    K: EcdsaKeySend,
    A: AttributeService,
    S: SessionStore<IssuanceData>,
    N: NonceStore,
    L: StatusListServices,
{
    let access_token = authorization_header.into();
    let response = state
        .issuer
        .process_credential(access_token, dpop, credential_request)
        .await
        .inspect_err(|error| warn!("processing credential failed: {}", error))?;

    Ok(Json(response))
}

async fn batch_credential<K, A, S, N, L, P>(
    State(state): State<ApplicationState<K, A, S, N, L, P>>,
    TypedHeader(Authorization(authorization_header)): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Json(credential_requests): Json<CredentialRequests>,
) -> Result<Json<CredentialResponses>, ErrorResponse<CredentialErrorCode>>
where
    K: EcdsaKeySend,
    A: AttributeService,
    S: SessionStore<IssuanceData>,
    N: NonceStore,
    L: StatusListServices,
{
    let access_token = authorization_header.into();
    let response = state
        .issuer
        .process_batch_credential(access_token, dpop, credential_requests)
        .await
        .inspect_err(|error| warn!("processing batch credential failed: {}", error))?;

    Ok(Json(response))
}

async fn reject_issuance<K, A, S, N, L, P>(
    State(state): State<ApplicationState<K, A, S, N, L, P>>,
    TypedHeader(Authorization(authorization_header)): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    uri: Uri,
) -> Result<StatusCode, ErrorResponse<CredentialErrorCode>>
where
    S: SessionStore<IssuanceData>,
{
    let uri_path = &uri.path()[1..]; // strip off leading slash

    let access_token = authorization_header.into();
    state
        .issuer
        .process_reject_issuance(access_token, dpop, uri_path)
        .await
        .inspect_err(|error| warn!("processing rejection of issuance failed: {}", error))?;

    Ok(StatusCode::NO_CONTENT)
}

static DPOP_HEADER_NAME_LOWERCASE: HeaderName = HeaderName::from_static("dpop");

pub struct DpopHeader(Dpop);

impl Header for DpopHeader {
    fn name() -> &'static HeaderName {
        &DPOP_HEADER_NAME_LOWERCASE
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        // Exactly one value must be provided
        let value = values.next().ok_or(headers::Error::invalid())?;
        if values.next().is_some() {
            return Err(headers::Error::invalid());
        }

        let value = value
            .to_str()
            .map_err(|_| headers::Error::invalid())?
            .parse()
            .map_err(|_| headers::Error::invalid())?;
        Ok(DpopHeader(value))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let DpopHeader(dpop) = self;
        values.extend(HeaderValue::from_bytes(dpop.as_ref().serialization().as_bytes()));
    }
}

#[derive(Clone, Debug)]
pub struct DpopBearer(String);

impl From<DpopBearer> for AccessToken {
    fn from(value: DpopBearer) -> Self {
        value.0.into()
    }
}

impl Credentials for DpopBearer {
    const SCHEME: &'static str = DPOP_HEADER_NAME;

    fn decode(value: &HeaderValue) -> Option<Self> {
        value
            .to_str()
            .ok() // + 1 to account for space after "DPoP"
            .map(|value| Self(value[(DPOP_HEADER_NAME.len() + 1)..].to_string()))
    }

    fn encode(&self) -> HeaderValue {
        HeaderValue::from_str(&(DPOP_HEADER_NAME.to_string() + " " + &self.0)).unwrap()
    }
}

async fn pushed_authorization_request<K, A, S, N, L, P>(
    State(state): State<ApplicationState<K, A, S, N, L, P>>,
    Form(authorization_request): Form<AuthorizationRequest>,
) -> Result<(StatusCode, Json<PushedAuthorizationResponse>), ErrorResponse<ParError>>
where
    K: EcdsaKeySend,
    A: AttributeService,
    S: SessionStore<IssuanceData>,
    N: NonceStore,
    L: StatusListServices,
    P: ParStore,
{
    if !state
        .accepted_wallet_client_ids
        .contains(&authorization_request.client_id)
    {
        return Err(ErrorResponse {
            error: ParError::InvalidClient,
            error_description: Some(format!("unknown client_id: {}", authorization_request.client_id)),
            error_uri: None,
        });
    }

    let request_uri = par::generate_request_uri();
    let expires_at = Utc::now() + PAR_TTL;

    state
        .par_store
        .store(request_uri.clone(), authorization_request, expires_at)
        .await
        .map_err(|error| {
            warn!("storing PAR request failed: {}", error);
            ErrorResponse {
                error: ParError::ServerError,
                error_description: Some(error.to_string()),
                error_uri: None,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(PushedAuthorizationResponse {
            request_uri,
            expires_in: PAR_TTL,
        }),
    ))
}

async fn authorize<K, A, S, N, L, P>(
    State(state): State<ApplicationState<K, A, S, N, L, P>>,
    Query(PushedAuthorizationRequest { request_uri, client_id }): Query<PushedAuthorizationRequest>,
) -> Result<Response, ErrorResponse<AuthorizeError>>
where
    K: EcdsaKeySend,
    A: AttributeService,
    S: SessionStore<IssuanceData>,
    N: NonceStore,
    L: StatusListServices,
    P: ParStore,
{
    if !state.accepted_wallet_client_ids.contains(&client_id) {
        return Err(ErrorResponse {
            error: AuthorizeError::InvalidClient,
            error_description: Some(format!("unknown client_id: {client_id}")),
            error_uri: None,
        });
    }

    let upstream_authorization_adapter = state
        .upstream_authorization_adapter
        .clone()
        .ok_or_else(|| ErrorResponse {
            error: AuthorizeError::ServerError,
            error_description: Some("no upstream authorization adapter configured".to_string()),
            error_uri: None,
        })?;

    let authorization_request = state
        .par_store
        .consume(&request_uri)
        .await
        .map_err(|error| {
            warn!("consuming PAR request failed: {}", error);
            ErrorResponse {
                error: AuthorizeError::ServerError,
                error_description: Some(error.to_string()),
                error_uri: None,
            }
        })?
        .ok_or_else(|| ErrorResponse {
            error: AuthorizeError::InvalidRequest,
            error_description: Some(format!("request_uri not found or expired: {request_uri}")),
            error_uri: None,
        })?;

    let (authorization_endpoint, authorization_request) = upstream_authorization_adapter
        .adapt(authorization_request)
        .await
        .map_err(|error| {
            warn!("adapting authorization request for upstream failed: {}", error);
            ErrorResponse {
                error: AuthorizeError::ServerError,
                error_description: Some(error.to_string()),
                error_uri: None,
            }
        })?;

    let query_string = serde_urlencoded::to_string(&authorization_request).map_err(|error| {
        warn!("encoding authorization request as query string failed: {}", error);
        ErrorResponse {
            error: AuthorizeError::ServerError,
            error_description: Some(error.to_string()),
            error_uri: None,
        }
    })?;

    let mut redirect_url = authorization_endpoint;
    redirect_url.set_query(Some(&query_string));

    Ok((StatusCode::FOUND, [(header::LOCATION, redirect_url.to_string())]).into_response())
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ParError {
    InvalidClient,
    ServerError,
}

impl ErrorStatusCode for ParError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidClient => StatusCode::UNAUTHORIZED,
            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
enum AuthorizeError {
    InvalidClient,
    InvalidRequest,
    ServerError,
}

impl ErrorStatusCode for AuthorizeError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidClient => StatusCode::UNAUTHORIZED,
            Self::InvalidRequest => StatusCode::BAD_REQUEST,
            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
