use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::Infallible;
use std::error::Error as StdError;
use std::num::NonZeroU8;
use std::num::NonZeroUsize;
use std::ops::Add;
use std::sync::Arc;
use std::time::Duration;

use attestation_data::attributes::AttributesError;
use attestation_data::credential_payload::CredentialPayload;
use attestation_data::credential_payload::CredentialPayloadError;
use attestation_data::credential_payload::MdocCredentialPayloadError;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::credential_payload::SdJwtCredentialPayloadError;
use attestation_types::status_claim::StatusClaim;
use chrono::DateTime;
use chrono::DurationRound;
use chrono::Utc;
use crypto::EcdsaKeySend;
use crypto::trust_anchor::BorrowingTrustAnchor;
use crypto::utils::random_string;
use derive_more::AsRef;
use derive_more::Debug;
use derive_more::From;
use derive_more::Into;
use futures::TryFutureExt;
use futures::future::try_join_all;
use futures::join;
use http_utils::urls::BaseUrl;
use itertools::Itertools;
use jwt::Algorithm;
use jwt::Validation;
use jwt::error::JwkConversionError;
use jwt::error::JwtError;
use jwt::nonce::Nonce;
use jwt::wia::WiaDisclosure;
use jwt::wia::WiaError;
use p256::ecdsa::VerifyingKey;
use reqwest::Method;
use serde::Deserialize;
use serde::Serialize;
use token_status_list::status_list_service::StatusListServices;
use tokio::task::AbortHandle;
use tracing::info;
use tracing::warn;
use url::Url;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use uuid::Uuid;

