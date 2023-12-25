use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::State,
    headers::{authorization::Credentials, Authorization, Header},
    response::{IntoResponse, Response},
    routing::{delete, post},
    Form, Json, Router, TypedHeader,
};
use http::{HeaderName, HeaderValue, StatusCode};
use serde::Serialize;
use tower_http::trace::TraceLayer;

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    server_keys::{KeyRing, PrivateKey},
    server_state::{SessionState, SessionStore},
    utils::serialization::CborBase64,
    IssuerSigned,
};
use openid4vc::{
    credential::{CredentialErrorType, CredentialRequest, CredentialRequests},
    dpop::Dpop,
    token::{TokenErrorType, TokenRequest},
    ErrorStatusCode,
};
use tracing::warn;

use crate::{log_requests::log_request_response, settings::Settings};

use openid4vc::issuer::*;

struct ApplicationState<A, K, S> {
    issuer: Issuer<A, K, S>,
}

pub struct IssuerKeyRing(pub HashMap<String, PrivateKey>);

impl KeyRing for IssuerKeyRing {
    fn private_key(&self, usecase: &str) -> Option<&PrivateKey> {
        self.0.get(usecase)
    }
}

type CredentialResponse = openid4vc::credential::CredentialResponse<CborBase64<IssuerSigned>>;
type CredentialResponses = openid4vc::credential::CredentialResponses<CborBase64<IssuerSigned>>;
type TokenResponseWithPreviews = openid4vc::token::TokenResponseWithPreviews<UnsignedMdoc>;

pub async fn create_issuance_router<A, S>(settings: Settings, sessions: S, attr_service: A) -> anyhow::Result<Router>
where
    A: AttributeService,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    let application_state = Arc::new(ApplicationState {
        issuer: Issuer::new(
            sessions,
            attr_service,
            IssuerKeyRing(
                settings
                    .issuer
                    .private_keys
                    .into_iter()
                    .map(|(doctype, keypair)| {
                        Ok((
                            doctype,
                            PrivateKey::from_der(&keypair.private_key.0, &keypair.certificate.0)?,
                        ))
                    })
                    .collect::<anyhow::Result<HashMap<_, _>>>()?,
            ),
            settings.public_url.clone(),
            settings.issuer.wallet_client_ids.clone().into_iter().collect(),
        ),
    });

    let issuance_router = Router::new()
        .route("/token", post(token))
        .route("/credential", post(credential))
        .route("/credential", delete(reject_issuance))
        .route("/batch_credential", post(batch_credential))
        .route("/batch_credential", delete(reject_batch_issuance))
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(log_request_response))
        .with_state(application_state);

    Ok(issuance_router)
}

async fn token<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Form(token_request): Form<TokenRequest>,
) -> Result<Json<TokenResponseWithPreviews>, ErrorResponse<TokenErrorType>>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    let response = state.issuer.process_token_request(token_request, dpop).await?;
    Ok(Json(response))
}

async fn credential<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Json(credential_request): Json<CredentialRequest>,
) -> Result<Json<CredentialResponse>, ErrorResponse<CredentialErrorType>>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    let access_token = authorization_header.0.token();
    let response = state
        .issuer
        .process_credential(access_token, dpop, credential_request)
        .await?;
    Ok(Json(response))
}

async fn batch_credential<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
    Json(credential_requests): Json<CredentialRequests>,
) -> Result<Json<CredentialResponses>, ErrorResponse<CredentialErrorType>>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    let access_token = authorization_header.0.token();
    let response = state
        .issuer
        .process_batch_credential(access_token, dpop, credential_requests)
        .await?;
    Ok(Json(response))
}

async fn reject_issuance<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
) -> Result<StatusCode, ErrorResponse<CredentialErrorType>>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    let access_token = authorization_header.0.token();
    state
        .issuer
        .process_reject_issuance(access_token, dpop, "credential")
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn reject_batch_issuance<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<DpopBearer>>,
    TypedHeader(DpopHeader(dpop)): TypedHeader<DpopHeader>,
) -> Result<StatusCode, ErrorResponse<CredentialErrorType>>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    let access_token = authorization_header.0.token();
    state
        .issuer
        .process_reject_issuance(access_token, dpop, "batch_credential")
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Newtype of [`openid4vc::ErrorResponse`] so that we can implement [`IntoResponse`] on it.
#[derive(Serialize, Debug)]
struct ErrorResponse<T>(openid4vc::ErrorResponse<T>);

impl<T> From<openid4vc::ErrorResponse<T>> for ErrorResponse<T> {
    fn from(value: openid4vc::ErrorResponse<T>) -> Self {
        Self(value)
    }
}

