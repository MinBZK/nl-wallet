use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    response::{IntoResponse, Response},
    routing::{delete, post},
    Form, Json, Router, TypedHeader,
};
use http::StatusCode;
use serde::Serialize;
use tower_http::trace::TraceLayer;

use nl_wallet_mdoc::{
    server_keys::{KeyRing, PrivateKey},
    server_state::{MemorySessionStore, SessionState, SessionStore},
};
use openid4vc::{
    credential::{CredentialErrorType, CredentialRequest, CredentialRequests, CredentialResponse, CredentialResponses},
    token::{TokenErrorType, TokenRequest, TokenResponseWithPreviews},
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

pub async fn create_issuance_router<A: AttributeService>(
    settings: Settings,
    attr_service: A,
) -> anyhow::Result<Router> {
    let application_state = Arc::new(ApplicationState {
        issuer: Issuer::new(
            MemorySessionStore::new(),
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
            settings.issuer.credential_issuer_identifier.clone(),
            settings.issuer.wallet_client_ids.clone().into_iter().collect(),
        ),
    });

    let issuance_router = Router::new()
        .route("/token", post(token))
        .route("/credential", post(credential))
        .route("/credential", delete(reject_issuance))
        .route("/batch_credential", post(batch_credential))
        .route("/batch_credential", delete(reject_issuance))
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(log_request_response))
        .with_state(application_state);

    Ok(issuance_router)
}

async fn token<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    Form(token_request): Form<TokenRequest>,
) -> Result<Json<TokenResponseWithPreviews>, ErrorResponse<TokenErrorType>>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    let response = state.issuer.process_token_request(token_request).await?;
    Ok(Json(response))
}

async fn credential<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<Bearer>>,
    Json(credential_request): Json<CredentialRequest>,
) -> Result<Json<CredentialResponse>, ErrorResponse<CredentialErrorType>>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    let access_token = authorization_header.token();
    let response = state
        .issuer
        .process_credential(access_token, credential_request)
        .await?;
    Ok(Json(response))
}

async fn batch_credential<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<Bearer>>,
    Json(credential_requests): Json<CredentialRequests>,
) -> Result<Json<CredentialResponses>, ErrorResponse<CredentialErrorType>>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    let access_token = authorization_header.token();
    let response = state
        .issuer
        .process_batch_credential(access_token, credential_requests)
        .await?;
    Ok(Json(response))
}

async fn reject_issuance<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<Bearer>>,
) -> Result<StatusCode, ErrorResponse<CredentialErrorType>>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    state
        .issuer
        .process_reject_issuance(authorization_header.token())
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
