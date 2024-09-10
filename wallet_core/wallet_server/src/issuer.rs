use std::{collections::HashMap, str::FromStr, sync::Arc};

use axum::{
    extract::State,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, Uri},
    routing::{delete, get, post},
    Form, Json, Router,
};
use axum_extra::{
    headers::{self, authorization::Credentials, Authorization, Header},
    TypedHeader,
};
use nutype::nutype;
use p256::ecdsa::SigningKey;
use serde::Serialize;

use nl_wallet_mdoc::server_keys::{AttestationSigner, KeyRing};
use openid4vc::{
    credential::{CredentialRequest, CredentialRequests, CredentialResponse, CredentialResponses},
    dpop::{Dpop, DPOP_HEADER_NAME, DPOP_NONCE_HEADER_NAME},
    metadata::IssuerMetadata,
    oidc,
    server_state::SessionStore,
    token::{AccessToken, TokenRequest, TokenResponseWithPreviews},
    CredentialErrorCode, ErrorResponse, ErrorStatusCode, TokenErrorCode,
};
use wallet_common::keys::EcdsaKeySend;

use crate::settings::{self, Urls};

use openid4vc::issuer::{AttributeService, IssuanceData, Issuer};

struct ApplicationState<A, K, S> {
    issuer: Issuer<A, K, S>,
}

#[nutype(derive(From, AsRef))]
pub struct IssuerKeyRing<K>(HashMap<String, AttestationSigner<K>>);

impl<K: EcdsaKeySend> KeyRing for IssuerKeyRing<K> {
    type Key = K;

    fn key_pair(&self, id: &str) -> Option<&AttestationSigner<K>> {
        self.as_ref().get(id)
    }
}

impl TryFrom<HashMap<String, settings::KeyPair>> for IssuerKeyRing<SigningKey> {
    type Error = p256::pkcs8::Error;

    fn try_from(private_keys: HashMap<String, settings::KeyPair>) -> Result<Self, Self::Error> {
        let key_ring = private_keys
            .into_iter()
            .map(|(doctype, key_pair)| {
                let key_pair = (&key_pair).try_into()?;

                Ok((doctype, key_pair))
            })
            .collect::<Result<HashMap<_, _>, Self::Error>>()?
            .into();

        Ok(key_ring)
    }
}

pub fn create_issuance_router<A, K, S>(
    urls: &Urls,
    private_keys: K,
    sessions: S,
    attr_service: A,
    wallet_client_ids: Vec<String>,
) -> anyhow::Result<Router>
where
    A: AttributeService + Send + Sync + 'static,
    K: KeyRing + Send + Sync + 'static,
    <K as KeyRing>::Key: Sync + 'static,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
{
    let application_state = Arc::new(ApplicationState {
        issuer: Issuer::new(
            sessions,
            attr_service,
            private_keys,
            &urls.public_url,
            wallet_client_ids,
        ),
    });

    let issuance_router = Router::new()
        .route("/.well-known/openid-credential-issuer", get(metadata))
        .route("/.well-known/oauth-authorization-server", get(oauth_metadata))
        .route("/token", post(token))
        .route("/credential", post(credential))
        .route("/credential", delete(reject_issuance))
        .route("/batch_credential", post(batch_credential))
        .route("/batch_credential", delete(reject_issuance))
        .with_state(application_state);

    Ok(issuance_router)
}

// Although there is no standard here mandating what our error response looks like, we use `ErrorResponse`
// for consistency with the other endpoints.
async fn oauth_metadata<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
) -> Result<Json<oidc::Config>, ErrorResponse<MetadataError>>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<IssuanceData>,
{
    let metadata = state
        .issuer
        .oauth_metadata()
        .await
        .map_err(|error| openid4vc::ErrorResponse {
            error: MetadataError::Metadata,
            error_description: Some(error.to_string()),
            error_uri: None,
        })?;

    Ok(Json(metadata))
}

async fn metadata<A, K, S>(State(state): State<Arc<ApplicationState<A, K, S>>>) -> Json<IssuerMetadata> {
    Json(state.issuer.metadata.clone())
}

async fn token<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Form(token_request): Form<TokenRequest>,
) -> Result<(HeaderMap, Json<TokenResponseWithPreviews>), ErrorResponse<TokenErrorCode>>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<IssuanceData>,
{
    let (response, dpop_nonce) = state.issuer.process_token_request(token_request, dpop).await?;
    let headers = HeaderMap::from_iter([(
        HeaderName::from_str(DPOP_NONCE_HEADER_NAME).unwrap(),
        HeaderValue::from_str(&dpop_nonce).unwrap(),
    )]);
    Ok((headers, Json(response)))
}

async fn credential<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    TypedHeader(Authorization(authorization_header)): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Json(credential_request): Json<CredentialRequest>,
) -> Result<Json<CredentialResponse>, ErrorResponse<CredentialErrorCode>>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<IssuanceData>,
{
    let access_token = authorization_header.into();
    let response = state
        .issuer
        .process_credential(access_token, dpop, credential_request)
        .await?;
    Ok(Json(response))
}

async fn batch_credential<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    TypedHeader(Authorization(authorization_header)): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Json(credential_requests): Json<CredentialRequests>,
) -> Result<Json<CredentialResponses>, ErrorResponse<CredentialErrorCode>>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<IssuanceData>,
{
    let access_token = authorization_header.into();
    let response = state
        .issuer
        .process_batch_credential(access_token, dpop, credential_requests)
        .await?;
    Ok(Json(response))
}

async fn reject_issuance<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    TypedHeader(Authorization(authorization_header)): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    uri: Uri,
) -> Result<StatusCode, ErrorResponse<CredentialErrorCode>>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<IssuanceData>,
{
    let uri_path = &uri.path()[1..]; // strip off leading slash

    let access_token = authorization_header.into();
    state
        .issuer
        .process_reject_issuance(access_token, dpop, uri_path)
        .await?;
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