impl<T: ErrorStatusCode + Serialize + std::fmt::Debug> IntoResponse for ErrorResponse<T> {
    fn into_response(self) -> Response {
        warn!("{:?}", &self);
        (self.0.error.status_code(), Json(self)).into_response()
    }
}

impl From<CredentialRequestError> for ErrorResponse<CredentialErrorType> {
    fn from(err: CredentialRequestError) -> ErrorResponse<CredentialErrorType> {
        let description = err.to_string();
        openid4vc::ErrorResponse {
            error: match err {
                CredentialRequestError::IssuanceError(err) => match err {
                    openid4vc::issuer::Error::UnexpectedState => CredentialErrorType::InvalidRequest,
                    openid4vc::issuer::Error::UnknownSession(_) => CredentialErrorType::InvalidRequest,
                    openid4vc::issuer::Error::SessionStore(_) => CredentialErrorType::ServerError,
                    Error::DpopInvalid(_) => CredentialErrorType::InvalidRequest,
                },
                CredentialRequestError::Unauthorized => CredentialErrorType::InvalidToken,
                CredentialRequestError::MalformedToken => CredentialErrorType::InvalidToken,
                CredentialRequestError::UseBatchIssuance => CredentialErrorType::InvalidRequest,
                CredentialRequestError::UnsupportedCredentialFormat(_) => {
                    CredentialErrorType::UnsupportedCredentialFormat
                }
                CredentialRequestError::MissingJwk => CredentialErrorType::InvalidProof,
                CredentialRequestError::IncorrectNonce => CredentialErrorType::InvalidProof,
                CredentialRequestError::UnsupportedJwtAlgorithm { expected: _, found: _ } => {
                    CredentialErrorType::InvalidProof
                }
                CredentialRequestError::JwtDecodingFailed(_) => CredentialErrorType::InvalidProof,
                CredentialRequestError::JwkConversion(_) => CredentialErrorType::InvalidProof,
                CredentialRequestError::CoseKeyConversion(_) => CredentialErrorType::ServerError,
                CredentialRequestError::MissingPrivateKey(_) => CredentialErrorType::ServerError,
                CredentialRequestError::AttestationSigning(_) => CredentialErrorType::ServerError,
                CredentialRequestError::CborSerialization(_) => CredentialErrorType::ServerError,
                CredentialRequestError::JsonSerialization(_) => CredentialErrorType::ServerError,
                CredentialRequestError::DoctypeMismatch => CredentialErrorType::InvalidCredentialRequest,
            },
            error_description: Some(description),
            error_uri: None,
        }
        .into()
    }
}

impl From<TokenRequestError> for ErrorResponse<TokenErrorType> {
    fn from(err: TokenRequestError) -> Self {
        let description = err.to_string();
        openid4vc::ErrorResponse {
            error: match err {
                TokenRequestError::IssuanceError(err) => match err {
                    openid4vc::issuer::Error::UnexpectedState => TokenErrorType::InvalidRequest,
                    openid4vc::issuer::Error::UnknownSession(_) => TokenErrorType::InvalidRequest,
                    openid4vc::issuer::Error::SessionStore(_) => TokenErrorType::ServerError,
                    Error::DpopInvalid(_) => TokenErrorType::InvalidRequest,
                },
                TokenRequestError::UnsupportedTokenRequestType => TokenErrorType::UnsupportedGrantType,
                TokenRequestError::AttributeService(_) => TokenErrorType::ServerError,
            },
            error_description: Some(description),
            error_uri: None,
        }
        .into()
    }
}

static DPOP_HEADER_NAME: HeaderName = HeaderName::from_static("dpop");

pub struct DpopHeader(Dpop);

impl Header for DpopHeader {
    fn name() -> &'static HeaderName {
        &DPOP_HEADER_NAME
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

        let str = value.to_str().map_err(|_| axum::headers::Error::invalid())?;
        Ok(DpopHeader(Dpop(str.into())))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(HeaderValue::from_bytes(self.0 .0 .0.as_bytes()));
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct DpopBearer(String);

impl DpopBearer {
    pub fn token(&self) -> &str {
        &self.0.as_str()["DPoP ".len()..]
    }
}

impl Credentials for DpopBearer {
    const SCHEME: &'static str = "DPoP";

    fn decode(value: &HeaderValue) -> Option<Self> {
        value.to_str().ok().map(|value| Self(value.to_string()))
    }

    fn encode(&self) -> HeaderValue {
        HeaderValue::from_str(&self.0).unwrap()
    }
}
