use std::{collections::HashMap, str::FromStr, sync::Arc};

use axum::{
    extract::State,
    headers::{authorization::Credentials, Authorization, Header},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{delete, post},
    Form, Json, Router, TypedHeader,
};
use serde::Serialize;
use tower_http::trace::TraceLayer;

use nl_wallet_mdoc::{
    server_keys::{KeyPair, KeyRing},
    server_state::{SessionState, SessionStore},
};
use openid4vc::{
    credential::{CredentialErrorCode, CredentialRequest, CredentialRequests, CredentialResponse, CredentialResponses},
    dpop::{Dpop, DPOP_HEADER_NAME, DPOP_NONCE_HEADER_NAME},
    token::{AccessToken, TokenErrorCode, TokenRequest, TokenResponseWithPreviews},
    ErrorStatusCode,
};
use tracing::warn;

use crate::settings::{self, Settings};

use openid4vc::issuer::{AttributeService, IssuanceData, Issuer};

struct ApplicationState<A, K, S> {
    issuer: Issuer<A, K, S>,
}

pub struct IssuerKeyRing(pub HashMap<String, KeyPair>);

impl KeyRing for IssuerKeyRing {
    fn private_key(&self, usecase: &str) -> Option<&KeyPair> {
        self.0.get(usecase)
    }
}

impl TryFrom<HashMap<String, settings::KeyPair>> for IssuerKeyRing {
    type Error = nl_wallet_mdoc::Error;

    fn try_from(private_keys: HashMap<String, settings::KeyPair>) -> Result<Self, Self::Error> {
        Ok(Self(
            private_keys
                .into_iter()
                .map(|(doctype, keypair)| Ok((doctype, KeyPair::from_der(&keypair.private_key, &keypair.certificate)?)))
                .collect::<Result<_, Self::Error>>()?,
        ))
    }
}

pub async fn create_issuance_router<A, S>(settings: Settings, sessions: S, attr_service: A) -> anyhow::Result<Router>
where
    A: AttributeService + Send + Sync + 'static,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    let application_state = Arc::new(ApplicationState {
        issuer: Issuer::new(
            sessions,
            attr_service,
            IssuerKeyRing::try_from(settings.issuer.private_keys)?,
            &settings.public_url,
            settings.issuer.wallet_client_ids,
        ),
    });

    let issuance_router = Router::new()
        .route("/token", post(token))
        .route("/credential", post(credential))
        .route("/credential", delete(reject_issuance))
        .route("/batch_credential", post(batch_credential))
        .route("/batch_credential", delete(reject_issuance))
        .layer(TraceLayer::new_for_http())
        .with_state(application_state);

    Ok(issuance_router)
}

async fn token<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Form(token_request): Form<TokenRequest>,
) -> Result<(HeaderMap, Json<TokenResponseWithPreviews>), ErrorResponse<TokenErrorCode>>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>>,
{
    let (response, dpop_nonce) = state
        .issuer
        .process_token_request(token_request, dpop)
        .await
        .map_err(|err| ErrorResponse(err.into()))?;
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
    S: SessionStore<Data = SessionState<IssuanceData>>,
{
    let access_token = authorization_header.into();
    let response = state
        .issuer
        .process_credential(access_token, dpop, credential_request)
        .await
        .map_err(|err| ErrorResponse(err.into()))?;
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
    S: SessionStore<Data = SessionState<IssuanceData>>,
{
    let access_token = authorization_header.into();
    let response = state
        .issuer
        .process_batch_credential(access_token, dpop, credential_requests)
        .await
        .map_err(|err| ErrorResponse(err.into()))?;
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
    S: SessionStore<Data = SessionState<IssuanceData>>,
{
    let uri_path = &uri.path()[1..]; // strip off leading slash

    let access_token = authorization_header.into();
    state
        .issuer
        .process_reject_issuance(access_token, dpop, uri_path)
        .await
        .map_err(|err| ErrorResponse(err.into()))?;
    Ok(StatusCode::NO_CONTENT)
}

/// Newtype of [`openid4vc::ErrorResponse`] so that we can implement [`IntoResponse`] on it.
#[derive(Serialize, Debug)]
struct ErrorResponse<T>(openid4vc::ErrorResponse<T>);

impl<T: ErrorStatusCode + Serialize + std::fmt::Debug> IntoResponse for ErrorResponse<T> {
    fn into_response(self) -> Response {
        warn!("{:?}", &self);
        (self.0.error.status_code(), Json(self)).into_response()
    }
}

static DPOP_HEADER_NAME_LOWERCASE: HeaderName = HeaderName::from_static("dpop");

pub struct DpopHeader(Dpop);

impl Header for DpopHeader {
    fn name() -> &'static HeaderName {
        &DPOP_HEADER_NAME_LOWERCASE
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        // Exactly one value must be provided
        let value = values.next().ok_or(axum::headers::Error::invalid())?;
        if values.next().is_some() {
            return Err(axum::headers::Error::invalid());
        }

        let value = value.to_str().map_err(|_| axum::headers::Error::invalid())?.to_string();
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
