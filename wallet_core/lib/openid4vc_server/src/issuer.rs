use std::str::FromStr;
use std::sync::Arc;

use axum::Form;
use axum::Json;
use axum::Router;
use axum::extract::Query;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::HeaderName;
use axum::http::HeaderValue;
use axum::http::StatusCode;
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
use crypto::keys::EcdsaKeySend;
use openid4vc::AuthorizeErrorCode;
use openid4vc::CredentialErrorCode;
use openid4vc::CredentialPreviewErrorCode;
use openid4vc::ErrorResponse;
use openid4vc::ParErrorCode;
use openid4vc::TokenErrorCode;
use openid4vc::authorization::PushedAuthorizationRequest;
use openid4vc::authorization::PushedAuthorizationResponse;
use openid4vc::authorization::VciAuthorizationRequest;
use openid4vc::authorization_code_flow::AuthorizationCodeFlow;
use openid4vc::authorizing_issuer::AuthorizingIssuer;
use openid4vc::credential::CredentialRequest;
use openid4vc::credential::CredentialRequests;
use openid4vc::credential::CredentialResponse;
use openid4vc::credential::CredentialResponses;
use openid4vc::dpop::DPOP_HEADER_NAME;
use openid4vc::dpop::DPOP_NONCE_HEADER_NAME;
use openid4vc::dpop::Dpop;
use openid4vc::issuer::IssuanceData;
use openid4vc::issuer::Issuer;
use openid4vc::metadata::issuer_metadata::IssuerMetadata;
use openid4vc::metadata::oauth_metadata::AuthorizationServerMetadata;
use openid4vc::nonce::response::NonceResponse;
use openid4vc::nonce::store::NonceStore;
use openid4vc::preview::CredentialPreviewRequest;
use openid4vc::preview::CredentialPreviewResponse;
use openid4vc::server_state::SessionStore;
use openid4vc::store::Store;
use openid4vc::token::AccessToken;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenResponse;
use token_status_list::status_list_service::StatusListService;
use tracing::warn;

/// Axum state for the Issuance Phase router (and the pre-authorized-code `/token` route).
struct IssuanceState<K, L, S, N> {
    issuer: Arc<Issuer<K, L, S, N>>,
}

// Implement `Clone` manually, because `#[derive(Clone)]` unnecessarily adds `Clone` bounds on its type parameters,
// which we don't want.
impl<K, L, S, N> Clone for IssuanceState<K, L, S, N> {
    fn clone(&self) -> Self {
        Self {
            issuer: self.issuer.clone(),
        }
    }
}

/// Axum state for the Authorization Phase router.
struct AuthorizationState<K, L, S, N, PAS, AF> {
    authorizing_issuer: Arc<AuthorizingIssuer<K, L, S, N, PAS, AF>>,
}

// Implement `Clone` manually, because `#[derive(Clone)]` unnecessarily adds `Clone` bounds on its type parameters,
// which we don't want.
impl<K, L, S, N, PAS, AF> Clone for AuthorizationState<K, L, S, N, PAS, AF> {
    fn clone(&self) -> Self {
        Self {
            authorizing_issuer: self.authorizing_issuer.clone(),
        }
    }
}

/// Issuance Phase endpoints (credential / nonce / metadata). Does **not** include the token endpoint;
/// that route is supplied per deployment by either [`create_authorization_router`] (the AF provisions
/// issuables for the auth-code flow) or [`create_pre_authorized_token_router`] (pre-authorized-code:
/// no provisioning, session pre-populated externally).
pub fn create_issuance_router<K, L, S, N>(issuer: Arc<Issuer<K, L, S, N>>) -> Router
where
    K: EcdsaKeySend + Sync + 'static,
    L: StatusListService + Send + Sync + 'static,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
    N: NonceStore + Send + Sync + 'static,
{
    Router::new()
        .route("/.well-known/openid-credential-issuer", get(metadata))
        .route("/.well-known/oauth-authorization-server", get(oauth_metadata))
        .route("/issuance/credential_preview", post(credential_preview))
        .route("/issuance/nonce", post(nonce))
        .route("/issuance/credential", post(credential))
        .route("/issuance/credential", delete(reject_credential))
        .route("/issuance/batch_credential", post(batch_credential))
        .route("/issuance/batch_credential", delete(reject_batch_credential))
        .with_state(IssuanceState { issuer })
}

