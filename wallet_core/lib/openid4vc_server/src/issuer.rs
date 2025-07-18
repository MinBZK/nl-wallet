use std::str::FromStr;
use std::sync::Arc;

use axum::Form;
use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::HeaderName;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::http::Uri;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::post;
use axum_extra::TypedHeader;
use axum_extra::headers;
use axum_extra::headers::Authorization;
use axum_extra::headers::Header;
use axum_extra::headers::authorization::Credentials;
use serde::Serialize;
use tracing::warn;

use crypto::keys::EcdsaKeySend;
use openid4vc::CredentialErrorCode;
use openid4vc::ErrorResponse;
use openid4vc::ErrorStatusCode;
use openid4vc::TokenErrorCode;
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
use openid4vc::metadata::IssuerMetadata;
use openid4vc::oidc;
use openid4vc::server_state::SessionStore;
use openid4vc::server_state::WteTracker;
use openid4vc::token::AccessToken;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenResponseWithPreviews;

struct ApplicationState<A, K, S, W> {
    issuer: Arc<Issuer<A, K, S, W>>,
}

// Implement `Clone` manually, because `#[derive(Clone)]` unnecessarily adds `Clone` bounds on A, K, S and W,
// which we don't want.
impl<A, K, S, W> Clone for ApplicationState<A, K, S, W> {
    fn clone(&self) -> Self {
        Self {
            issuer: self.issuer.clone(),
        }
    }
}

pub fn create_issuance_router<A, K, S, W>(issuer: Arc<Issuer<A, K, S, W>>) -> Router
where
    A: AttributeService + Send + Sync + 'static,
    K: EcdsaKeySend + Sync + 'static,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
    W: WteTracker + Send + Sync + 'static,
{
    let application_state = ApplicationState { issuer };

    Router::new()
        .route("/.well-known/openid-credential-issuer", get(metadata))
        .route("/.well-known/oauth-authorization-server", get(oauth_metadata))
        .route("/token", post(token))
        .route("/credential", post(credential))
        .route("/credential", delete(reject_issuance))
        .route("/batch_credential", post(batch_credential))
        .route("/batch_credential", delete(reject_issuance))
        .with_state(application_state)
}

// Although there is no standard here mandating what our error response looks like, we use `ErrorResponse`
// for consistency with the other endpoints.
async fn oauth_metadata<A, K, S, W>(
    State(state): State<ApplicationState<A, K, S, W>>,
) -> Result<Json<oidc::Config>, ErrorResponse<MetadataError>>
where
    A: AttributeService,
    K: EcdsaKeySend,
    S: SessionStore<IssuanceData>,
    W: WteTracker,
{
    let metadata = state.issuer.oauth_metadata().await.map_err(|error| {
        warn!("retrieving OAuth metadata failed: {}", error);

        openid4vc::ErrorResponse {
            error: MetadataError::Metadata,
            error_description: Some(error.to_string()),
            error_uri: None,
        }
    })?;

    Ok(Json(metadata))
}

async fn metadata<A, K, S, W>(State(state): State<ApplicationState<A, K, S, W>>) -> Json<IssuerMetadata> {
    Json(state.issuer.metadata.clone())
}

async fn token<A, K, S, W>(
    State(state): State<ApplicationState<A, K, S, W>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Form(token_request): Form<TokenRequest>,
) -> Result<(HeaderMap, Json<TokenResponseWithPreviews>), ErrorResponse<TokenErrorCode>>
where
    A: AttributeService,
    K: EcdsaKeySend,
    S: SessionStore<IssuanceData>,
    W: WteTracker,
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

async fn credential<A, K, S, W>(
    State(state): State<ApplicationState<A, K, S, W>>,
    TypedHeader(Authorization(authorization_header)): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Json(credential_request): Json<CredentialRequest>,
) -> Result<Json<CredentialResponse>, ErrorResponse<CredentialErrorCode>>
where
    A: AttributeService,
    K: EcdsaKeySend,
    S: SessionStore<IssuanceData>,
    W: WteTracker,
{
    let access_token = authorization_header.into();
    let response = state
        .issuer
        .process_credential(access_token, dpop, credential_request)
        .await
        .inspect_err(|error| warn!("processing credential failed: {}", error))?;

    Ok(Json(response))
}

async fn batch_credential<A, K, S, W>(
    State(state): State<ApplicationState<A, K, S, W>>,
    TypedHeader(Authorization(authorization_header)): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Json(credential_requests): Json<CredentialRequests>,
) -> Result<Json<CredentialResponses>, ErrorResponse<CredentialErrorCode>>
where
    A: AttributeService,
    K: EcdsaKeySend,
    S: SessionStore<IssuanceData>,
    W: WteTracker,
{
    let access_token = authorization_header.into();
    let response = state
        .issuer
        .process_batch_credential(access_token, dpop, credential_requests)
        .await
        .inspect_err(|error| warn!("processing batch credential failed: {}", error))?;

    Ok(Json(response))
}

async fn reject_issuance<A, K, S, W>(
    State(state): State<ApplicationState<A, K, S, W>>,
    TypedHeader(Authorization(authorization_header)): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    uri: Uri,
) -> Result<StatusCode, ErrorResponse<CredentialErrorCode>>
where
    A: AttributeService,
    K: EcdsaKeySend,
    S: SessionStore<IssuanceData>,
    W: WteTracker,
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

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum_extra::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        // Exactly one value must be provided
        let value = values.next().ok_or(headers::Error::invalid())?;
        if values.next().is_some() {
            return Err(headers::Error::invalid());
        }

        let value = value.to_str().map_err(|_| headers::Error::invalid())?.to_string();
        Ok(DpopHeader(value.into()))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let DpopHeader(dpop) = self;
        values.extend(HeaderValue::from_bytes(dpop.as_ref().as_bytes()));
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

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
enum MetadataError {
    Metadata,
}

impl ErrorStatusCode for MetadataError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
