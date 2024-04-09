use std::{collections::HashMap, str::FromStr, sync::Arc};

use axum::{
    extract::State,
    headers::{authorization::Credentials, Authorization, Header},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Form, Json, Router, TypedHeader,
};
use serde::Serialize;

use nl_wallet_mdoc::{
    server_keys::{KeyPair, KeyRing},
    server_state::{SessionState, SessionStore},
};
use openid4vc::{
    credential::{CredentialErrorCode, CredentialRequest, CredentialRequests, CredentialResponse, CredentialResponses},
    dpop::{Dpop, DPOP_HEADER_NAME, DPOP_NONCE_HEADER_NAME},
    metadata::{CredentialResponseEncryption, IssuerData, IssuerMetadata},
    token::{AccessToken, TokenErrorCode, TokenRequest, TokenResponseWithPreviews},
    ErrorStatusCode,
};
use tracing::warn;
use wallet_common::{config::wallet_config::BaseUrl, reqwest::trusted_reqwest_client_builder};

use crate::settings::{self, Settings};

use openid4vc::issuer::{AttributeService, IssuanceData, Issuer};

struct ApplicationState<A, K, S> {
    issuer: Issuer<A, K, S>,
    metadata_http_client: reqwest::Client,
    digid_url: BaseUrl,
    token_url: BaseUrl,
    metadata: IssuerMetadata,
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
        metadata_http_client: trusted_reqwest_client_builder(settings.issuer.digid.trust_anchors.clone()).build()?,
        digid_url: settings.issuer.digid.issuer_url.clone(),
        token_url: settings.public_url.join_base_url("/issuance/token"),
        metadata: IssuerMetadata {
            issuer_config: IssuerData {
                credential_issuer: settings.public_url.join_base_url("/issuance"),
                authorization_servers: None,
                credential_endpoint: settings.public_url.join_base_url("/issuance/credential"),
                batch_credential_endpoint: Some(settings.public_url.join_base_url("/issuance/batch_credential")),
                deferred_credential_endpoint: None,
                notification_endpoint: None,
                credential_response_encryption: CredentialResponseEncryption {
                    alg_values_supported: vec![],
                    enc_values_supported: vec![],
                    encryption_required: false,
                },
                credential_identifiers_supported: Some(false),
                display: None,
                credential_configurations_supported: HashMap::new(),
            },
            signed_metadata: None,
        },
    });

    let issuance_router = Router::new()
        .route("/.well-known/openid-credential-issuer", get(metadata))
        .route("/.well-known/openid-configuration", get(oidc_metadata))
        .route("/token", post(token))
        .route("/credential", post(credential))
        .route("/credential", delete(reject_issuance))
        .route("/batch_credential", post(batch_credential))
        .route("/batch_credential", delete(reject_issuance))
        .with_state(application_state);

    Ok(issuance_router)
}

/// Get the OAuth metadata from the DigiD issuer, making no assumptions about its contents except that it is a JSON object.
/// Then, we override the `token_endpoint` to our own Token endpoint.
async fn oidc_metadata<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
) -> Result<Json<serde_json::Value>, ErrorResponse<MetadataError>> {
    let metadata: serde_json::Value = state
        .metadata_http_client
        .get(state.digid_url.join("/.well-known/openid-configuration"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let mut metadata = match metadata {
        serde_json::Value::Object(x) => Ok(x),
        _ => Err(ErrorResponse(openid4vc::ErrorResponse {
            error: MetadataError::NotAnObject,
            error_description: None,
            error_uri: None,
        })),
    }?;
    metadata["token_endpoint"] = serde_json::Value::String(state.token_url.as_ref().to_string());

    Ok(Json(serde_json::Value::Object(metadata)))
}

async fn metadata<A, K, S>(State(state): State<Arc<ApplicationState<A, K, S>>>) -> Json<IssuerMetadata> {
    Json(state.metadata.clone())
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

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
enum MetadataError {
    Transport,
    NotAnObject,
}

impl ErrorStatusCode for MetadataError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<reqwest::Error> for ErrorResponse<MetadataError> {
    fn from(error: reqwest::Error) -> Self {
        ErrorResponse(openid4vc::ErrorResponse {
            error: MetadataError::Transport,
            error_description: Some(error.to_string()),
            error_uri: None,
        })
    }
}