/// The pre-authorized-code-grant `/token` route: delegates straight to the inner [`Issuer`]
/// without any flow involvement. Used by deployments that pre-populate the session externally
/// (e.g. via `IssuanceResultHandler`).
pub fn create_pre_authorized_token_router<K, L, S, N>(issuer: Arc<Issuer<K, L, S, N>>) -> Router
where
    K: EcdsaKeySend + Sync + 'static,
    L: Send + Sync + 'static,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
    N: Send + Sync + 'static,
{
    Router::new()
        .route("/issuance/token", post(token_pre_authorized))
        .with_state(IssuanceState { issuer })
}

/// Authorization Phase endpoints (par / authorize / token (delegated to AF)).
pub fn create_authorization_router<K, L, S, N, PAS, AF>(
    authorizing_issuer: Arc<AuthorizingIssuer<K, L, S, N, PAS, AF>>,
) -> Router
where
    K: EcdsaKeySend + Sync + 'static,
    L: Send + Sync + 'static,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
    N: Send + Sync + 'static,
    PAS: Store<String, VciAuthorizationRequest> + Send + Sync + 'static,
    AF: AuthorizationCodeFlow + Send + Sync + 'static,
{
    Router::new()
        .route("/issuance/par", post(pushed_authorization_request))
        .route("/issuance/authorize", get(authorize))
        .route("/issuance/token", post(token))
        .with_state(AuthorizationState { authorizing_issuer })
}

async fn metadata<K, L, S, N>(State(state): State<IssuanceState<K, L, S, N>>) -> Json<IssuerMetadata> {
    Json(state.issuer.metadata().clone())
}

async fn oauth_metadata<K, L, S, N>(
    State(state): State<IssuanceState<K, L, S, N>>,
) -> Json<AuthorizationServerMetadata> {
    Json(state.issuer.oauth_metadata())
}