use crate::Format;
use crate::authorization::OidcAuthorizationRequest;
use crate::authorization::PkceCodeChallenge;
use crate::authorization::PushedAuthorizationResponse;
use crate::authorization::VciAuthorizationRequest;
use crate::credential::Credential;
use crate::credential::CredentialRequest;
use crate::credential::CredentialRequestProof;
use crate::credential::CredentialRequests;
use crate::credential::CredentialResponse;
use crate::credential::CredentialResponses;
use crate::credential_configurations::CredentialConfiguration;
use crate::credential_configurations::CredentialConfigurationParameters;
use crate::credential_configurations::CredentialConfigurations;
use crate::credential_configurations::CredentialConfigurationsError;
use crate::dpop::Dpop;
use crate::dpop::DpopError;
use crate::issuable_document::IssuableDocument;
use crate::issuer_identifier::IssuerIdentifier;
use crate::metadata::issuer_metadata::AtLeastTwoU64;
use crate::metadata::issuer_metadata::BatchCredentialIssuance;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::metadata::issuer_metadata::IssuerMetadata;
use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
use crate::nonce::store::NonceStatus;
use crate::nonce::store::NonceStore;
use crate::nonce::store::NonceStoreError;
use crate::par;
use crate::par::PAR_TTL;
use crate::par::ParStore;
use crate::pkce::PkcePair;
use crate::pkce::S256PkcePair;
use crate::pkce::store::PKCE_FLOW_TTL;
use crate::pkce::store::PkceFlowStore;
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
    SessionStore(#[source] SessionStoreError),

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
    AttributesError(#[source] AttributesError),

    #[error("credential type in format \"{0}\" not offered: {1}")]
    CredentialTypeNotOffered(Format, String),

    #[error("missing code_verifier")]
    MissingCodeVerifier,

    #[error("consuming PKCE bridge entry failed: {0}")]
    PkceStore(#[source] Box<dyn StdError + Send + Sync + 'static>),

    #[error("PKCE verification failed")]
    PkceVerificationFailed,
}

/// Errors that can occur during processing of a Pushed Authorization Request.
#[derive(Debug, thiserror::Error)]
pub enum ParError {
    #[error("unknown client_id: {0}")]
    InvalidClient(String),

    #[error("storing PAR request failed: {0}")]
    Store(#[source] Box<dyn StdError + Send + Sync + 'static>),
}

/// Errors that can occur during processing of an authorization request.
#[derive(Debug, thiserror::Error)]
pub enum AuthorizeError {
    #[error("unknown client_id: {0}")]
    InvalidClient(String),

    #[error("no upstream authorization adapter configured")]
    NoUpstreamAdapter,

    #[error("request_uri not found or expired: {0}")]
    UnknownRequestUri(String),

    #[error("only S256 code_challenge_method is supported")]
    UnsupportedCodeChallenge,

    #[error("consuming PAR request failed: {0}")]
    ParStore(#[source] Box<dyn StdError + Send + Sync + 'static>),

    #[error("storing PKCE bridge entry failed: {0}")]
    PkceStore(#[source] Box<dyn StdError + Send + Sync + 'static>),

    #[error("adapting authorization request for upstream failed: {0}")]
    UpstreamResolve(#[source] UpstreamResolveError),

    #[error("encoding authorization request as query string failed: {0}")]
    Encode(#[source] serde_urlencoded::ser::Error),
}

/// Error returned by [`UpstreamAuthorizationAdapter::adapt`].
#[derive(Debug, thiserror::Error)]
pub enum UpstreamResolveError {
    #[error("upstream metadata discovery failed: {0}")]
    Discovery(Box<dyn StdError + Send + Sync>),

    #[error("upstream metadata has no authorization_endpoint")]
    NoAuthorizationEndpoint,
}

/// Adapts the wallet's authorization request to what the upstream OIDC provider expects.
///
/// The implementer resolves the upstream authorization endpoint (e.g. via OIDC discovery)
/// and rewrites the request.
#[trait_variant::make(Send)]
pub trait UpstreamAuthorizationAdapter {
    async fn adapt(
        &self,
        request: VciAuthorizationRequest,
    ) -> Result<(Url, OidcAuthorizationRequest), UpstreamResolveError>;
}

/// No-op [`UpstreamAuthorizationAdapter`] used as the default for [`Issuer`]'s `UAA` type
/// parameter when the issuer is constructed without an upstream adapter. The field is always
/// `None` in that case, so this method is unreachable.
impl UpstreamAuthorizationAdapter for () {
    async fn adapt(&self, _: VciAuthorizationRequest) -> Result<(Url, OidcAuthorizationRequest), UpstreamResolveError> {
        unimplemented!("() UpstreamAuthorizationAdapter does not adapt authorization requests")
    }
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

    #[error("invalid nonce used in credential request proof, WIA or PoA")]
    InvalidNonce,

    #[error("JWT error: {0}")]
    Jwt(#[from] JwtError),

    #[error("missing credential configuration with identifier: {0}")]
    MissingCredentialConfiguration(CredentialConfigurationId),

    #[error("mismatch between requested: {requested} and offered attestation types: {offered}")]
    CredentialTypeMismatch { requested: Format, offered: Format },

    #[error("wrong number of credential requests")]
    WrongNumberOfCredentialRequests,

    #[error("missing credential request proof of possession")]
    MissingCredentialRequestPoP,

    #[error("missing WIA")]
    MissingWia,

    #[error("error converting PreviewableCredentialPayload to CredentialPayload: {0}")]
    PreviewConversion(#[from] CredentialPayloadError),

    #[error("error converting CredentialPayload to Mdoc: {0}")]
    MdocConversion(#[from] MdocCredentialPayloadError),

    #[error("error converting CredentialPayload to SD-JWT: {0}")]
    SdJwtConversion(#[from] SdJwtCredentialPayloadError),

    #[error("error verifying WIA: {0}")]
    Wia(#[from] WiaError),

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

    #[error("missing credential configuration with identifier: {0}")]
    MissingCredentialConfiguration(CredentialConfigurationId),

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
    pub credential_previews: VecNonEmpty<CredentialPreviewState>,
    pub dpop_public_key: VerifyingKey,
    pub dpop_nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialPreviewState {
    pub credential_configuration_id: CredentialConfigurationId,
    pub format: Format,
    pub batch_size: NonZeroU8,
    pub credential_payload: PreviewableCredentialPayload,
    pub batch_id: Uuid,
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

/// A PKCE `code_verifier` that the `/token` handler decoupled from the wallet's pair, forwarded
/// through the [`Issuer`] to the [`AttributeService`] for use against the upstream OIDC provider.
///
/// The [`Issuer`] never inspects this value — it is an opaque pass-through.
#[derive(Debug, Clone, From, AsRef, Into)]
pub struct UpstreamCodeVerifier(String);

/// Implementations of this trait are responsible for determining the attributes to be issued, given the session and
/// the token request. See for example the [`BrpPidAttributeService`].
#[trait_variant::make(Send)]
pub trait AttributeService {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn attributes(
        &self,
        token_request: TokenRequest,
        upstream_code_verifier: Option<UpstreamCodeVerifier>,
    ) -> Result<VecNonEmpty<IssuableDocument>, Self::Error>;
}

impl AttributeService for () {
    type Error = Infallible;

    async fn attributes(
        &self,
        _: TokenRequest,
        _: Option<UpstreamCodeVerifier>,
    ) -> Result<VecNonEmpty<IssuableDocument>, Infallible> {
        unimplemented!("() AttributeService does not provide attributes")
    }
}

pub struct Issuer<K, A, S, N, L, PAS = (), PKS = (), UAA = ()> {
    attr_service: A,
    issuer_data: IssuerData<K>,
    sessions: Arc<S>,
    proof_nonce_store: Arc<N>,
    status_list_services: Arc<L>,
    par_store: Arc<PAS>,
    pkce_flow_store: Arc<PKS>,
    upstream_authorization_adapter: Option<UAA>,
    cleanup_task: AbortHandle,
}

/// Fields of the [`Issuer`] needed by the issuance functions.
pub struct IssuerData<K> {
    credential_configs: CredentialConfigurations<K>,
    wia_config: Option<WiaConfig>,

    /// Wallet IDs accepted by this server, MUST be used by the wallet as `iss` in its PoP JWTs.
    accepted_wallet_client_ids: Vec<String>,

    /// URL prefix of the `/token`, `/credential` and `/batch_crededential` endpoints.
    server_url: BaseUrl,

    /// The maximum amount of copies of a credential that the holder can request.
    batch_size: NonZeroU8,

    metadata: IssuerMetadata,
}

impl<K> IssuerData<K> {
    fn credential_configuration_for_preview_state(
        &self,
        preview_state: &CredentialPreviewState,
    ) -> Option<&CredentialConfiguration<K>> {
        self.credential_configs
            .get_by_configuration_id(&preview_state.credential_configuration_id)
            .and_then(|config| {
                // Do a sanity check to see if the credential configuration has changed from the stored preview state.
                (preview_state.format == config.format
                    && preview_state.credential_payload.attestation_type == config.attestation_type)
                    .then_some(config)
            })
    }
}

pub struct WiaConfig {
    /// Public key of the WIA issuer.
    pub wia_trust_anchors: Vec<BorrowingTrustAnchor>,
}

impl<K, A, S, N, L, PAS, PKS, UAA> Drop for Issuer<K, A, S, N, L, PAS, PKS, UAA> {
    fn drop(&mut self) {
        // Stop the tasks at the next .await
        self.cleanup_task.abort();
    }
}

impl<K, A, S, N, L, PAS, PKS, UAA> Issuer<K, A, S, N, L, PAS, PKS, UAA> {
    pub fn credential_config_id_by_format_and_attestation_type(
        &self,
        format: Format,
        attestation_type: &str,
    ) -> Option<&CredentialConfigurationId> {
        self.issuer_data
            .credential_configs
            .get_by_format_and_attestation_type(format, attestation_type)
            .map(|(config_id, _config)| config_id)
    }

    pub fn metadata(&self) -> &IssuerMetadata {
        &self.issuer_data.metadata
    }
}

impl<K, A, S, N, L, PAS, PKS, UAA> Issuer<K, A, S, N, L, PAS, PKS, UAA>
where
    S: SessionStore<IssuanceData> + Sync + 'static,
    N: NonceStore + Sync + 'static,
{
    #[expect(clippy::too_many_arguments, reason = "Constructor")]
    pub fn try_new(
        issuer_identifier: IssuerIdentifier,
        batch_size: NonZeroU8,
        wallet_client_ids: Vec<String>,
        credential_config_params: HashMap<CredentialConfigurationId, CredentialConfigurationParameters<K>>,
        wia_config: Option<WiaConfig>,
        attr_service: A,
        sessions: Arc<S>,
        proof_nonce_store: N,
        status_list_services: Arc<L>,
        par_store: Arc<PAS>,
        pkce_flow_store: Arc<PKS>,
        upstream_authorization_adapter: Option<UAA>,
    ) -> Result<Self, CredentialConfigurationsError> {
        let credential_configs = CredentialConfigurations::try_new(credential_config_params)?;

        let server_url = issuer_identifier.join_issuer_url("/issuance");
        let credential_endpoint = server_url.join_issuer_url("/credential");
        let batch_credential_endpoint = server_url.join_issuer_url("/batch_credential");
        let nonce_endpoint = server_url.join_issuer_url("/nonce");
        let credential_preview_endpoint = server_url.join_issuer_url("/credential_preview");

        let batch_credential_issuance = AtLeastTwoU64::try_new(batch_size.into())
            .ok()
            .map(|batch_size| BatchCredentialIssuance { batch_size });
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
            batch_credential_issuance,
            display: None,
            credential_configurations_supported: credential_configs.to_credential_configurations_supported(),
            credential_preview_endpoint: Some(credential_preview_endpoint),
        };

        let issuer_data = IssuerData {
            credential_configs,
            accepted_wallet_client_ids: wallet_client_ids,
            wia_config,

            // In this implementation, the public server URL is composed of the
            // Credential Issuer Identifier appended with the "/issuance/" path.
            server_url: server_url.into_inner(),
            batch_size,
            metadata,
        };

        let proof_nonce_store = Arc::new(proof_nonce_store);

        let task_sessions = Arc::clone(&sessions);
        let task_nonce_store = Arc::clone(&proof_nonce_store);
        let cleanup_task = start_recurring_task(CLEANUP_INTERVAL, move || {
            let task_sessions = Arc::clone(&task_sessions);
            let task_nonce_store = Arc::clone(&task_nonce_store);

            async move {
                let _ = join!(
                    task_sessions.cleanup().inspect_err(|error| {
                        warn!("error during session cleanup: {error}");
                    }),
                    task_nonce_store.remove_expired_nonces().inspect_err(|error| {
                        warn!("error during proof nonce cleanup: {error}");
                    })
                );
            }
        });

        let issuer = Self {
            issuer_data,
            attr_service,
            sessions,
            proof_nonce_store,
            status_list_services,
            par_store,
            pkce_flow_store,
            upstream_authorization_adapter,
            cleanup_task,
        };

        Ok(issuer)
    }
}

fn logged_issuance_result<T, E: std::error::Error>(result: Result<T, E>) -> Result<T, E> {
    result
        .inspect(|_| info!("Issuance success"))
        .inspect_err(|error| info!("Issuance error: {error}"))
}

impl<K, A, S, N, L, PAS, PKS, UAA> Issuer<K, A, S, N, L, PAS, PKS, UAA>
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

impl<K, A, S, N, L, PAS, PKS, UAA> Issuer<K, A, S, N, L, PAS, PKS, UAA>
where
    N: NonceStore,
{
    pub async fn generate_proof_nonce(&self) -> Result<Nonce, NonceStoreError<N::Error>> {
        let nonce = Nonce::new_random();

        self.proof_nonce_store.store_nonce(nonce.clone()).await?;

        Ok(nonce)
    }
}

impl<K, A, S, N, L, PAS, PKS, UAA> Issuer<K, A, S, N, L, PAS, PKS, UAA>
where
    A: AttributeService,
{
    pub fn oauth_metadata(&self) -> AuthorizationServerMetadata {
        let issuer_url = self.issuer_data.metadata.credential_issuer.as_base_url();

        AuthorizationServerMetadata {
            authorization_endpoint: Some(issuer_url.join("/issuance/authorize")),
            pushed_authorization_request_endpoint: Some(issuer_url.join("/issuance/par")),
            require_pushed_authorization_requests: true,
            ..AuthorizationServerMetadata::new(
                self.issuer_data.metadata.credential_issuer.clone(),
                issuer_url.join("issuance/token"),
            )
        }
    }
}

impl<K, A, S, N, L, PAS, PKS, UAA> Issuer<K, A, S, N, L, PAS, PKS, UAA>
where
    PAS: ParStore,
{
    pub async fn process_pushed_authorization_request(
        &self,
        request: VciAuthorizationRequest,
    ) -> Result<PushedAuthorizationResponse, ParError> {
        if !self
            .issuer_data
            .accepted_wallet_client_ids
            .contains(&request.oauth_request.client_id)
        {
            return Err(ParError::InvalidClient(request.oauth_request.client_id));
        }

        let request_uri = par::generate_request_uri();
        let expires_at = Utc::now() + PAR_TTL;

        self.par_store
            .store(request_uri.clone(), request, expires_at)
            .await
            .map_err(|error| ParError::Store(Box::new(error)))?;

        Ok(PushedAuthorizationResponse {
            request_uri,
            expires_in: PAR_TTL,
        })
    }
}

impl<K, A, S, N, L, PAS, PKS, UAA> Issuer<K, A, S, N, L, PAS, PKS, UAA>
where
    PAS: ParStore,
    PKS: PkceFlowStore,
    UAA: UpstreamAuthorizationAdapter,
{
    /// Consume the PAR, swap the wallet's PKCE challenge for an upstream one (storing the upstream
    /// verifier under the wallet's challenge for the matching `/token` call), dispatch via the
    /// configured [`UpstreamAuthorizationAdapter`], and return the URL the wallet should be
    /// redirected to.
    pub async fn process_authorize(&self, request_uri: &str, client_id: &str) -> Result<Url, AuthorizeError> {
        if !self
            .issuer_data
            .accepted_wallet_client_ids
            .iter()
            .any(|id| id == client_id)
        {
            return Err(AuthorizeError::InvalidClient(client_id.to_string()));
        }

        let upstream_authorization_adapter = self
            .upstream_authorization_adapter
            .as_ref()
            .ok_or(AuthorizeError::NoUpstreamAdapter)?;

        let mut authorization_request = self
            .par_store
            .consume(request_uri)
            .await
            .map_err(|error| AuthorizeError::ParStore(Box::new(error)))?
            .ok_or_else(|| AuthorizeError::UnknownRequestUri(request_uri.to_string()))?;

        // Bridge PKCE: generate a new PKCE pair for the upstream server, substitute the wallet's challenge with the
        // upstream challenge, and store the upstream verifier keyed by the wallet's challenge for the matching
        // /token call.
        {
            let wallet_code_challenge = match &authorization_request.code_challenge {
                PkceCodeChallenge::S256 { code_challenge } => code_challenge.clone(),
                PkceCodeChallenge::Plain { .. } => return Err(AuthorizeError::UnsupportedCodeChallenge),
            };

            let upstream_pkce = S256PkcePair::generate();
            authorization_request.code_challenge = PkceCodeChallenge::S256 {
                code_challenge: upstream_pkce.code_challenge().to_string(),
            };

            self.pkce_flow_store
                .store(
                    wallet_code_challenge,
                    upstream_pkce.into_code_verifier(),
                    Utc::now() + PKCE_FLOW_TTL,
                )
                .await
                .map_err(|error| AuthorizeError::PkceStore(Box::new(error)))?;
        }

        let (authorization_endpoint, authorization_request) = upstream_authorization_adapter
            .adapt(authorization_request)
            .await
            .map_err(AuthorizeError::UpstreamResolve)?;

        let query_string = serde_urlencoded::to_string(&authorization_request).map_err(AuthorizeError::Encode)?;

        let mut redirect_url = authorization_endpoint;
        redirect_url.set_query(Some(&query_string));

        Ok(redirect_url)
    }
}

impl<K, A, S, N, L, PAS, PKS, UAA> Issuer<K, A, S, N, L, PAS, PKS, UAA>
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
                let requested_configuration_ids = credential_configuration_ids.iter().collect::<HashSet<_>>();

                // Return previews only for the types that are actually in the session; silently ignore IDs that appear
                // in the requested_attestation_types but are not part of this session.
                session_data
                    .credential_previews
                    .iter()
                    .filter(|preview_state| {
                        requested_configuration_ids.contains(&preview_state.credential_configuration_id)
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
        let credential_config = self
            .issuer_data
            .credential_configuration_for_preview_state(state)
            .ok_or_else(|| {
                CredentialPreviewError::MissingCredentialConfiguration(state.credential_configuration_id.clone())
            })?;

        let preview = CredentialPreview {
            content: CredentialPreviewContent {
                format: state.format,
                batch_size: state.batch_size,
                credential_payload: state.credential_payload.clone(),
                issuer_certificate: credential_config.key_pair.certificate().clone(),
            },
            type_metadata: credential_config.metadata.documents().clone().into(),
        };

        Ok(preview)
    }
}

impl<K, A, S, N, L, PAS, PKS, UAA> Issuer<K, A, S, N, L, PAS, PKS, UAA>
where
    K: EcdsaKeySend,
    A: AttributeService,
    S: SessionStore<IssuanceData>,
    PKS: PkceFlowStore,
{
    /// Process a token request, performing the wallet ↔ upstream PKCE bridge consumption when the
    /// grant type is `authorization_code`. Pre-authorized-code grants bypass PKCE entirely.
    pub async fn process_token_request(
        &self,
        token_request: TokenRequest,
        dpop: Dpop,
    ) -> Result<(TokenResponse, String), TokenRequestError> {
        let upstream_code_verifier = match &token_request.grant_type {
            TokenRequestGrantType::AuthorizationCode { .. } => {
                let wallet_code_verifier = token_request
                    .code_verifier
                    .as_ref()
                    .ok_or(TokenRequestError::MissingCodeVerifier)?;
                let wallet_code_challenge = S256PkcePair::challenge_for(wallet_code_verifier);

                let verifier = self
                    .pkce_flow_store
                    .consume(&wallet_code_challenge)
                    .await
                    .map_err(|error| TokenRequestError::PkceStore(Box::new(error)))?
                    .map(UpstreamCodeVerifier::from)
                    .ok_or(TokenRequestError::PkceVerificationFailed)?;

                Some(verifier)
            }
            TokenRequestGrantType::PreAuthorizedCode { .. } => None,
        };

        self.process_token_request_with_verifier(token_request, dpop, upstream_code_verifier)
            .await
    }
}

impl<K, A, S, N, L, PAS, PKS, UAA> Issuer<K, A, S, N, L, PAS, PKS, UAA>
where
    K: EcdsaKeySend,
    A: AttributeService,
    S: SessionStore<IssuanceData>,
{
    /// Process a token request with the upstream code verifier supplied explicitly, bypassing the
    /// PKCE bridge.
    ///
    /// Production code should call [`Self::process_token_request`] instead; this entry point exists
    /// for tests that drive the issuer without an authorization flow having occurred.
    async fn process_token_request_with_verifier(
        &self,
        token_request: TokenRequest,
        dpop: Dpop,
        upstream_code_verifier: Option<UpstreamCodeVerifier>,
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
                &self.issuer_data.credential_configs,
                self.issuer_data.batch_size,
                is_new_session,
                upstream_code_verifier,
            )
            .await;

        let (response, next) = match result {
            Ok((response, dpop_nonce, next)) => (Ok((response, dpop_nonce)), next.into()),
            Err((err, next)) => (Err(err), next.into()),
        };

        self.sessions
            .write(next, is_new_session)
            .await
            .map_err(|e| TokenRequestError::IssuanceError(IssuanceError::SessionStore(e)))?;

        response
    }
}

impl<K, A, S, N, L, PAS, PKS, UAA> Issuer<K, A, S, N, L, PAS, PKS, UAA>
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
                    proof_nonce_store: self.proof_nonce_store.as_ref(),
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
                    proof_nonce_store: self.proof_nonce_store.as_ref(),
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

impl<K, A, S, N, L, PAS, PKS, UAA> Issuer<K, A, S, N, L, PAS, PKS, UAA>
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
    async fn process_token_request<K>(
        self,
        token_request: TokenRequest,
        accepted_wallet_client_ids: &[String],
        dpop: Dpop,
        attr_service: &impl AttributeService,
        server_url: &BaseUrl,
        credential_configurations: &CredentialConfigurations<K>,
        batch_size: NonZeroU8,
        is_new_session: bool,
        upstream_code_verifier: Option<UpstreamCodeVerifier>,
    ) -> Result<(TokenResponse, String, Session<WaitingForResponse>), (TokenRequestError, Session<Done>)> {
        let result = self
            .process_token_request_inner(
                token_request,
                dpop,
                attr_service,
                server_url,
                credential_configurations,
                batch_size,
                is_new_session,
                upstream_code_verifier,
            )
            .await;

        match result {
            Ok((token_response, credential_previews, dpop_pubkey, dpop_nonce)) => {
                let next = self.transition(WaitingForResponse {
                    access_token: token_response.access_token.clone(),
                    accepted_wallet_client_ids: accepted_wallet_client_ids.to_vec(),
                    credential_previews,
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

    fn credential_preview_state_for_issuable_document<K>(
        credential_configurations: &CredentialConfigurations<K>,
        document: IssuableDocument,
        batch_size: NonZeroU8,
    ) -> Result<CredentialPreviewState, TokenRequestError> {
        let format = document.format;
        let (credential_config_id, credential_config) = credential_configurations
            .get_by_format_and_attestation_type(format, &document.attestation_type)
            .ok_or_else(|| TokenRequestError::CredentialTypeNotOffered(format, document.attestation_type.clone()))?;

        document
            .validate_with_metadata(credential_config.metadata.normalized())
            .map_err(TokenRequestError::AttributesError)?;

        // Truncate the current time to only include the date part, so that all issued credentials on a single
        // day have the same `nbf` and `exp` field
        let now = utc_now_truncated_to_days();
        let valid_until = now.add(credential_config.valid_days);

        let (batch_id, credential_payload) = document.into_id_and_previewable_credential_payload(
            now,
            valid_until,
            credential_config.issuer_uri.clone(),
            credential_config.attestation_qualification,
        );

        let state = CredentialPreviewState {
            credential_configuration_id: credential_config_id.clone(),
            format,
            batch_size,
            credential_payload,
            batch_id,
        };

        Ok(state)
    }

    #[expect(clippy::too_many_arguments, reason = "Cascading effect because of constructor")]
    async fn process_token_request_inner<K>(
        &self,
        token_request: TokenRequest,
        dpop: Dpop,
        attr_service: &impl AttributeService,
        server_url: &BaseUrl,
        credential_configurations: &CredentialConfigurations<K>,
        batch_size: NonZeroU8,
        is_new_session: bool,
        upstream_code_verifier: Option<UpstreamCodeVerifier>,
    ) -> Result<(TokenResponse, VecNonEmpty<CredentialPreviewState>, VerifyingKey, String), TokenRequestError> {
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
                .attributes(token_request, upstream_code_verifier)
                .await
                .map_err(|e| TokenRequestError::AttributeService(Box::new(e)))?,
        };

        let preview_states = issuables
            .into_nonempty_iter()
            .map(|document| {
                Self::credential_preview_state_for_issuable_document(credential_configurations, document, batch_size)
            })
            .collect::<Result<VecNonEmpty<_>, TokenRequestError>>()?;

        let dpop_nonce = random_string(32);

        let token_response = TokenResponse::new(AccessToken::new(&code));

        Ok((token_response, preview_states, dpop_public_key, dpop_nonce))
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

    fn verify_wia<K>(
        &self,
        attestations: Option<&WiaDisclosure>,
        issuer_data: &IssuerData<K>,
    ) -> Result<Option<Nonce>, CredentialRequestError> {
        let issuer_identifier = issuer_data.metadata.credential_issuer.as_ref();

        issuer_data
            .wia_config
            .as_ref()
            .map(|wia_config| {
                let wia_disclosure = attestations.ok_or(CredentialRequestError::MissingWia)?;

                let (_, wia_nonce) = wia_disclosure.verify(
                    &wia_config.wia_trust_anchors,
                    issuer_identifier,
                    &self.state.data.accepted_wallet_client_ids,
                )?;

                Ok::<_, CredentialRequestError>(wia_nonce)
            })
            .transpose()
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
            .filter(|preview| preview.format == requested_format)
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

        let wia_nonce = self.verify_wia(credential_request.attestations.as_ref(), issuer_data)?;

        // Check the validity of all of the nonces used, which may be equal to each other.
        let nonce_status = services
            .proof_nonce_store
            .check_nonce_status_and_remove([&request_nonce].iter().copied().chain(wia_nonce.as_ref()))
            .await
            .map_err(|error| CredentialRequestError::ProofNonceStore(Box::new(error)))?;

        if !matches!(nonce_status, NonceStatus::AllValid) {
            return Err(CredentialRequestError::InvalidNonce);
        }

        let credential_config = issuer_data
            .credential_configuration_for_preview_state(preview)
            .ok_or_else(|| {
                CredentialRequestError::MissingCredentialConfiguration(preview.credential_configuration_id.clone())
            })?;

        let status_claim = services
            .status_list_services
            .obtain_status_claims(
                &credential_config.status_list_group,
                preview.batch_id,
                preview.credential_payload.expires,
                NonZeroUsize::MIN,
            )
            .await
            .map_err(|err| CredentialRequestError::ObtainStatusClaim(Box::new(err)))?
            .into_first();

        let credential_response = CredentialResponse::new(
            requested_format,
            preview.credential_payload.clone(),
            utc_now_truncated_to_days(),
            &holder_pubkey,
            credential_config,
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
                // For every preview collect for every copy the verified key
                let format_pubkeys: VecNonEmpty<_> = (0..preview.batch_size.get())
                    .map(|_| {
                        let cred_req = credential_requests
                            .credential_requests
                            .as_ref()
                            .get(request_nonces.len())
                            .ok_or(CredentialRequestError::WrongNumberOfCredentialRequests)?;

                        // Verify the assumption that the order of the incoming requests matches exactly
                        // that of the flattened batch_size by matching the requested format.
                        if preview.format != cred_req.credential_type.as_ref().format() {
                            return Err(CredentialRequestError::CredentialTypeMismatch {
                                offered: preview.format,
                                requested: cred_req.credential_type.as_ref().format(),
                            });
                        }

                        let (key, nonce) = cred_req.verify(
                            &issuer_data.accepted_wallet_client_ids,
                            &issuer_data.metadata.credential_issuer,
                        )?;

                        request_nonces.push(nonce);

                        Ok(key)
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .unwrap(); // ok because `batch_size` has a `NonZeroU8` value in `CredentialPreviewContent`.

                let credential_config = issuer_data
                    .credential_configuration_for_preview_state(preview)
                    .ok_or_else(|| {
                        CredentialRequestError::MissingCredentialConfiguration(
                            preview.credential_configuration_id.clone(),
                        )
                    })?;

                Ok((preview, credential_config, format_pubkeys))
            })
            .collect::<Result<Vec<_>, CredentialRequestError>>()?;

        // Verify that we have consumed all credential requests
        if request_nonces.len() != credential_requests.credential_requests.as_ref().len() {
            return Err(CredentialRequestError::WrongNumberOfCredentialRequests);
        }

        let wia_nonce = self.verify_wia(credential_requests.attestations.as_ref(), issuer_data)?;

        // Check the validity of all of the nonces used, which may be equal to each other.
        let nonce_status = services
            .proof_nonce_store
            .check_nonce_status_and_remove(request_nonces.iter().chain(wia_nonce.as_ref()))
            .await
            .map_err(|error| CredentialRequestError::ProofNonceStore(Box::new(error)))?;

        if !matches!(nonce_status, NonceStatus::AllValid) {
            return Err(CredentialRequestError::InvalidNonce);
        }

        // Obtain a status claim for every attestation copy, linked to a single batch id per preview
        let status_claims = try_join_all(previews_and_holder_pubkeys.iter().map(
            |(preview, credential_config, format_pubkeys)| async move {
                let claims = services
                    .status_list_services
                    .obtain_status_claims(
                        &credential_config.status_list_group,
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
                .flat_map(|((preview, credential_config, format_pubkeys), claims)| {
                    format_pubkeys.into_iter().zip(claims.into_inner()).map(|(key, claim)| {
                        CredentialResponse::new(
                            preview.format,
                            preview.credential_payload.clone(),
                            issued_at,
                            key,
                            credential_config,
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
        credential_config: &CredentialConfiguration<impl EcdsaKeySend>,
        status_claim: StatusClaim,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        let payload = CredentialPayload::from_previewable_credential_payload(
            preview_credential_payload,
            issued_at,
            holder_pubkey,
            credential_config.metadata.normalized(),
            credential_config.metadata.first_document_integrity().clone(),
            status_claim,
        )?;

        match credential_format {
            Format::MsoMdoc => Self::new_for_mdoc(payload, credential_config).await,
            Format::SdJwt => Self::new_for_sd_jwt(payload, credential_config).await,
            other => Err(CredentialRequestError::CredentialTypeNotOffered(other.to_string())),
        }
    }

    async fn new_for_mdoc(
        credential_payload: CredentialPayload,
        credential_config: &CredentialConfiguration<impl EcdsaKeySend + Sized>,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        // Construct an mdoc `IssuerSigned` from the contents of `PreviewableCredentialPayload`
        // and the attestation config by signing it.
        let (issuer_signed, _) = credential_payload.into_signed_mdoc(&credential_config.key_pair).await?;

        Ok(CredentialResponse::new_immediate(Credential::new_mdoc(issuer_signed)))
    }

    async fn new_for_sd_jwt(
        credential_payload: CredentialPayload,
        credential_config: &CredentialConfiguration<impl EcdsaKeySend + Sized>,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        let signed_sd_jwt = credential_payload
            .into_signed_sd_jwt(credential_config.metadata.normalized(), &credential_config.key_pair)
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
    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::x509::generate::mock::generate_issuer_mock_with_registration;
    use attestation_types::qualification::AttestationQualification;
    use chrono::Days;
    use chrono::Timelike;
    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::BorrowingTrustAnchor;
    use derive_more::Debug;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use thiserror::Error;
    use tracing_test::traced_test;
    use url::Url;
    use wscd::mock_remote::MockRemoteWscd;

    use super::*;
    use crate::CredentialErrorCode;
    use crate::credential::CredentialRequest;
    use crate::credential::CredentialRequestProof;
    use crate::credential::CredentialRequests;
    use crate::credential::CredentialResponse;
    use crate::credential::CredentialResponses;
    use crate::credential_configurations::CredentialConfigurationParameters;
    use crate::dpop::Dpop;
    use crate::issuable_document::IssuableDocument;
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

    #[derive(Debug, Error, Clone, Eq, PartialEq)]
    #[error("MyError")]
    struct MyError;

    #[test]
    fn test_credential_preview_from_issuable_document() {
        let document = IssuableDocument::new_mock_degree("Education".to_string());

        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuance_keypair = generate_issuer_mock_with_registration(&ca, IssuerRegistration::new_mock()).unwrap();
        let config_params = CredentialConfigurationParameters {
            format: document.format,
            attestation_type: document.attestation_type.clone(),
            key_pair: KeyPair::new_from_signing_key(
                issuance_keypair.private_key().to_owned(),
                issuance_keypair.certificate().to_owned(),
            )
            .unwrap(),
            status_list_group: "status_list_group".to_string(),
            valid_days: Days::new(1),
            issuer_uri: "https://example.com".parse().unwrap(),
            attestation_qualification: AttestationQualification::default(),
            metadata_documents: TypeMetadataDocuments::degree_example().1,
        };
        let credential_configs =
            CredentialConfigurations::try_new([("credential_config_id".to_string().into(), config_params)].into())
                .unwrap();

        let CredentialPreviewState { credential_payload, .. } =
            Session::<Created>::credential_preview_state_for_issuable_document(
                &credential_configs,
                document,
                NonZeroU8::MIN,
            )
            .expect("creating credential preview for issuable document should succeed");

        assert_eq!(credential_payload.not_before.unwrap().as_ref().second(), 0);
        assert_eq!(credential_payload.not_before.unwrap().as_ref().minute(), 0);
        assert_eq!(credential_payload.not_before.unwrap().as_ref().hour(), 0);
        assert_eq!(credential_payload.expires.unwrap().as_ref().second(), 0);
        assert_eq!(credential_payload.expires.unwrap().as_ref().minute(), 0);
        assert_eq!(credential_payload.expires.unwrap().as_ref().hour(), 0);
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

    fn setup_simple_mock_issuer() -> (MockIssuer, BorrowingTrustAnchor, IssuerIdentifier, KeyPair) {
        let issuer_identifier: IssuerIdentifier = "https://example.com/".parse().unwrap();
        let (issuer, trust_anchor, wia_keypair) = setup_mock_issuer(
            issuer_identifier.clone(),
            MockAttrService {
                documents: mock_issuable_documents(NonZeroUsize::MIN),
            },
            NonZeroUsize::MIN,
            Arc::new(MemorySessionStore::default()),
            Arc::new(()),
            Arc::new(()),
            None,
        );
        (issuer, trust_anchor, issuer_identifier, wia_keypair)
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
        strip_wia: bool,
    }

    impl VcMessageClientStub {
        fn new(issuer: MockIssuer) -> Self {
            Self {
                issuer,
                wrong_access_token: false,
                invalidate_dpop: false,
                invalidate_pop: false,
                strip_wia: false,
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

            if self.strip_wia {
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

            if self.strip_wia {
                credential_requests.attestations.take();
            }

            credential_requests
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
                .process_token_request_with_verifier(token_request.clone(), dpop_header.clone(), None)
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
        trust_anchor: BorrowingTrustAnchor,
        wia_keypair: KeyPair,
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

        let wscd = MockRemoteWscd::new_with_wia_keypair(wia_keypair);
        session.accept_issuance(trust_anchors, &wscd, true).await.unwrap_err()
    }

    #[tokio::test]
    async fn wrong_access_token() {
        let (issuer, trust_anchor, issuer_identifier, wia_issuer_privkey) = setup_simple_mock_issuer();
        let message_client = VcMessageClientStub {
            wrong_access_token: true,
            ..VcMessageClientStub::new(issuer)
        };

        let result = start_and_accept_err(message_client, issuer_identifier, trust_anchor, wia_issuer_privkey).await;
        assert_matches!(
            result,
            WalletIssuanceError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidToken)
        );
    }

    #[tokio::test]
    async fn invalid_dpop() {
        let (issuer, trust_anchor, issuer_identifier, wia_issuer_privkey) = setup_simple_mock_issuer();
        let message_client = VcMessageClientStub {
            invalidate_dpop: true,
            ..VcMessageClientStub::new(issuer)
        };

        let result = start_and_accept_err(message_client, issuer_identifier, trust_anchor, wia_issuer_privkey).await;
        assert_matches!(
            result,
            WalletIssuanceError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidCredentialRequest)
        );
    }

    #[tokio::test]
    async fn invalid_pop() {
        let (issuer, trust_anchor, issuer_identifier, wia_issuer_privkey) = setup_simple_mock_issuer();
        let message_client = VcMessageClientStub {
            invalidate_pop: true,
            ..VcMessageClientStub::new(issuer)
        };

        let result = start_and_accept_err(message_client, issuer_identifier, trust_anchor, wia_issuer_privkey).await;
        assert_matches!(
            result,
            WalletIssuanceError::CredentialRequest(err) if matches!(err.error, CredentialErrorCode::InvalidProof)
        );
    }

    #[tokio::test]
    async fn no_wia() {
        let (issuer, trust_anchor, issuer_identifier, wia_issuer_privkey) = setup_simple_mock_issuer();
        let message_client = VcMessageClientStub {
            strip_wia: true,
            ..VcMessageClientStub::new(issuer)
        };

        let result = start_and_accept_err(message_client, issuer_identifier, trust_anchor, wia_issuer_privkey).await;
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
            Arc::new(()),
            Arc::new(()),
            None::<()>,
        );

        let token = issuer.new_session(documents).await.unwrap();
        test_memory_store_with_cleanup_task(sessions, token, &mock_time, CLEANUP_INTERVAL).await;
    }
}
