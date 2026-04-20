use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::Infallible;
use std::num::NonZero;
use std::ops::Add;
use std::sync::Arc;
use std::time::Duration;

use chrono::DateTime;
use chrono::Days;
use chrono::DurationRound;
use chrono::Utc;
use derive_more::AsRef;
use derive_more::Debug;
use derive_more::From;
use futures::future::try_join_all;
use indexmap::IndexMap;
use itertools::Itertools;
use p256::ecdsa::VerifyingKey;
use reqwest::Method;
use serde::Deserialize;
use serde::Serialize;
use ssri::Integrity;
use tokio::task::AbortHandle;
use tracing::info;
use tracing::warn;
use uuid::Uuid;

use attestation_data::attributes::AttributesError;
use attestation_data::credential_payload::CredentialPayload;
use attestation_data::credential_payload::CredentialPayloadError;
use attestation_data::credential_payload::MdocCredentialPayloadError;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::credential_payload::SdJwtCredentialPayloadError;
use attestation_data::issuable_document::IssuableDocument;
use attestation_types::qualification::AttestationQualification;
use attestation_types::status_claim::StatusClaim;
use crypto::EcdsaKeySend;
use crypto::server_keys::KeyPair;
use crypto::utils::random_string;
use http_utils::urls::BaseUrl;
use http_utils::urls::HttpsUri;
use jwt::Algorithm;
use jwt::EcdsaDecodingKey;
use jwt::Validation;
use jwt::error::JwkConversionError;
use jwt::error::JwtError;
use jwt::nonce::Nonce;
use jwt::wua::WuaDisclosure;
use jwt::wua::WuaError;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataChainError;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use token_status_list::status_list_service::StatusListServices;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use wscd::Poa;
use wscd::PoaVerificationError;

use crate::Format;
use crate::credential::Credential;
use crate::credential::CredentialRequest;
use crate::credential::CredentialRequestProof;
use crate::credential::CredentialRequests;
use crate::credential::CredentialResponse;
use crate::credential::CredentialResponses;
use crate::dpop::Dpop;
use crate::dpop::DpopError;
use crate::issuer_identifier::IssuerIdentifier;
use crate::metadata::issuer_metadata::CredentialConfiguration;
use crate::metadata::issuer_metadata::IssuerMetadata;
use crate::metadata::issuer_metadata::ProofType;
use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
use crate::nonce::store::NonceStatus;
use crate::nonce::store::NonceStore;
use crate::nonce::store::NonceStoreError;
use crate::preview::CredentialPreviewRequest;
use crate::preview::CredentialPreviewResponse;
use crate::recurring_task::start_recurring_task;
use crate::server_state::Expirable;
use crate::server_state::HasProgress;
use crate::server_state::Progress;
use crate::server_state::SessionDataType;
use crate::server_state::SessionState;
use crate::server_state::SessionStore;
use crate::server_state::SessionStoreError;
use crate::server_state::SessionToken;
use crate::token::AccessToken;
use crate::token::AuthorizationCode;
use crate::token::CredentialPreview;
use crate::token::CredentialPreviewContent;
use crate::token::TokenRequest;
use crate::token::TokenRequestGrantType;
use crate::token::TokenResponse;

/// The cleanup task that removes stale sessions runs every so often.
const CLEANUP_INTERVAL: Duration = Duration::from_secs(120);

// Errors are structured as follows in this module: the handler for a token request on the one hand, and the handlers
// for the other endpoints on the other hand, have specific error types. (There is also a general error type included
// by both of them for errors that can occur in all endpoints.) The reason for this split in the errors is that per
// the OpenID4VCI and OAuth specs, these endpoints each have to return error codes that are specific to them, i.e., the
// token request endpoint can return error codes that the credential endpoint can't and vice versa, so we want to keep
// the errors separate in the type system here.

/// Errors that can occur during processing of any endpoint.
#[derive(Debug, thiserror::Error)]
pub enum IssuanceError {
    #[error("session not in expected state")]
    UnexpectedState,

    #[error("unknown session: {0:?}")]
    UnknownSession(AuthorizationCode),