async fn credential_preview<K, L, S, N>(
    State(state): State<IssuanceState<K, L, S, N>>,
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

async fn nonce<K, L, S, N>(
    State(state): State<IssuanceState<K, L, S, N>>,
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

async fn credential<K, L, S, N>(
    State(state): State<IssuanceState<K, L, S, N>>,
    TypedHeader(Authorization(authorization_header)): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Json(credential_request): Json<CredentialRequest>,
) -> Result<Json<CredentialResponse>, ErrorResponse<CredentialErrorCode>>
where
    K: EcdsaKeySend,
    L: StatusListService,
    S: SessionStore<IssuanceData>,
    N: NonceStore,
{
    let access_token = authorization_header.into();
    let response = state
        .issuer
        .process_credential(access_token, dpop, credential_request)
        .await
        .inspect_err(|error| warn!("processing credential failed: {}", error))?;

    Ok(Json(response))
}

async fn batch_credential<K, L, S, N>(
    State(state): State<IssuanceState<K, L, S, N>>,
    TypedHeader(Authorization(authorization_header)): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Json(credential_requests): Json<CredentialRequests>,
) -> Result<Json<CredentialResponses>, ErrorResponse<CredentialErrorCode>>
where
    K: EcdsaKeySend,
    L: StatusListService,
    S: SessionStore<IssuanceData>,
    N: NonceStore,
{
    let access_token = authorization_header.into();
    let response = state
        .issuer
        .process_batch_credential(access_token, dpop, credential_requests)
        .await
        .inspect_err(|error| warn!("processing batch credential failed: {}", error))?;

    Ok(Json(response))
}

async fn reject_credential<K, L, S, N>(
    State(state): State<IssuanceState<K, L, S, N>>,
    TypedHeader(Authorization(authorization_header)): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
) -> Result<StatusCode, ErrorResponse<CredentialErrorCode>>
where
    S: SessionStore<IssuanceData>,
{
    let access_token = authorization_header.into();
    state
        .issuer
        .process_reject_issuance(access_token, dpop, "credential")
        .await
        .inspect_err(|error| warn!("processing rejection of issuance failed: {}", error))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn reject_batch_credential<K, L, S, N>(
    State(state): State<IssuanceState<K, L, S, N>>,
    TypedHeader(Authorization(authorization_header)): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
) -> Result<StatusCode, ErrorResponse<CredentialErrorCode>>
where
    S: SessionStore<IssuanceData>,
{
    let access_token = authorization_header.into();
    state
        .issuer
        .process_reject_issuance(access_token, dpop, "batch_credential")
        .await
        .inspect_err(|error| warn!("processing rejection of issuance failed: {}", error))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn token_pre_authorized<K, L, S, N>(
    State(state): State<IssuanceState<K, L, S, N>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Form(token_request): Form<TokenRequest>,
) -> Result<(HeaderMap, Json<TokenResponse>), ErrorResponse<TokenErrorCode>>
where
    K: EcdsaKeySend,
    S: SessionStore<IssuanceData>,
{
    let (response, dpop_nonce) = state
        .issuer
        .process_token_request(token_request, dpop)
        .await
        .inspect_err(|error| warn!("processing token request failed: {error}"))?;

    let headers = HeaderMap::from_iter([(
        HeaderName::from_str(DPOP_NONCE_HEADER_NAME).unwrap(),
        HeaderValue::from_str(&dpop_nonce).unwrap(),
    )]);
    Ok((headers, Json(response)))
}

async fn pushed_authorization_request<K, L, S, N, PAS, AF>(
    State(state): State<AuthorizationState<K, L, S, N, PAS, AF>>,
    Form(authorization_request): Form<VciAuthorizationRequest>,
) -> Result<(StatusCode, Json<PushedAuthorizationResponse>), ErrorResponse<ParErrorCode>>
where
    PAS: Store<String, VciAuthorizationRequest>,
{
    let response = state
        .authorizing_issuer
        .process_pushed_authorization_request(authorization_request)
        .await
        .inspect_err(|error| warn!("processing pushed authorization request failed: {error}"))?;

    Ok((StatusCode::CREATED, Json(response)))
}

async fn authorize<K, L, S, N, PAS, AF>(
    State(state): State<AuthorizationState<K, L, S, N, PAS, AF>>,
    Query(PushedAuthorizationRequest { request_uri, client_id }): Query<PushedAuthorizationRequest>,
) -> Result<Response, ErrorResponse<AuthorizeErrorCode>>
where
    PAS: Store<String, VciAuthorizationRequest>,
    AF: AuthorizationCodeFlow,
{
    let redirect_url = state
        .authorizing_issuer
        .process_authorize(&request_uri, &client_id)
        .await
        .inspect_err(|error| warn!("processing authorization request failed: {error}"))?;

    Ok((StatusCode::FOUND, [(header::LOCATION, redirect_url.to_string())]).into_response())
}

async fn token<K, L, S, N, PAS, AF>(
    State(state): State<AuthorizationState<K, L, S, N, PAS, AF>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Form(token_request): Form<TokenRequest>,
) -> Result<(HeaderMap, Json<TokenResponse>), ErrorResponse<TokenErrorCode>>
where
    K: EcdsaKeySend,
    S: SessionStore<IssuanceData>,
    AF: AuthorizationCodeFlow,
{
    let (response, dpop_nonce) = state
        .authorizing_issuer
        .process_token_request(token_request, dpop)
        .await
        .inspect_err(|error| warn!("processing token request failed: {error}"))?;

    let headers = HeaderMap::from_iter([(
        HeaderName::from_str(DPOP_NONCE_HEADER_NAME).unwrap(),
        HeaderValue::from_str(&dpop_nonce).unwrap(),
    )]);
    Ok((headers, Json(response)))
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