    #[error("failed to retrieve session: {0}")]
    SessionStore(#[from] SessionStoreError),

    #[error("invalid DPoP header: {0}")]
    DpopInvalid(#[source] DpopError),
}

/// Errors that can occur during processing of the token request.
#[derive(Debug, thiserror::Error)]
pub enum TokenRequestError {
    #[error("issuance error: {0}")]
    IssuanceError(#[from] IssuanceError),

    #[error("unexpected grant type for this session: expected {expected}, got {actual}")]
    UnexpectedGrantType {
        expected: &'static str,
        actual: &'static str,
    },

    #[error("failed to get attributes to be issued: {0}")]
    AttributeService(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("attributes do not match type metadata: {0}")]
    AttributesError(#[from] AttributesError),

    #[error("credential type not offered: {0}")]
    CredentialTypeNotOffered(String),
}

/// Errors that can occur during handling of the (batch) credential request.
#[derive(Debug, thiserror::Error)]
pub enum CredentialRequestError {
    #[error("issuance error: {0}")]
    IssuanceError(#[from] IssuanceError),

    #[error("unauthorized: incorrect access token")]
    Unauthorized,

    #[error("malformed access token")]
    MalformedToken,

    #[error("credential type not offered")]
    CredentialTypeNotOffered(String),

    #[error("credential request ambiguous, use /batch_credential instead")]
    UseBatchIssuance,

    #[error("invalid proof JWT: {0}")]
    InvalidProofJwt(#[source] JwtError),

    #[error("could not extract holder public key from proof JWT: {0}")]
    InvalidProofPublicKey(#[source] JwkConversionError),

    #[error("nonce is not provided in credential request proof")]
    MissingProofNonce,

    #[error("could not check proof nonce against nonce storage: {0}")]
    ProofNonceStore(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("invalid nonce used in credential request proof, WUA or PoA")]
    InvalidNonce,

    #[error("JWT error: {0}")]
    Jwt(#[from] JwtError),

    #[error("missing attestation type config for {0}")]
    MissingAttestationTypeConfiguration(String),

    #[error("mismatch between requested: {requested} and offered attestation types: {offered}")]
    CredentialTypeMismatch { requested: Format, offered: Format },

    #[error("wrong number of credential requests")]
    WrongNumberOfCredentialRequests,

    #[error("missing credential request proof of possession")]
    MissingCredentialRequestPoP,

    #[error("missing WUA")]
    MissingWua,

    #[error("missing PoA")]
    MissingPoa,

    #[error("error verifying PoA: {0}")]
    PoaVerification(#[from] PoaVerificationError),

    #[error("error converting PreviewableCredentialPayload to CredentialPayload: {0}")]
    PreviewConversion(#[from] CredentialPayloadError),

    #[error("error converting CredentialPayload to Mdoc: {0}")]
    MdocConversion(#[from] MdocCredentialPayloadError),

    #[error("error converting CredentialPayload to SD-JWT: {0}")]
    SdJwtConversion(#[from] SdJwtCredentialPayloadError),

    #[error("error verifying WUA: {0}")]
    Wua(#[from] WuaError),

    #[error("error obtaining status claim: {0}")]
    ObtainStatusClaim(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("incorrect number of status claims for attestation_type: {0}")]
    IncorrectNumberOfStatusClaims(String),
}

/// Errors that can occur during handling of the credential preview request.
#[derive(Debug, thiserror::Error)]
pub enum CredentialPreviewError {
    #[error("issuance error: {0}")]
    IssuanceError(#[from] IssuanceError),

    #[error("malformed access token")]
    MalformedToken,

    #[error("unauthorized: incorrect access token")]
    Unauthorized,

    #[error("unknown credential identifier: {0}")]
    UnknownCredentialIdentifier(String),

    #[error("missing attestation type config for {0}")]
    MissingAttestationTypeConfig(String),

    #[error("requested credential previews not found in session")]
    CredentialPreviewsNotFound,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Created {
    pub issuable_documents: Option<VecNonEmpty<IssuableDocument>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitingForResponse {
    pub access_token: AccessToken,
    pub accepted_wallet_client_ids: Vec<String>,
    pub credential_previews: Vec<CredentialPreviewState>,
    pub dpop_public_key: VerifyingKey,
    pub dpop_nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialPreviewState {
    /// The amount of copies of this attestation that the holder will receive per credential format. This is serialized
    /// as a list of pairs in order to guarantee the order across system boundaries.
    #[serde(with = "indexmap::map::serde_seq")]
    pub copies_per_format: IndexMap<Format, NonZero<u8>>,
    pub credential_payload: PreviewableCredentialPayload,
    pub batch_id: Uuid,
}

impl CredentialPreviewState {
    fn from(value: CredentialPreview, batch_id: Uuid) -> Self {
        Self {
            copies_per_format: value.content.copies_per_format,
            credential_payload: value.content.credential_payload,
            batch_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Done {
    pub session_result: SessionResult,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IssuanceData {
    Created(Created),
    WaitingForResponse(Box<WaitingForResponse>),
    Done(Done),
}

impl SessionDataType for IssuanceData {
    const TYPE: &'static str = "openid4vci_issuance";
}

impl HasProgress for IssuanceData {
    fn progress(&self) -> Progress {
        match self {
            Self::Created(_) | Self::WaitingForResponse(_) => Progress::Active,
            Self::Done(done) => Progress::Finished {
                has_succeeded: matches!(done.session_result, SessionResult::Done),
            },
        }
    }
}

impl Expirable for IssuanceData {
    fn is_expired(&self) -> bool {
        matches!(
            self,
            Self::Done(Done {
                session_result: SessionResult::Expired
            })
        )
    }

    fn expire(&mut self) {
        *self = Self::Done(Done {
            session_result: SessionResult::Expired,
        });
    }
}

pub trait IssuanceState {}
impl IssuanceState for Created {}
impl IssuanceState for WaitingForResponse {}
impl IssuanceState for Done {}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE", tag = "status")]
pub enum SessionResult {
    Done,
    Failed { error: String },
    Cancelled,
    Expired,
}

#[derive(Debug)]
pub struct Session<S: IssuanceState> {
    pub state: SessionState<S>,
}

/// Implementations of this trait are responsible for determining the attributes to be issued, given the session and
/// the token request. See for example the [`BrpPidAttributeService`].
#[trait_variant::make(Send)]
pub trait AttributeService {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn attributes(&self, token_request: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Self::Error>;
}

impl AttributeService for () {
    type Error = Infallible;

    async fn attributes(&self, _: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Infallible> {
        unimplemented!("() AttributeService does not provide attributes")
    }
}

/// Static attestation data shared across all instances of an attestation type. The issuer augments this with an
/// [`IssuableDocument`] to form the attestation.
#[derive(Debug)]
pub struct AttestationTypeConfig<K> {
    #[debug(skip)]
    pub key_pair: KeyPair<K>,
    pub valid_days: Days,
    pub copies_per_format: IndexMap<Format, NonZero<u8>>,
    pub issuer_uri: HttpsUri,
    pub attestation_qualification: AttestationQualification,
    #[debug(skip)]
    pub metadata_documents: TypeMetadataDocuments,
    first_metadata_integrity: Integrity,
    metadata: NormalizedTypeMetadata,
}

impl<K> AttestationTypeConfig<K> {
    /// Create a new [`AttestationTypeConfig`] and decode and validate the type metadata documents.
    #[expect(clippy::too_many_arguments, reason = "Constructor")]
    pub fn try_new(
        attestation_type: &str,
        key_pair: KeyPair<K>,
        valid_days: Days,
        copies_per_format: IndexMap<Format, NonZero<u8>>,
        issuer_uri: HttpsUri,
        attestation_qualification: AttestationQualification,
        metadata_documents: TypeMetadataDocuments,
    ) -> Result<Self, TypeMetadataChainError> {
        // Calculate and cache the integrity hash for the first metadata document in the chain.
        let first_metadata_integrity = Integrity::from(metadata_documents.as_ref().first().as_slice());
        let (metadata, sorted_documents) = metadata_documents.into_normalized(attestation_type)?;

        let config = Self {
            key_pair,
            valid_days,
            copies_per_format,
            issuer_uri,
            attestation_qualification,
            metadata_documents: sorted_documents.into(),
            first_metadata_integrity,
            metadata,
        };

        Ok(config)
    }
}

/// Static attestation data indexed by attestation type.
#[derive(Debug, From, AsRef)]
pub struct AttestationTypesConfig<K>(HashMap<String, AttestationTypeConfig<K>>);

pub struct Issuer<K, A, S, N, L> {
    attr_service: A,
    issuer_data: IssuerData<K>,
    sessions: Arc<S>,
    proof_nonce_store: N,
    status_list_services: Arc<L>,
    cleanup_task: AbortHandle,
}

/// Fields of the [`Issuer`] needed by the issuance functions.
pub struct IssuerData<K> {
    attestation_config: AttestationTypesConfig<K>,
    wua_config: Option<WuaConfig>,

    /// Wallet IDs accepted by this server, MUST be used by the wallet as `iss` in its PoP JWTs.
    accepted_wallet_client_ids: Vec<String>,

    /// URL prefix of the `/token`, `/credential` and `/batch_crededential` endpoints.
    server_url: BaseUrl,

    metadata: IssuerMetadata,

    /// The upstream OAuth identifier, if any.
    upstream_oauth_identifier: Option<IssuerIdentifier>,
}

pub struct WuaConfig {
    /// Public key of the WUA issuer.
    pub wua_issuer_pubkey: EcdsaDecodingKey,
}

impl<K, A, S, N, L> Drop for Issuer<K, A, S, N, L> {
    fn drop(&mut self) {
        // Stop the tasks at the next .await
        self.cleanup_task.abort();
    }
}

impl<K, A, S, N, L> Issuer<K, A, S, N, L> {
    pub fn metadata(&self) -> &IssuerMetadata {
        &self.issuer_data.metadata
    }
}

impl<K, A, S, N, L> Issuer<K, A, S, N, L>
where
    S: SessionStore<IssuanceData> + Sync + 'static,
{
    #[expect(clippy::too_many_arguments, reason = "Constructor")]
    pub fn new(
        issuer_identifier: IssuerIdentifier,
        wallet_client_ids: Vec<String>,
        attestation_config: AttestationTypesConfig<K>,
        wua_config: Option<WuaConfig>,
        upstream_oauth_identifier: Option<IssuerIdentifier>,
        attr_service: A,
        sessions: Arc<S>,
        proof_nonce_store: N,
        status_list_services: Arc<L>,
    ) -> Self {
        let credential_configurations_supported = attestation_config
            .as_ref()
            .iter()
            .flat_map(|(attestation_type, config)| {
                config.copies_per_format.keys().flat_map(move |format| {
                    // TODO (PVW-5554): Include the credential configuration id in the settings, instead of
                    //                  hard coupling the AttestationTypeConfig key with the doctype / vct.
                    let config_id = format!("{attestation_type}_{format}");
                    // TODO (PVW-5548): Add "attestation" proof type.
                    let proof_types = vec![ProofType::Jwt];
                    let display = config.metadata.display().to_vec();
                    let claims = config.metadata.claims().to_vec();

                    match format {
                        Format::MsoMdoc => Some((
                            config_id,
                            CredentialConfiguration::new_mdoc_ecdsa_p256_sha256(
                                attestation_type.clone(),
                                proof_types,
                                display,
                                claims,
                            ),
                        )),
                        Format::SdJwt => Some((
                            config_id,
                            CredentialConfiguration::new_sd_jwt_ecdsa_p256_sha256(
                                attestation_type.clone(),
                                proof_types,
                                display,
                                claims,
                            ),
                        )),
                        _ => None,
                    }
                })
            })
            .collect();

        let server_url = issuer_identifier.join_issuer_url("/issuance");
        let credential_endpoint = server_url.join_issuer_url("/credential");
        let batch_credential_endpoint = server_url.join_issuer_url("/batch_credential");
        let nonce_endpoint = server_url.join_issuer_url("/nonce");
        let credential_preview_endpoint = server_url.join_issuer_url("/credential_preview");

        let metadata = IssuerMetadata {
            credential_issuer: issuer_identifier,
            authorization_servers: None,
            credential_endpoint,
            batch_credential_endpoint: Some(batch_credential_endpoint),
            nonce_endpoint: Some(nonce_endpoint),
            deferred_credential_endpoint: None,
            notification_endpoint: None,
            credential_request_encryption: None,
            credential_response_encryption: None,
            // TODO (PVW-5554): Configure batch size globally for the issuer and include it here.
            batch_credential_issuance: None,
            display: None,
            credential_configurations_supported,
            credential_preview_endpoint: Some(credential_preview_endpoint),
        };

        let issuer_data = IssuerData {
            attestation_config,
            accepted_wallet_client_ids: wallet_client_ids,
            wua_config,
            upstream_oauth_identifier,

            // In this implementation, the public server URL is composed of the
            // Credential Issuer Identifier appended with the "/issuance/" path.
            server_url: server_url.into_inner(),
            metadata,
        };

        let task_sessions = Arc::clone(&sessions);
        let cleanup_task = start_recurring_task(CLEANUP_INTERVAL, move || {
            let task_sessions = Arc::clone(&task_sessions);

            async move {
                if let Err(error) = task_sessions.cleanup().await {
                    warn!("error during session cleanup: {error}");
                }
            }
        });

        Self {
            issuer_data,
            attr_service,
            sessions,
            proof_nonce_store,
            status_list_services,
            cleanup_task,
        }
    }
}

fn logged_issuance_result<T, E: std::error::Error>(result: Result<T, E>) -> Result<T, E> {
    result
        .inspect(|_| info!("Issuance success"))
        .inspect_err(|error| info!("Issuance error: {error}"))
}

impl<K, A, S, N, L> Issuer<K, A, S, N, L>
where
    S: SessionStore<IssuanceData>,
{
    pub async fn new_session(
        &self,
        to_issue: VecNonEmpty<IssuableDocument>,
    ) -> Result<SessionToken, SessionStoreError> {
        let token = SessionToken::new_random();

        let session = SessionState::new(
            token.clone(),
            IssuanceData::Created(Created {
                issuable_documents: Some(to_issue),
            }),
        );

        self.sessions.write(session, true).await?;

        Ok(token)
    }

    async fn get_session(
        &self,
        code: AuthorizationCode,
    ) -> Result<Session<WaitingForResponse>, CredentialRequestError> {
        self.sessions
            .get(&code.clone().into())
            .await
            .map_err(IssuanceError::SessionStore)?
            .ok_or(IssuanceError::UnknownSession(code))?
            .try_into()
            .map_err(CredentialRequestError::IssuanceError)
    }
}

impl<K, A, S, N, L> Issuer<K, A, S, N, L>
where
    N: NonceStore,
{
    pub async fn generate_proof_nonce(&self) -> Result<Nonce, NonceStoreError<N::Error>> {
        let nonce = Nonce::new_random();

        self.proof_nonce_store.store_nonce(nonce.clone()).await?;

        Ok(nonce)
    }
}

impl<K, A, S, N, L> Issuer<K, A, S, N, L>
where
    S: SessionStore<IssuanceData>,
{
    pub async fn process_credential_preview(
        &self,
        access_token: AccessToken,
        request: CredentialPreviewRequest,
    ) -> Result<CredentialPreviewResponse, CredentialPreviewError> {
        let code = access_token.code().ok_or(CredentialPreviewError::MalformedToken)?;

        let session: Session<WaitingForResponse> = self
            .sessions
            .get(&code.clone().into())
            .await
            .map_err(IssuanceError::SessionStore)?
            .ok_or(IssuanceError::UnknownSession(code))?
            .try_into()
            .map_err(CredentialPreviewError::IssuanceError)?;

        let session_data = session.session_data();

        if session_data.access_token != access_token {
            return Err(CredentialPreviewError::Unauthorized);
        }

        let previews = match request {
            CredentialPreviewRequest::CredentialIdentifiers { .. } => {
                todo!("implement in PVW-5541")
            }
            CredentialPreviewRequest::CredentialConfigurationIds {
                credential_configuration_ids,
            } => {
                // Resolve credential_configuration_ids to attestation types by looking them up in the issuer metadata.
                let requested_attestation_types: HashSet<&str> = credential_configuration_ids
                    .iter()
                    .filter_map(|id| {
                        self.issuer_data
                            .metadata
                            .credential_configurations_supported
                            .get(id)
                            .and_then(|config| config.format.attestation_type())
                    })
                    .collect();

                // Return previews only for the types that are actually in the session; silently ignore IDs that appear
                // in the requested_attestation_types but are not part of this session.
                session_data
                    .credential_previews
                    .iter()
                    .filter(|preview_state| {
                        requested_attestation_types.contains(preview_state.credential_payload.attestation_type.as_str())
                    })
                    .map(|state| self.credential_preview_from_state(state))
                    .collect::<Result<Vec<_>, _>>()?
            }
        };

        Ok(CredentialPreviewResponse {
            credential_previews: previews
                .try_into()
                .map_err(|_| CredentialPreviewError::CredentialPreviewsNotFound)?,
        })
    }

    fn credential_preview_from_state(
        &self,
        state: &CredentialPreviewState,
    ) -> Result<CredentialPreview, CredentialPreviewError> {
        let attestation_type = &state.credential_payload.attestation_type;
        let config = self
            .issuer_data
            .attestation_config
            .as_ref()
            .get(attestation_type)
            .ok_or_else(|| CredentialPreviewError::MissingAttestationTypeConfig(attestation_type.clone()))?;

        Ok(CredentialPreview {
            content: CredentialPreviewContent {
                copies_per_format: state.copies_per_format.clone(),
                credential_payload: state.credential_payload.clone(),
                issuer_certificate: config.key_pair.certificate().clone(),
            },
            type_metadata: config.metadata_documents.clone(),
        })
    }
}

impl<K, A, S, N, L> Issuer<K, A, S, N, L>
where
    K: EcdsaKeySend,
    A: AttributeService,
    S: SessionStore<IssuanceData>,
{
    pub async fn process_token_request(
        &self,
        token_request: TokenRequest,
        dpop: Dpop,
    ) -> Result<(TokenResponse, String), TokenRequestError> {
        let session_token = token_request.code().clone().into();

        // Retrieve the session from the session store, if present. It need not be, depending on the implementation of
        // the attribute service.
        let maybe_session = self
            .sessions
            .get(&session_token)
            .await
            .map_err(IssuanceError::SessionStore)?;

        let (is_new_session, session) = match maybe_session {
            Some(session) => (false, session),
            None => (
                true,
                SessionState::<IssuanceData>::new(
                    session_token,
                    IssuanceData::Created(Created {
                        issuable_documents: None,
                    }),
                ),
            ),
        };

        let session: Session<Created> = session.try_into().map_err(TokenRequestError::IssuanceError)?;

        let result = session
            .process_token_request(
                token_request,
                &self.issuer_data.accepted_wallet_client_ids,
                dpop,
                &self.attr_service,
                &self.issuer_data.server_url,
                &self.issuer_data.attestation_config,
                is_new_session,
            )
            .await;

        let (response, next) = match result {
            Ok((response, dpop_nonce, next)) => (Ok((response, dpop_nonce)), next.into()),
            Err((err, next)) => (Err(err), next.into()),
        };

        self.sessions
            .write(next, is_new_session)
            .await
            .map_err(|e| TokenRequestError::IssuanceError(e.into()))?;

        response
    }
}

impl<K, A, S, N, L> Issuer<K, A, S, N, L>
where
    K: EcdsaKeySend,
    A: AttributeService,
    N: NonceStore,
    S: SessionStore<IssuanceData>,
    L: StatusListServices,
{
    pub async fn process_credential(
        &self,
        access_token: AccessToken,
        dpop: Dpop,
        credential_request: CredentialRequest,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        let code = access_token.code().ok_or(CredentialRequestError::MalformedToken)?;
        let session = self.get_session(code).await?;

        let (response, next) = session
            .process_credential(
                credential_request,
                access_token,
                dpop,
                &self.issuer_data,
                IssuerServices {
                    proof_nonce_store: &self.proof_nonce_store,
                    status_list_services: self.status_list_services.as_ref(),
                },
            )
            .await;

        self.sessions
            .write(next.into(), false)
            .await
            .map_err(IssuanceError::SessionStore)?;

        logged_issuance_result(response)
    }

    pub async fn process_batch_credential(
        &self,
        access_token: AccessToken,
        dpop: Dpop,
        credential_requests: CredentialRequests,
    ) -> Result<CredentialResponses, CredentialRequestError> {
        let code = access_token.code().ok_or(CredentialRequestError::MalformedToken)?;
        let session = self.get_session(code).await?;

        let (response, next) = session
            .process_batch_credential(
                credential_requests,
                access_token,
                dpop,
                &self.issuer_data,
                IssuerServices {
                    proof_nonce_store: &self.proof_nonce_store,
                    status_list_services: self.status_list_services.as_ref(),
                },
            )
            .await;

        self.sessions
            .write(next.into(), false)
            .await
            .map_err(IssuanceError::SessionStore)?;

        logged_issuance_result(response)
    }
}

impl<K, A, S, N, L> Issuer<K, A, S, N, L>
where
    S: SessionStore<IssuanceData>,
{
    pub async fn process_reject_issuance(
        &self,
        access_token: AccessToken,
        dpop: Dpop,
        endpoint_name: &str,
    ) -> Result<(), CredentialRequestError> {
        let code = access_token.code().ok_or(CredentialRequestError::MalformedToken)?;
        let session = self.get_session(code).await?;

        // Check authorization of the request
        let session_data = session.session_data();
        if session_data.access_token != access_token {
            return Err(CredentialRequestError::Unauthorized);
        }

        dpop.verify_expecting_key(
            &session_data.dpop_public_key,
            &self.issuer_data.server_url.join(endpoint_name),
            &Method::DELETE,
            Some(&access_token),
            Some(&session_data.dpop_nonce),
        )
        .map_err(|err| CredentialRequestError::IssuanceError(IssuanceError::DpopInvalid(err)))?;

        let next = session.transition(Done {
            session_result: SessionResult::Cancelled,
        });

        self.sessions
            .write(next.into(), false)
            .await
            .map_err(IssuanceError::SessionStore)?;

        Ok(())
    }
}

impl<K, A, S, N, L> Issuer<K, A, S, N, L>
where
    A: AttributeService,
{
    pub fn oauth_metadata(&self) -> AuthorizationServerMetadata {
        let issuer_url = self.issuer_data.metadata.credential_issuer.as_base_url();

        AuthorizationServerMetadata {
            authorization_endpoint: self
                .issuer_data
                .upstream_oauth_identifier
                .as_ref()
                // TODO (PVW-5746): decouple from upstream OAuth
                .map(|identifier| identifier.as_base_url().join("authorize")),
            ..AuthorizationServerMetadata::new(
                self.issuer_data.metadata.credential_issuer.clone(),
                issuer_url.join("issuance/token"),
            )
        }
    }
}

impl TryFrom<SessionState<IssuanceData>> for Session<Created> {
    type Error = IssuanceError;

    fn try_from(value: SessionState<IssuanceData>) -> Result<Self, Self::Error> {
        let IssuanceData::Created(session_data) = value.data else {
            return Err(IssuanceError::UnexpectedState);
        };
        Ok(Session::<Created> {
            state: SessionState {
                data: session_data,
                token: value.token,
                last_active: value.last_active,
            },
        })
    }
}

fn utc_now_truncated_to_days() -> DateTime<Utc> {
    Utc::now()
        .duration_trunc(chrono::Duration::days(1))
        .expect("should never exceed Unix time bounds")
}

impl Session<Created> {
    #[expect(clippy::too_many_arguments, reason = "Indirect constructor of a session")]
    async fn process_token_request(
        self,
        token_request: TokenRequest,
        accepted_wallet_client_ids: &[String],
        dpop: Dpop,
        attr_service: &impl AttributeService,
        server_url: &BaseUrl,
        attestation_settings: &AttestationTypesConfig<impl EcdsaKeySend>,
        is_new_session: bool,
    ) -> Result<(TokenResponse, String, Session<WaitingForResponse>), (TokenRequestError, Session<Done>)> {
        let result = self
            .process_token_request_inner(
                token_request,
                dpop,
                attr_service,
                server_url,
                attestation_settings,
                is_new_session,
            )
            .await;

        match result {
            Ok((token_response, previews, ids, dpop_pubkey, dpop_nonce)) => {
                let next = self.transition(WaitingForResponse {
                    access_token: token_response.access_token.clone(),
                    accepted_wallet_client_ids: accepted_wallet_client_ids.to_vec(),
                    credential_previews: previews
                        .into_iter()
                        // ids are unzipped from token_request issuable_documents which are transformed into previews
                        .zip_eq(ids)
                        .map(|(preview, id)| CredentialPreviewState::from(preview, id))
                        .collect(),
                    dpop_public_key: dpop_pubkey,
                    dpop_nonce: dpop_nonce.clone(),
                });
                Ok((token_response, dpop_nonce, next))
            }
            Err(err) => {
                let next = self.transition_fail(&err);
                Err((err, next))
            }
        }
    }

    fn id_and_credential_preview_from_issuable_document(
        document: IssuableDocument,
        attestation_data: &AttestationTypeConfig<impl EcdsaKeySend>,
    ) -> (Uuid, CredentialPreview) {
        // Truncate the current time to only include the date part, so that all issued credentials on a single
        // day have the same `nbf` and `exp` field
        let now = utc_now_truncated_to_days();
        let valid_until = now.add(attestation_data.valid_days);

        let (id, credential_payload) = document.into_id_and_previewable_credential_payload(
            now,
            valid_until,
            attestation_data.issuer_uri.clone(),
            attestation_data.attestation_qualification,
        );

        let preview = CredentialPreview {
            content: CredentialPreviewContent {
                copies_per_format: attestation_data.copies_per_format.clone(),
                credential_payload,
                issuer_certificate: attestation_data.key_pair.certificate().clone(),
            },
            type_metadata: attestation_data.metadata_documents.clone(),
        };
        (id, preview)
    }

    #[expect(clippy::too_many_arguments, reason = "Cascading effect because of constructor")]
    async fn process_token_request_inner(
        &self,
        token_request: TokenRequest,
        dpop: Dpop,
        attr_service: &impl AttributeService,
        server_url: &BaseUrl,
        attestation_settings: &AttestationTypesConfig<impl EcdsaKeySend>,
        is_new_session: bool,
    ) -> Result<
        (
            TokenResponse,
            VecNonEmpty<CredentialPreview>,
            VecNonEmpty<Uuid>,
            VerifyingKey,
            String,
        ),
        TokenRequestError,
    > {
        // Pre-populated sessions (e.g. disclosure-based issuance) must use PreAuthorizedCode.
        // New sessions (authorization code flow) must use AuthorizationCode.
        match (&token_request.grant_type, is_new_session) {
            (TokenRequestGrantType::AuthorizationCode { .. }, true)
            | (TokenRequestGrantType::PreAuthorizedCode { .. }, false) => {}
            (TokenRequestGrantType::PreAuthorizedCode { .. }, true) => {
                return Err(TokenRequestError::UnexpectedGrantType {
                    expected: "authorization_code",
                    actual: "urn:ietf:params:oauth:grant-type:pre-authorized_code",
                });
            }
            (TokenRequestGrantType::AuthorizationCode { .. }, false) => {
                return Err(TokenRequestError::UnexpectedGrantType {
                    expected: "urn:ietf:params:oauth:grant-type:pre-authorized_code",
                    actual: "authorization_code",
                });
            }
        }

        let dpop_public_key = dpop
            .verify(&server_url.join("token"), &Method::POST, None)
            .map_err(|err| TokenRequestError::IssuanceError(IssuanceError::DpopInvalid(err)))?;

        let code = token_request.code().clone();

        let issuables = match &self.session_data().issuable_documents {
            Some(docs) => docs.clone(),
            None => attr_service
                .attributes(token_request)
                .await
                .map_err(|e| TokenRequestError::AttributeService(Box::new(e)))?,
        };

        let (ids, previews) = issuables
            .into_nonempty_iter()
            .map(|document| {
                let attestation_data = attestation_settings
                    .as_ref()
                    .get(document.attestation_type())
                    .ok_or_else(|| {
                        TokenRequestError::CredentialTypeNotOffered(document.attestation_type().to_string())
                    })?;

                document.validate_with_metadata(&attestation_data.metadata)?;
                let (id, preview) = Self::id_and_credential_preview_from_issuable_document(document, attestation_data);

                Ok((id, preview))
            })
            .collect::<Result<VecNonEmpty<(_, _)>, TokenRequestError>>()?
            .into_nonempty_iter()
            .unzip();

        let dpop_nonce = random_string(32);

        let token_response = TokenResponse::new(AccessToken::new(&code));

        Ok((token_response, previews, ids, dpop_public_key, dpop_nonce))
    }
}

impl From<Session<WaitingForResponse>> for SessionState<IssuanceData> {
    fn from(value: Session<WaitingForResponse>) -> Self {
        SessionState {
            data: IssuanceData::WaitingForResponse(Box::new(value.state.data)),
            token: value.state.token,
            last_active: value.state.last_active,
        }
    }
}

impl TryFrom<SessionState<IssuanceData>> for Session<WaitingForResponse> {
    type Error = IssuanceError;

    fn try_from(value: SessionState<IssuanceData>) -> Result<Self, Self::Error> {
        let IssuanceData::WaitingForResponse(session_data) = value.data else {
            return Err(IssuanceError::UnexpectedState);
        };
        Ok(Session::<WaitingForResponse> {
            state: SessionState {
                data: *session_data,
                token: value.token,
                last_active: value.last_active,
            },
        })
    }
}

struct IssuerServices<'a, N, S> {
    proof_nonce_store: &'a N,
    status_list_services: &'a S,
}

impl Session<WaitingForResponse> {
    async fn process_credential<'a, K, N, S>(
        self,
        credential_request: CredentialRequest,
        access_token: AccessToken,
        dpop: Dpop,
        issuer_data: &IssuerData<K>,
        services: IssuerServices<'a, N, S>,
    ) -> (Result<CredentialResponse, CredentialRequestError>, Session<Done>)
    where
        K: EcdsaKeySend,
        N: NonceStore,
        S: StatusListServices,
    {
        let result = self
            .process_credential_inner(credential_request, access_token, dpop, issuer_data, services)
            .await;

        // In case of success, transition the session to done. This means the client won't be able to reuse its access
        // token in more requests to this endpoint. (The OpenID4VCI and OAuth specs allow reuse of access tokens, but
        // don't forbid that a server doesn't allow that.)
        let next = match &result {
            Ok(_) => self.transition(Done {
                session_result: SessionResult::Done,
            }),
            Err(err) => self.transition_fail(err),
        };

        (result, next)
    }

    pub fn check_credential_endpoint_access(
        &self,
        access_token: &AccessToken,
        dpop: Dpop,
        server_url: &BaseUrl,
        endpoint: &str,
    ) -> Result<(), CredentialRequestError> {
        let session_data = self.session_data();

        // Check authorization of the request
        if session_data.access_token != *access_token {
            return Err(CredentialRequestError::Unauthorized);
        }

        // Check that the DPoP is valid and its key matches the one from the Token Request
        dpop.verify_expecting_key(
            &session_data.dpop_public_key,
            &server_url.join(endpoint),
            &Method::POST,
            Some(access_token),
            Some(&session_data.dpop_nonce),
        )
        .map_err(|err| CredentialRequestError::IssuanceError(IssuanceError::DpopInvalid(err)))?;

        Ok(())
    }

    fn verify_wua(
        &self,
        wua_config: &WuaConfig,
        attestations: Option<&WuaDisclosure>,
        issuer_identifier: &str,
    ) -> Result<(VerifyingKey, Nonce), CredentialRequestError> {
        let wua_disclosure = attestations.ok_or(CredentialRequestError::MissingWua)?;

        let (wua_pubkey, wua_nonce) = wua_disclosure.verify(
            &wua_config.wua_issuer_pubkey,
            issuer_identifier,
            &self.state.data.accepted_wallet_client_ids,
        )?;

        Ok((wua_pubkey, wua_nonce))
    }

    pub fn verify_wua_and_poa<K>(
        &self,
        attestations: Option<&WuaDisclosure>,
        poa: Option<Poa>,
        attestation_keys: impl Iterator<Item = VerifyingKey>,
        issuer_data: &IssuerData<K>,
    ) -> Result<(Option<Nonce>, Nonce), CredentialRequestError> {
        let issuer_identifier = issuer_data.metadata.credential_issuer.as_ref();

        let (attestation_keys, wua_nonce) = match &issuer_data.wua_config {
            None => (attestation_keys.collect_vec(), None),
            Some(wua) => {
                let (wua_pubkey, wua_nonce) = self.verify_wua(wua, attestations, issuer_identifier)?;
                (attestation_keys.chain([wua_pubkey]).collect_vec(), Some(wua_nonce))
            }
        };

        let poa_nonce = poa.ok_or(CredentialRequestError::MissingPoa)?.verify_returning_nonce(
            &attestation_keys,
            issuer_identifier,
            &issuer_data.accepted_wallet_client_ids,
        )?;

        Ok((wua_nonce, poa_nonce))
    }

    async fn process_credential_inner<'a, K, N, S>(
        &self,
        credential_request: CredentialRequest,
        access_token: AccessToken,
        dpop: Dpop,
        issuer_data: &IssuerData<K>,
        services: IssuerServices<'a, N, S>,
    ) -> Result<CredentialResponse, CredentialRequestError>
    where
        K: EcdsaKeySend,
        N: NonceStore,
        S: StatusListServices,
    {
        let session_data = self.session_data();

        self.check_credential_endpoint_access(&access_token, dpop, &issuer_data.server_url, "credential")?;

        // If we have exactly one credential on offer that matches the credential type that the client is
        // requesting, then we issue that credential.
        // NB: the OpenID4VCI specification leaves open how to make this decision, this is our own behaviour.
        let requested_format = credential_request.credential_type.as_ref().format();
        let offered_creds = session_data
            .credential_previews
            .iter()
            .filter(|preview| preview.copies_per_format.contains_key(&requested_format))
            .collect_vec();

        let preview = match (offered_creds.first(), offered_creds.len()) {
            (Some(preview), 1) => Ok(*preview),
            (_, 0) => Err(CredentialRequestError::CredentialTypeNotOffered(
                credential_request.credential_type.as_ref().to_string(),
            )),
            // If we have more than one credential on offer of the specified credential type then it is not clear which
            // one we should issue; abort
            _ => Err(CredentialRequestError::UseBatchIssuance),
        }?;

        let (holder_pubkey, request_nonce) = credential_request.verify(
            &issuer_data.accepted_wallet_client_ids,
            &issuer_data.metadata.credential_issuer,
        )?;

        let (wua_nonce, poa_nonce) = self.verify_wua_and_poa(
            credential_request.attestations.as_ref(),
            credential_request.poa,
            [holder_pubkey].into_iter(),
            issuer_data,
        )?;

        // Check the validity of all of the nonces used, which may be equal to each other.
        let nonce_status = services
            .proof_nonce_store
            .check_nonce_status_and_remove([&request_nonce, &poa_nonce].iter().copied().chain(wua_nonce.as_ref()))
            .await
            .map_err(|error| CredentialRequestError::ProofNonceStore(Box::new(error)))?;

        if !matches!(nonce_status, NonceStatus::AllValid) {
            return Err(CredentialRequestError::InvalidNonce);
        }

        let attestation_type = &preview.credential_payload.attestation_type;
        let config = issuer_data
            .attestation_config
            .as_ref()
            .get(attestation_type)
            .ok_or_else(|| CredentialRequestError::MissingAttestationTypeConfiguration(attestation_type.to_owned()))?;

        let status_claim = services
            .status_list_services
            .obtain_status_claims(
                &preview.credential_payload.attestation_type,
                preview.batch_id,
                preview.credential_payload.expires,
                NonZero::<usize>::MIN,
            )
            .await
            .map_err(|err| CredentialRequestError::ObtainStatusClaim(Box::new(err)))?
            .into_first();

        let credential_response = CredentialResponse::new(
            requested_format,
            preview.credential_payload.clone(),
            utc_now_truncated_to_days(),
            &holder_pubkey,
            config,
            status_claim,
        )
        .await?;

        Ok(credential_response)
    }

    async fn process_batch_credential<'a, K, N, S>(
        self,
        credential_requests: CredentialRequests,
        access_token: AccessToken,
        dpop: Dpop,
        issuer_data: &IssuerData<K>,
        services: IssuerServices<'a, N, S>,
    ) -> (Result<CredentialResponses, CredentialRequestError>, Session<Done>)
    where
        K: EcdsaKeySend,
        N: NonceStore,
        S: StatusListServices,
    {
        let result = self
            .process_batch_credential_inner(credential_requests, access_token, dpop, issuer_data, services)
            .await;

        // In case of success, transition the session to done. This means the client won't be able to reuse its access
        // token in more requests to this endpoint. (The OpenID4VCI and OAuth specs allow reuse of access tokens, but
        // don't forbid that a server doesn't allow that.)
        let next = match &result {
            Ok(_) => self.transition(Done {
                session_result: SessionResult::Done,
            }),
            Err(err) => self.transition_fail(err),
        };

        (result, next)
    }

    async fn process_batch_credential_inner<'a, K, N, S>(
        &self,
        credential_requests: CredentialRequests,
        access_token: AccessToken,
        dpop: Dpop,
        issuer_data: &IssuerData<K>,
        services: IssuerServices<'a, N, S>,
    ) -> Result<CredentialResponses, CredentialRequestError>
    where
        K: EcdsaKeySend,
        N: NonceStore,
        S: StatusListServices,
    {
        let session_data = self.session_data();

        self.check_credential_endpoint_access(&access_token, dpop, &issuer_data.server_url, "batch_credential")?;

        let mut request_nonces = Vec::with_capacity(credential_requests.credential_requests.as_ref().len());
        let previews_and_holder_pubkeys = session_data
            .credential_previews
            .iter()
            .map(|preview| {
                // For every preview collect for every copy the verified key and the format
                let format_pubkeys: VecNonEmpty<_> = preview
                    .copies_per_format
                    .iter()
                    .flat_map(|(format, copies)| itertools::repeat_n(*format, copies.get().into()))
                    .map(|format| {
                        let cred_req = credential_requests
                            .credential_requests
                            .as_ref()
                            .get(request_nonces.len())
                            .ok_or(CredentialRequestError::WrongNumberOfCredentialRequests)?;

                        // Verify the assumption that the order of the incoming requests matches exactly
                        // that of the flattened copies_per_format by matching the requested format.
                        if format != cred_req.credential_type.as_ref().format() {
                            return Err(CredentialRequestError::CredentialTypeMismatch {
                                offered: format,
                                requested: cred_req.credential_type.as_ref().format(),
                            });
                        }

                        let (key, nonce) = cred_req.verify(
                            &issuer_data.accepted_wallet_client_ids,
                            &issuer_data.metadata.credential_issuer,
                        )?;

                        request_nonces.push(nonce);

                        Ok((format, key))
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .unwrap(); // ok because copies_per_format has a NonZeroU8 value in AttestationConfig (source)

                let attestation_type = &preview.credential_payload.attestation_type;
                let config = issuer_data
                    .attestation_config
                    .as_ref()
                    .get(attestation_type)
                    .ok_or_else(|| {
                        CredentialRequestError::MissingAttestationTypeConfiguration(attestation_type.to_string())
                    })?;

                Ok((preview, config, format_pubkeys))
            })
            .collect::<Result<Vec<_>, CredentialRequestError>>()?;

        // Verify that we have consumed all credential requests
        if request_nonces.len() != credential_requests.credential_requests.as_ref().len() {
            return Err(CredentialRequestError::WrongNumberOfCredentialRequests);
        }

        let (wua_nonce, poa_nonce) = self.verify_wua_and_poa(
            credential_requests.attestations.as_ref(),
            credential_requests.poa,
            previews_and_holder_pubkeys
                .iter()
                .flat_map(|(_, _, format_pubkeys)| format_pubkeys.iter().map(|(_, key)| *key)),
            issuer_data,
        )?;

        // Check the validity of all of the nonces used, which may be equal to each other.
        let nonce_status = services
            .proof_nonce_store
            .check_nonce_status_and_remove(
                request_nonces
                    .iter()
                    .chain(wua_nonce.as_ref())
                    .chain(std::iter::once(&poa_nonce)),
            )
            .await
            .map_err(|error| CredentialRequestError::ProofNonceStore(Box::new(error)))?;

        if !matches!(nonce_status, NonceStatus::AllValid) {
            return Err(CredentialRequestError::InvalidNonce);
        }

        // Obtain a status claim for every attestation copy, linked to a single batch id per preview
        let status_claims = try_join_all(previews_and_holder_pubkeys.iter().map(
            |(preview, _, format_pubkeys)| async move {
                let claims = services
                    .status_list_services
                    .obtain_status_claims(
                        &preview.credential_payload.attestation_type,
                        preview.batch_id,
                        preview.credential_payload.expires,
                        format_pubkeys.len(),
                    )
                    .await
                    .map_err(|err| CredentialRequestError::ObtainStatusClaim(Box::new(err)))?;
                if claims.len() != format_pubkeys.len() {
                    return Err(CredentialRequestError::IncorrectNumberOfStatusClaims(
                        preview.credential_payload.attestation_type.clone(),
                    ));
                }
                Ok(claims)
            },
        ))
        .await?;

        // Make sure all credentials are issued with the same `issued_at` timestamp
        let issued_at = utc_now_truncated_to_days();
        let credential_responses = try_join_all(
            previews_and_holder_pubkeys
                .iter()
                // The claims size is explicitly checked to be equal to the number of copies
                .zip_eq(status_claims)
                .flat_map(|((preview, config, format_pubkeys), claims)| {
                    format_pubkeys
                        .into_iter()
                        .zip(claims.into_inner())
                        .map(|((format, key), claim)| {
                            CredentialResponse::new(
                                *format,
                                preview.credential_payload.clone(),
                                issued_at,
                                key,
                                config,
                                claim,
                            )
                        })
                }),
        )
        .await?;

        Ok(CredentialResponses { credential_responses })
    }
}

impl From<Session<Done>> for SessionState<IssuanceData> {
    fn from(value: Session<Done>) -> Self {
        SessionState {
            data: IssuanceData::Done(value.state.data),
            token: value.state.token,
            last_active: value.state.last_active,
        }
    }
}

// Transitioning functions and helpers valid for any state
impl<T: IssuanceState> Session<T> {
    /// Transition `self` to a new state, consuming the old state, also updating the `last_active` timestamp.
    pub fn transition<NewT: IssuanceState>(self, new_state: NewT) -> Session<NewT> {
        Session {
            state: SessionState::new(self.state.token, new_state),
        }
    }

    fn transition_fail(self, error: &impl ToString) -> Session<Done> {
        self.transition(Done {
            session_result: SessionResult::Failed {
                error: error.to_string(),
            },
        })
    }

    pub fn session_data(&self) -> &T {
        &self.state.data
    }
}

impl CredentialRequest {
    fn verify(
        &self,
        accepted_wallet_client_ids: &[impl ToString],
        credential_issuer_identifier: &IssuerIdentifier,
    ) -> Result<(VerifyingKey, Nonce), CredentialRequestError> {
        let (holder_pubkey, nonce) = self
            .proof
            .as_ref()
            .ok_or(CredentialRequestError::MissingCredentialRequestPoP)?
            .verify(accepted_wallet_client_ids, credential_issuer_identifier)?;

        Ok((holder_pubkey, nonce))
    }
}

impl CredentialResponse {
    async fn new(
        credential_format: Format,
        preview_credential_payload: PreviewableCredentialPayload,
        issued_at: DateTime<Utc>,
        holder_pubkey: &VerifyingKey,
        attestation_config: &AttestationTypeConfig<impl EcdsaKeySend>,
        status_claim: StatusClaim,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        let payload = CredentialPayload::from_previewable_credential_payload(
            preview_credential_payload,
            issued_at,
            holder_pubkey,
            &attestation_config.metadata,
            attestation_config.first_metadata_integrity.clone(),
            status_claim,
        )?;

        match credential_format {
            Format::MsoMdoc => Self::new_for_mdoc(payload, attestation_config).await,
            Format::SdJwt => Self::new_for_sd_jwt(payload, attestation_config).await,
            other => Err(CredentialRequestError::CredentialTypeNotOffered(other.to_string())),
        }
    }

    async fn new_for_mdoc(
        credential_payload: CredentialPayload,
        attestation_config: &AttestationTypeConfig<impl EcdsaKeySend + Sized>,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        // Construct an mdoc `IssuerSigned` from the contents of `PreviewableCredentialPayload`
        // and the attestation config by signing it.
        let (issuer_signed, _) = credential_payload
            .into_signed_mdoc(&attestation_config.key_pair)
            .await?;

        Ok(CredentialResponse::new_immediate(Credential::new_mdoc(issuer_signed)))
    }

    async fn new_for_sd_jwt(
        credential_payload: CredentialPayload,
        attestation_config: &AttestationTypeConfig<impl EcdsaKeySend + Sized>,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        let signed_sd_jwt = credential_payload
            .into_signed_sd_jwt(&attestation_config.metadata, &attestation_config.key_pair)
            .await?;

        Ok(CredentialResponse::new_immediate(Credential::new_sd_jwt(
            signed_sd_jwt.into_unverified(),
        )))
    }
}

impl CredentialRequestProof {
    pub fn verify(
        &self,
        accepted_wallet_client_ids: &[impl ToString],
        credential_issuer_identifier: &IssuerIdentifier,
    ) -> Result<(VerifyingKey, Nonce), CredentialRequestError> {
        let CredentialRequestProof::Jwt { jwt } = self;

        let mut validation_options = Validation::new(Algorithm::ES256);
        validation_options.set_required_spec_claims(&["iss", "aud"]);
        validation_options.set_issuer(accepted_wallet_client_ids);
        validation_options.set_audience(&[credential_issuer_identifier]);

        let (header, payload) = jwt
            .parse_and_verify_with_jwk(&validation_options)
            .map_err(CredentialRequestError::InvalidProofJwt)?;

        let public_key = header
            .verifying_key()
            .map_err(CredentialRequestError::InvalidProofPublicKey)?;

        let nonce = payload.nonce.ok_or(CredentialRequestError::MissingProofNonce)?;

        Ok((public_key, nonce))
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use chrono::Days;
    use chrono::Timelike;
    use derive_more::Debug;
    use indexmap::IndexMap;
    use p256::ecdsa::SigningKey;
    use rustls_pki_types::TrustAnchor;
    use thiserror::Error;
    use tracing_test::traced_test;
    use url::Url;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::issuable_document::IssuableDocument;
    use attestation_data::x509::generate::mock::generate_issuer_mock_with_registration;
    use attestation_types::qualification::AttestationQualification;
    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use jwt::JsonJwt;
    use jwt::UnverifiedJwt;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use wscd::Poa;
    use wscd::PoaPayload;
    use wscd::mock_remote::MockRemoteWscd;

    use crate::CredentialErrorCode;
    use crate::Format;
    use crate::credential::CredentialRequest;
    use crate::credential::CredentialRequestProof;
    use crate::credential::CredentialRequests;
    use crate::credential::CredentialResponse;
    use crate::credential::CredentialResponses;
    use crate::dpop::Dpop;
    use crate::issuer_identifier::IssuerIdentifier;
    use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
    use crate::nonce::response::NonceResponse;
    use crate::preview::CredentialPreviewRequest;
    use crate::preview::CredentialPreviewResponse;
    use crate::server_state::MemorySessionStore;
    use crate::server_state::test::memory_session_store_with_mock_time;
    use crate::server_state::test::test_memory_store_with_cleanup_task;
    use crate::test::MockAttrService;
    use crate::test::MockIssuer;
    use crate::test::mock_issuable_documents;
    use crate::test::setup_mock_issuer;
    use crate::token::AccessToken;
    use crate::token::TokenRequest;
    use crate::token::TokenResponse;
    use crate::wallet_issuance::IssuanceSession;
    use crate::wallet_issuance::WalletIssuanceError;
    use crate::wallet_issuance::issuance_session::HttpIssuanceSession;
    use crate::wallet_issuance::issuance_session::VcMessageClient;

    use super::*;

    #[derive(Debug, Error, Clone, Eq, PartialEq)]
    #[error("MyError")]
    struct MyError;

    #[test]
    fn test_credential_preview_from_issuable_document() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuance_keypair = generate_issuer_mock_with_registration(&ca, IssuerRegistration::new_mock()).unwrap();
        let document = IssuableDocument::new_mock_degree("Education".to_string());
        let config = AttestationTypeConfig::try_new(
            document.attestation_type(),
            KeyPair::new_from_signing_key(
                issuance_keypair.private_key().to_owned(),
                issuance_keypair.certificate().to_owned(),
            )
            .unwrap(),
            Days::new(1),
            IndexMap::from_iter([(Format::MsoMdoc, NonZero::<u8>::MIN)]),
            "https://example.com".parse().unwrap(),
            AttestationQualification::default(),
            TypeMetadataDocuments::degree_example().1,
        )
        .unwrap();

        let (_, preview) = Session::<Created>::id_and_credential_preview_from_issuable_document(document, &config);
        assert_eq!(
            preview.content.credential_payload.not_before.unwrap().as_ref().second(),
            0
        );
        assert_eq!(
            preview.content.credential_payload.not_before.unwrap().as_ref().minute(),
            0
        );
        assert_eq!(
            preview.content.credential_payload.not_before.unwrap().as_ref().hour(),
            0
        );
        assert_eq!(preview.content.credential_payload.expires.unwrap().as_ref().second(), 0);
        assert_eq!(preview.content.credential_payload.expires.unwrap().as_ref().minute(), 0);
        assert_eq!(preview.content.credential_payload.expires.unwrap().as_ref().hour(), 0);
    }

    #[traced_test]
    #[test]
    fn test_logged_issuance_result() {
        let mut input: Result<String, MyError>;

        assert!(!logs_contain("Issuance success"));
        input = Ok("Alright".into());
        let result = logged_issuance_result(input.clone());
        assert_eq!(result, input);
        assert!(logs_contain("Issuance success"));

        assert!(!logs_contain("Issuance error: MyError"));
        input = Err(MyError);
        let result = logged_issuance_result(input.clone());
        assert_eq!(result, input);
        assert!(logs_contain("Issuance error: MyError"));
    }

    // Error injection tests

    fn setup_simple_mock_issuer() -> (MockIssuer, TrustAnchor<'static>, IssuerIdentifier, SigningKey) {
        let issuer_identifier: IssuerIdentifier = "https://example.com/".parse().unwrap();
        let (issuer, trust_anchor, wua_issuer_privkey) = setup_mock_issuer(
            issuer_identifier.clone(),
            MockAttrService {
                documents: mock_issuable_documents(NonZeroUsize::MIN),
            },
            NonZeroUsize::MIN,
            Arc::new(MemorySessionStore::default()),
            None,
        );
        (issuer, trust_anchor, issuer_identifier, wua_issuer_privkey)
    }

    /// An implementation of [`VcMessageClient`] that dispatches messages directly to the contained
    /// issuer by function invocation, optionally allowing the caller to mess with the input to trigger
    /// certain error cases.
    ///
    /// NOTE: This bypasses HTTP transport, so the transport part of the OpenID4VCI implementation is
    /// not tested here. See `openid4vc_server/tests/issuance.rs` for full-stack integration tests.
    struct VcMessageClientStub {
        issuer: MockIssuer,

        wrong_access_token: bool,
        invalidate_dpop: bool,
        invalidate_pop: bool,
        invalidate_poa: bool,
        strip_poa: bool,
        strip_wua: bool,
    }

    impl VcMessageClientStub {
        fn new(issuer: MockIssuer) -> Self {
            Self {
                issuer,
                wrong_access_token: false,
                invalidate_dpop: false,
                invalidate_pop: false,
                invalidate_poa: false,
                strip_poa: false,
                strip_wua: false,
            }
        }

        fn access_token(&self, access_token_header: &str) -> AccessToken {
            if self.wrong_access_token {
                let code = &access_token_header[32 + 5..]; // Strip "DPoP "
                AccessToken::from("0".repeat(32) + code)
            } else {
                AccessToken::from(access_token_header[5..].to_string())
            }
        }

        fn dpop_header(&self, dpop_header: &str) -> Dpop {
            if self.invalidate_dpop {
                invalidate_jwt_str(dpop_header).as_str().parse().unwrap()
            } else {
                dpop_header.parse().unwrap()
            }
        }

        fn tamper_credential_request(&self, mut credential_request: CredentialRequest) -> CredentialRequest {
            if self.invalidate_pop {
                let invalidated_proof = match credential_request.proof.as_ref().unwrap() {
                    CredentialRequestProof::Jwt { jwt } => CredentialRequestProof::Jwt {
                        jwt: invalidate_jwt_str(jwt.serialization()).parse().unwrap(),
                    },
                };
                credential_request.proof = Some(invalidated_proof);
            }

            if self.invalidate_poa {
                credential_request.poa = Some(Self::tamper_poa(credential_request.poa.unwrap()));
            }

            if self.strip_poa {
                credential_request.poa.take();
            }

            if self.strip_wua {
                credential_request.attestations.take();
            }

            credential_request
        }

        fn tamper_credential_requests(&self, mut credential_requests: CredentialRequests) -> CredentialRequests {
            if self.invalidate_pop {
                let invalidated_request =
                    self.tamper_credential_request(credential_requests.credential_requests.first().clone());

                let mut requests = credential_requests.credential_requests.into_inner();
                requests[0] = invalidated_request;
                credential_requests.credential_requests = requests.try_into().unwrap();
            }

            if self.invalidate_poa {
                credential_requests.poa = Some(Self::tamper_poa(credential_requests.poa.unwrap()));
            }

            if self.strip_poa {
                credential_requests.poa.take();
            }

            if self.strip_wua {
                credential_requests.attestations.take();
            }

            credential_requests
        }

        fn tamper_poa(poa: Poa) -> Poa {
            let mut jwts: Vec<UnverifiedJwt<PoaPayload>> = poa.into();
            jwts.pop();
            let jwts: VecNonEmpty<_> = jwts.try_into().unwrap();
            let poa: JsonJwt<PoaPayload> = jwts.try_into().unwrap();
            poa.into()
        }
    }

    fn invalidate_jwt_str(jwt: &str) -> String {
        let new_char = if !jwt.ends_with('A') { 'A' } else { 'B' };
        jwt[..jwt.len() - 1].to_string() + &new_char.to_string()
    }

    impl VcMessageClient for VcMessageClientStub {
        async fn request_token(
            &self,
            _url: &Url,
            token_request: &TokenRequest,
            dpop_header: &Dpop,
        ) -> Result<(TokenResponse, Option<String>), WalletIssuanceError> {
            let (token_response, dpop_nonce) = self
                .issuer
                .process_token_request(token_request.clone(), dpop_header.clone())
                .await
                .map_err(|err| WalletIssuanceError::TokenRequest(Box::new(err.into())))?;
            Ok((token_response, Some(dpop_nonce)))
        }

        async fn request_credential_preview(
            &self,
            _url: &Url,
            preview_request: &CredentialPreviewRequest,
            access_token: &AccessToken,
        ) -> Result<CredentialPreviewResponse, WalletIssuanceError> {
            self.issuer
                .process_credential_preview(access_token.clone(), preview_request.clone())
                .await
                .map_err(|err| WalletIssuanceError::CredentialPreviewRequest(Box::new(err.into())))
        }

        async fn request_nonce(&self, _url: Url) -> Result<(NonceResponse, Option<String>), WalletIssuanceError> {
            let c_nonce = self.issuer.generate_proof_nonce().await.unwrap();
            Ok((NonceResponse { c_nonce }, None))
        }

        async fn request_credential(
            &self,
            _url: &Url,
            credential_request: &CredentialRequest,
            dpop_header: &str,
            access_token_header: &str,
        ) -> Result<CredentialResponse, WalletIssuanceError> {
            self.issuer
                .process_credential(
                    self.access_token(access_token_header),
                    self.dpop_header(dpop_header),
                    self.tamper_credential_request(credential_request.clone()),
                )
                .await
                .map_err(|err| WalletIssuanceError::CredentialRequest(Box::new(err.into())))
        }

        async fn request_credentials(
            &self,
            _url: &Url,
            credential_requests: &CredentialRequests,
            dpop_header: &str,
            access_token_header: &str,
        ) -> Result<CredentialResponses, WalletIssuanceError> {
            self.issuer
                .process_batch_credential(
                    self.access_token(access_token_header),
                    self.dpop_header(dpop_header),
                    self.tamper_credential_requests(credential_requests.clone()),
                )
                .await
                .map_err(|err| WalletIssuanceError::CredentialRequest(Box::new(err.into())))
        }

        async fn reject(
            &self,
            _url: &Url,
            dpop_header: &str,
            access_token_header: &str,
        ) -> Result<(), WalletIssuanceError> {
            self.issuer
                .process_reject_issuance(
                    self.access_token(access_token_header),
                    self.dpop_header(dpop_header),
                    "batch_credential",
                )
                .await
                .map_err(|err| WalletIssuanceError::CredentialRequest(Box::new(err.into())))
        }
    }

    async fn start_and_accept_err(
        message_client: VcMessageClientStub,
        issuer_identifier: IssuerIdentifier,
        trust_anchor: TrustAnchor<'static>,
        wua_issuer_privkey: SigningKey,
    ) -> WalletIssuanceError {
        let trust_anchors = &[trust_anchor];
        let issuer_metadata = message_client.issuer.metadata().clone();
        let oauth_metadata = AuthorizationServerMetadata::new_mock(issuer_identifier);
        let mut session = HttpIssuanceSession::create(
            message_client,
            issuer_metadata,
            oauth_metadata,
            TokenRequest::new_mock(),
            trust_anchors,
        )
        .await
        .unwrap();

        let wscd = MockRemoteWscd::new_with_wua_signing_key(wua_issuer_privkey);
        session.accept_issuance(trust_anchors, &wscd, true).await.unwrap_err()
    }

    #[tokio::test]
    async fn wrong_access_token() {
        let (issuer, trust_anchor, issuer_identifier, wua_issuer_privkey) = setup_simple_mock_issuer();
        let message_client = VcMessageClientStub {
            wrong_access_token: true,
            ..VcMessageClientStub::new(issuer)
        };

        let result = start_and_accept_err(message_client, issuer_identifier, trust_anchor, wua_issuer_privkey).await;
        assert_matches!(
            result,
            WalletIssuanceError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidToken)
        );
    }

    #[tokio::test]
    async fn invalid_dpop() {
        let (issuer, trust_anchor, issuer_identifier, wua_issuer_privkey) = setup_simple_mock_issuer();
        let message_client = VcMessageClientStub {
            invalidate_dpop: true,
            ..VcMessageClientStub::new(issuer)
        };

        let result = start_and_accept_err(message_client, issuer_identifier, trust_anchor, wua_issuer_privkey).await;
        assert_matches!(
            result,
            WalletIssuanceError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidCredentialRequest)
        );
    }

    #[tokio::test]
    async fn invalid_pop() {
        let (issuer, trust_anchor, issuer_identifier, wua_issuer_privkey) = setup_simple_mock_issuer();
        let message_client = VcMessageClientStub {
            invalidate_pop: true,
            ..VcMessageClientStub::new(issuer)
        };

        let result = start_and_accept_err(message_client, issuer_identifier, trust_anchor, wua_issuer_privkey).await;
        assert_matches!(
            result,
            WalletIssuanceError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidProof)
        );
    }

    #[tokio::test]
    async fn invalid_poa() {
        let (issuer, trust_anchor, issuer_identifier, wua_issuer_privkey) = setup_simple_mock_issuer();
        let message_client = VcMessageClientStub {
            invalidate_poa: true,
            ..VcMessageClientStub::new(issuer)
        };

        let result = start_and_accept_err(message_client, issuer_identifier, trust_anchor, wua_issuer_privkey).await;
        assert_matches!(
            result,
            WalletIssuanceError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidProof)
        );
    }

    #[tokio::test]
    async fn no_poa() {
        let (issuer, trust_anchor, issuer_identifier, wua_issuer_privkey) = setup_simple_mock_issuer();
        let message_client = VcMessageClientStub {
            strip_poa: true,
            ..VcMessageClientStub::new(issuer)
        };

        let result = start_and_accept_err(message_client, issuer_identifier, trust_anchor, wua_issuer_privkey).await;
        assert_matches!(
            result,
            WalletIssuanceError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidCredentialRequest)
        );
    }

    #[tokio::test]
    async fn no_wua() {
        let (issuer, trust_anchor, issuer_identifier, wua_issuer_privkey) = setup_simple_mock_issuer();
        let message_client = VcMessageClientStub {
            strip_wua: true,
            ..VcMessageClientStub::new(issuer)
        };

        let result = start_and_accept_err(message_client, issuer_identifier, trust_anchor, wua_issuer_privkey).await;
        assert_matches!(
            result,
            WalletIssuanceError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidCredentialRequest)
        );
    }

    #[tokio::test]
    async fn test_cleanup_task() {
        let documents = mock_issuable_documents(NonZeroUsize::MIN);

        let (sessions, mock_time) = memory_session_store_with_mock_time();
        let sessions = Arc::new(sessions);

        let (issuer, _, _) = setup_mock_issuer(
            "https://example.com/".parse().unwrap(),
            MockAttrService {
                documents: documents.clone(),
            },
            NonZeroUsize::MIN,
            sessions.clone(),
            None,
        );

        let token = issuer.new_session(documents).await.unwrap();
        test_memory_store_with_cleanup_task(sessions, token, &mock_time, CLEANUP_INTERVAL).await;
    }
}
