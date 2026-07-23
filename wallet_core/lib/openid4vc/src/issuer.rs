use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::num::NonZeroU8;
use std::num::NonZeroUsize;
use std::ops::Add;
use std::sync::Arc;
use std::time::Duration;

use attestation_data::attributes::AttributesError;
use attestation_data::credential_payload::CredentialPayload;
use attestation_data::credential_payload::CredentialPayloadIntoSignedMdocError;
use attestation_data::credential_payload::CredentialPayloadIntoSignedSdJwtError;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_types::credential_format::Format;
use attestation_types::credential_kind::CredentialKind;
use attestation_types::status_claim::StatusClaim;
use chrono::DateTime;
use chrono::DurationRound;
use chrono::Utc;
use crypto::EcdsaKey;
use crypto::EcdsaKeySend;
use crypto::server_keys::KeyPair;
use crypto::trust_anchor::TrustAnchors;
use crypto::utils::random_string;
use derive_more::Constructor;
use derive_more::Debug;
use futures::future::try_join_all;
use futures::join;
use http_utils::urls::BaseUrl;
use indexmap::IndexSet;
use itertools::Itertools;
use jwt::Algorithm;
use jwt::SignedJwt;
use jwt::Validation;
use jwt::error::JwkConversionError;
use jwt::error::JwtSignError;
use jwt::error::JwtVerifyError;
use jwt::headers::HeaderWithX5c;
use jwt::nonce::Nonce;
use jwt::wia::WIA_CLIENT_AUTH_METHOD;
use jwt::wia::WiaClaims;
use jwt::wia::WiaDisclosure;
use jwt::wia::WiaError;
use p256::ecdsa::VerifyingKey;
use reqwest::Method;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use serde::Deserialize;
use serde::Serialize;
use token_status_list::status_list_service::StatusListService;
use tokio::task::AbortHandle;
use tracing::info;
use url::Url;
use utils::generator::Generator;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use uuid::Uuid;

use crate::authorization_details::AuthorizationDetails;
use crate::cleanup::PeriodicCleanup;
use crate::cleanup::log_cleanup_error;
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
use crate::credential_offer::CredentialOffer;
use crate::dpop::Dpop;
use crate::dpop::DpopError;
use crate::issuable_document::IssuableDocument;
use crate::issuer_identifier::IssuerIdentifier;
use crate::jose::JwsAlgorithm;
use crate::metadata::issuer_metadata::AtLeastTwoU64;
use crate::metadata::issuer_metadata::BatchCredentialIssuance;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::metadata::issuer_metadata::IssuerEndpoints;
use crate::metadata::issuer_metadata::IssuerMetadata;
use crate::metadata::issuer_metadata::SignedIssuerMetadataPayload;
use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
use crate::nonce::store::NonceStatus;
use crate::nonce::store::NonceStore;
use crate::nonce::store::NonceStoreError;
use crate::pkce::S256PkcePair;
use crate::preview::CredentialPreviewResponse;
use crate::scope::Scope;
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
use crate::token::TokenRequest;
use crate::token::TokenRequestGrantType;
use crate::token::TokenResponse;

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

#[derive(Debug, thiserror::Error)]
pub enum IssuableDocumentError {
    #[error("credential type not offered in Credential Configurations: {0}")]
    CredentialTypeNotOffered(CredentialKind),

    #[error("attributes do not match type metadata: {0}")]
    AttributesError(#[source] AttributesError),
}

#[derive(Debug, thiserror::Error)]
pub enum PreAuthorizedSessionError {
    #[error("issuable document is not valid: {0}")]
    IssuableDocument(#[source] IssuableDocumentError),

    #[error("failed to store new session: {0}")]
    SessionStore(#[source] SessionStoreError),
}

/// Errors that can occur during processing of the token request.
#[derive(Debug, thiserror::Error)]
pub enum TokenRequestError {
    #[error("issuance error: {0}")]
    IssuanceError(#[from] IssuanceError),

    #[error("session not found for the supplied code")]
    SessionNotFound,

    #[error("error verifying WIA: {0}")]
    Wia(#[source] WiaError),

    #[error("unexpected grant type for this session: expected {expected}, got {actual}")]
    UnexpectedGrantType { expected: String, actual: String },

    #[error("missing code_verifier")]
    MissingCodeVerifier,

    #[error("PKCE verification failed")]
    PkceVerificationFailed,

    #[error("unknown \"client_id\" in Token Request: {0}")]
    UnknownClient(String),

    #[error(
        "\"client_id\" in Token Request does not match the one provided in Authorization Request: expected \
         {expected}, got {actual}"
    )]
    ClientIdMismatch { expected: String, actual: String },

    #[error("a Token Request containing authorization_details is not supported")]
    AuthorizationDetailsUnsupported,

    #[error(
        "scope received in Token Request does not match scope requested in Authorization Request: \
         expected {}, received: {}",
        .expected.iter().join(" "),
        .actual.iter().join(" ")
    )]
    ScopeMismatch {
        expected: HashSet<Scope>,
        actual: HashSet<Scope>,
    },

    #[error("use of scope values in Pre-Authorized Token Request is not supported, received: {}", .0.iter().join(" "))]
    PreAuthorizedScopeUnsupported(HashSet<Scope>),

    #[error("missing redirect_uri in Authorization Code flow")]
    MissingRedirectUri,

    #[error(
        "redirect_uri received in Token Request does not match the one in the Authorization Request: expected \
         {expected}, received: {actual}"
    )]
    RedirectUriMismatch { expected: Box<Url>, actual: Box<Url> },

    #[error("credential configuration not offered: {0}")]
    CredentialConfigNotOffered(CredentialConfigurationId),
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
    InvalidProofJwt(#[source] JwtVerifyError),

    #[error("could not extract holder public key from proof JWT: {0}")]
    InvalidProofPublicKey(#[source] JwkConversionError),

    #[error("nonce is not provided in credential request proof")]
    MissingProofNonce,

    #[error("could not check proof nonce against nonce storage: {0}")]
    ProofNonceStore(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("invalid nonce used in credential request proof")]
    InvalidNonce,

    #[error("JWT error: {0}")]
    Jwt(#[from] JwtVerifyError),

    #[error("missing credential configuration with identifier: {0}")]
    MissingCredentialConfiguration(CredentialConfigurationId),

    #[error("mismatch between requested: {requested} and offered attestation types: {offered}")]
    CredentialTypeMismatch { requested: Format, offered: Format },

    #[error("wrong number of credential requests")]
    WrongNumberOfCredentialRequests,

    #[error("missing credential request proof of possession")]
    MissingCredentialRequestPoP,

    #[error("error converting holder VerifyingKey to JWK: {0}")]
    JwkConversion(#[from] JwkConversionError),

    #[error("error converting CredentialPayload to Mdoc: {0}")]
    MdocConversion(#[from] CredentialPayloadIntoSignedMdocError),

    #[error("error converting CredentialPayload to SD-JWT: {0}")]
    SdJwtConversion(#[from] CredentialPayloadIntoSignedSdJwtError),

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

    #[error("missing credential configuration with identifier: {0}")]
    MissingCredentialConfiguration(CredentialConfigurationId),
}

/// Session keyed by a code that the wallet will exchange at `/token`. Covers both grant types:
/// `Grant::PreAuthorizedCode` (no PKCE) and `Grant::AuthorizationCode` (PKCE-verified at `/token`).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthCodeIssued {
    pub grant: Grant,
    pub credential_ids_and_documents: VecNonEmpty<(CredentialConfigurationId, IssuableDocument)>,
}

/// Values present in the (pushed) Authorization Request that initiated the Authorization Code Flow.
#[derive(Debug, Clone, PartialEq, Eq, Constructor, Serialize, Deserialize)]
pub struct AuthRequestValues {
    pub client_id: String,
    pub redirect_uri: Url,
    pub code_challenge: String,
    // Note that the "scope" value has already been used to select issuable credentials and is at this point only
    // present in the state in order to validate any "scope" that is received in the Token Request.
    pub scope: HashSet<Scope>,
}

#[derive(Debug, Clone, strum::Display, Serialize, Deserialize)]
pub enum Grant {
    PreAuthorizedCode,
    AuthorizationCode(AuthRequestValues),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenIssued {
    pub access_token: AccessToken,
    pub prepared_credentials: VecNonEmpty<PreparedCredential>,
    pub dpop_public_key: VerifyingKey,
    pub dpop_nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedCredential {
    pub credential_configuration_id: CredentialConfigurationId,
    pub format: Format,
    pub credential_payload: PreviewableCredentialPayload,
    pub batch_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Done {
    pub session_result: SessionResult,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IssuanceData {
    AuthCodeIssued(Box<AuthCodeIssued>),
    AccessTokenIssued(Box<AccessTokenIssued>),
    Done(Done),
}

impl SessionDataType for IssuanceData {
    const TYPE: &'static str = "openid4vci_issuance";
}

impl HasProgress for IssuanceData {
    fn progress(&self) -> Progress {
        match self {
            Self::AuthCodeIssued(_) | Self::AccessTokenIssued(_) => Progress::Active,
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
impl IssuanceState for AuthCodeIssued {}
impl IssuanceState for AccessTokenIssued {}
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

pub struct Issuer<K, L, S, N> {
    issuer_data: IssuerData<K, L>,
    sessions: Arc<S>,
    proof_nonce_store: Arc<N>,
    status_list_refresh_tasks: Vec<AbortHandle>,
}

/// Fields of the [`Issuer`] needed by the issuance functions.
pub struct IssuerData<K, L> {
    credential_configs: CredentialConfigurations<K, L>,

    /// Wallet IDs accepted by this server, MUST be used by the wallet as `iss` in its PoP JWTs.
    accepted_wallet_client_ids: HashSet<String>,

    /// URL prefix of the `/token`, `/credential` and `/batch_crededential` endpoints.
    server_url: BaseUrl,

    metadata: IssuerMetadata,
    metadata_keypair: KeyPair<K>,

    /// Public key of the WIA issuer.
    wia_trust_anchors: TrustAnchors,
}

impl<K, L> IssuerData<K, L> {
    fn get_checked_credential_config(
        &self,
        config_id: &CredentialConfigurationId,
        format: Format,
        attestation_type: &str,
    ) -> Option<&CredentialConfiguration<K, L>> {
        self.credential_configs
            .get_by_configuration_id(config_id)
            .and_then(|config| {
                // Do a sanity check to see if the credential configuration has changed since determining its id.
                let credential_kind = &config.credential_kind;
                (credential_kind.format == format && credential_kind.attestation_type == attestation_type)
                    .then_some(config)
            })
    }

    fn get_credential_config_for_prepared_credential(
        &self,
        credential: &PreparedCredential,
    ) -> Option<&CredentialConfiguration<K, L>> {
        self.get_checked_credential_config(
            &credential.credential_configuration_id,
            credential.format,
            &credential.credential_payload.attestation_type,
        )
    }

    fn accepted_wallet_client_ids_vec(&self) -> Vec<&String> {
        self.accepted_wallet_client_ids.iter().collect()
    }

    fn verify_wia(&self, wia_disclosure: &WiaDisclosure, client_id: Option<&String>) -> Result<WiaClaims, WiaError> {
        // The RFC says we should use the Issuer Identifier of the Authorization for this (see
        // https://datatracker.ietf.org/doc/html/draft-ietf-oauth-attestation-based-client-auth-09#section-5.1-5.1.1.)
        // In this implementation, that coincides with the Issuer Identifier of the OpenID4VCI issuer.
        let expected_aud = self.metadata.credential_issuer.as_ref();

        wia_disclosure.verify(
            &self.wia_trust_anchors,
            expected_aud,
            &self.accepted_wallet_client_ids_vec(),
            None,
            client_id,
        )
    }
}

impl PreparedCredential {
    /// Convert the tuple of a [`CredentialConfigurationId`] and [`IssuableDocument`] to the state needed to both serve
    /// previews and actually issue credentials by adding the required timestamps. This will return the passed
    /// [`CredentialConfigurationId`] as the `Err` variant if the corresponding [`CredentialConfiguration`] cannot
    /// be found.
    fn try_new<K, L>(
        config_id: CredentialConfigurationId,
        issuable_document: IssuableDocument,
        issuer_data: &IssuerData<K, L>,
    ) -> Result<Self, CredentialConfigurationId> {
        let Some(credential_config) = issuer_data.get_checked_credential_config(
            &config_id,
            issuable_document.credential_kind.format,
            &issuable_document.credential_kind.attestation_type,
        ) else {
            return Err(config_id);
        };

        // Note that we rely in the `IssuableDocument` being validated against the Credential Configuration's metadata
        // when the session was created. Since the issuer state may be persisted in a database, it could be that the
        // Credential Configuration has changed since then. However, we rely on the sanity check when retrieving the
        // Credential Configuration and the fact that the metadata should never actually change.
        //
        // If this does occur for some reason, it is still the Wallet's responisibility to check the metadata on
        // reception of the issued credentials.

        // Truncate the current time to only include the date part, so that all issued credentials on a single
        // day have the same `nbf` and `exp` field
        let now = utc_now_truncated_to_days();
        let valid_until = now.add(credential_config.valid_days);

        let format = issuable_document.credential_kind.format;
        let (batch_id, credential_payload) = issuable_document.into_id_and_previewable_credential_payload(
            now,
            valid_until,
            credential_config.issuer_uri.clone(),
            credential_config.attestation_qualification,
        );

        let credential = Self {
            credential_configuration_id: config_id,
            format,
            credential_payload,
            batch_id,
        };

        Ok(credential)
    }
}

impl<K, L, S, N> Drop for Issuer<K, L, S, N> {
    fn drop(&mut self) {
        // Stop the tasks at the next .await
        for task in &self.status_list_refresh_tasks {
            task.abort();
        }
    }
}

impl<K, L, S, N> Issuer<K, L, S, N> {
    pub(crate) fn credential_configs(&self) -> &CredentialConfigurations<K, L> {
        &self.issuer_data.credential_configs
    }

    pub fn status_lists(&self) -> impl Iterator<Item = &L> {
        self.issuer_data
            .credential_configs
            .configurations()
            .map(|config| &config.status_list)
    }

    pub fn accepted_wallet_client_ids(&self) -> &HashSet<String> {
        &self.issuer_data.accepted_wallet_client_ids
    }

    pub fn issuer_identifier(&self) -> &IssuerIdentifier {
        &self.issuer_data.metadata.credential_issuer
    }

    pub fn metadata(&self) -> &IssuerMetadata {
        &self.issuer_data.metadata
    }

    pub fn type_metadata(&self, id: &CredentialConfigurationId) -> Option<TypeMetadataDocuments> {
        self.issuer_data
            .credential_configs
            .get_by_configuration_id(id)
            .map(|config| config.metadata.documents().clone().into())
    }
}

impl<K, L, S, N> Issuer<K, L, S, N>
where
    K: EcdsaKeySend,
{
    pub async fn signed_metadata<G: Generator<DateTime<Utc>>>(
        &self,
        ttl: Duration,
        time_generator: G,
    ) -> Result<SignedJwt<SignedIssuerMetadataPayload<'_>, HeaderWithX5c>, JwtSignError> {
        let iat = time_generator.generate();
        let exp = iat + ttl;
        let payload = SignedIssuerMetadataPayload {
            metadata: Cow::Borrowed(&self.issuer_data.metadata),

            iss: None,
            sub: Cow::Borrowed(self.issuer_identifier()),
            iat: iat.into(),
            exp: Some(exp.into()),
        };
        SignedJwt::sign_with_certificate(&payload, &self.issuer_data.metadata_keypair).await
    }
}

impl<K, L, S, N> Issuer<K, L, S, N>
where
    S: SessionStore<IssuanceData> + Sync + 'static,
    N: NonceStore + Sync + 'static,
    L: StatusListService,
{
    #[expect(clippy::too_many_arguments, reason = "Constructor")]
    pub fn try_new(
        issuer_identifier: IssuerIdentifier,
        keypair: KeyPair<K>,
        batch_size: NonZeroU8,
        wallet_client_ids: HashSet<String>,
        credential_config_params: HashMap<CredentialConfigurationId, CredentialConfigurationParameters<K, L>>,
        wia_trust_anchors: TrustAnchors,
        sessions: Arc<S>,
        proof_nonce_store: N,
    ) -> Result<Self, CredentialConfigurationsError> {
        let credential_configs = CredentialConfigurations::try_new(credential_config_params)?;

        let server_url = issuer_identifier.as_issuer_url().join_issuer_url("/issuance");
        let credential_endpoint = server_url.join_issuer_url("/credential");
        let batch_credential_endpoint = server_url.join_issuer_url("/batch_credential");
        let nonce_endpoint = server_url.join_issuer_url("/nonce");
        let credential_preview_endpoint = server_url.join_issuer_url("/credential_preview");
        let type_metadata_base_url = server_url.join_issuer_url("/type_metadata");

        let batch_credential_issuance = AtLeastTwoU64::try_new(batch_size.into())
            .ok()
            .map(|batch_size| BatchCredentialIssuance { batch_size });
        let metadata = IssuerMetadata {
            credential_issuer: issuer_identifier,
            authorization_servers: None,
            endpoints: IssuerEndpoints {
                credential_endpoint,
                batch_credential_endpoint: Some(batch_credential_endpoint),
                nonce_endpoint: Some(nonce_endpoint),
                deferred_credential_endpoint: None,
                notification_endpoint: None,
                credential_preview_endpoint: Some(credential_preview_endpoint),
            },
            credential_request_encryption: None,
            credential_response_encryption: None,
            batch_credential_issuance,
            display: None,
            credential_configurations_supported: credential_configs
                .to_credential_configurations_supported(&type_metadata_base_url),
        };

        let issuer_data = IssuerData {
            credential_configs,
            accepted_wallet_client_ids: wallet_client_ids,
            wia_trust_anchors,

            // In this implementation, the public server URL is composed of the
            // Credential Issuer Identifier appended with the "/issuance/" path.
            server_url: server_url.into_inner(),
            metadata,
            metadata_keypair: keypair,
        };

        let proof_nonce_store = Arc::new(proof_nonce_store);

        let status_list_refresh_tasks = issuer_data
            .credential_configs
            .configurations()
            .map(|config| config.status_list.start_refresh_job())
            .collect();

        let issuer = Self {
            issuer_data,
            sessions,
            proof_nonce_store,
            status_list_refresh_tasks,
        };

        Ok(issuer)
    }
}

impl<K, L, S, N> PeriodicCleanup for Issuer<K, L, S, N>
where
    K: Send + Sync,
    L: Send + Sync,
    S: SessionStore<IssuanceData> + Send + Sync,
    N: NonceStore + Send + Sync,
{
    /// Removes expired sessions and proof nonces.
    ///
    /// Scheduled by the server via [`start_cleanup_task`](crate::cleanup::start_cleanup_task).
    async fn cleanup(&self) {
        let _ = join!(
            log_cleanup_error("session", self.sessions.cleanup()),
            log_cleanup_error("proof nonce", self.proof_nonce_store.remove_expired_nonces()),
        );
    }
}

fn logged_issuance_result<T, E: std::error::Error>(result: Result<T, E>) -> Result<T, E> {
    result
        .inspect(|_| info!("Issuance success"))
        .inspect_err(|error| info!("Issuance error: {error}"))
}

impl<K, L, S, N> Issuer<K, L, S, N> {
    /// Process each [`IssuableDocument`] in a collection by finding the corresponding [`CredentialConfiguration`],
    /// validating it against that configuration's metadata and returning it along with the
    /// [`CredentialConfigurationId`]. This is called both for Pre-Authorized sessions (by
    /// `Isser::new_preauthorized_session()`) and Authorization Code sessions (by
    /// `AuthorizingIssuer::complete_authorization()`).
    pub(crate) fn validate_issuable_documents(
        &self,
        issuable_documents: VecNonEmpty<IssuableDocument>,
    ) -> Result<VecNonEmpty<(CredentialConfigurationId, IssuableDocument)>, IssuableDocumentError> {
        issuable_documents
            .into_nonempty_iter()
            .map(|document| {
                let (credential_config_id, credential_config) = self
                    .issuer_data
                    .credential_configs
                    .get_by_credential_kind(&document.credential_kind)
                    .ok_or_else(|| IssuableDocumentError::CredentialTypeNotOffered(document.credential_kind.clone()))?;

                document
                    .validate_with_metadata(credential_config.metadata.normalized())
                    .map_err(IssuableDocumentError::AttributesError)?;

                Ok((credential_config_id.clone(), document))
            })
            .collect()
    }
}

impl<K, L, S, N> Issuer<K, L, S, N>
where
    S: SessionStore<IssuanceData>,
{
    /// Create and store a new Pre-Authorized session, based on a collection of [`IssuableDocument`]s and return a
    /// [`CredentialConfiguration`] that can be presented to the wallet.
    pub async fn new_preauthorized_session(
        &self,
        issuable_documents: VecNonEmpty<IssuableDocument>,
    ) -> Result<CredentialOffer, PreAuthorizedSessionError> {
        let credential_ids_and_documents = self
            .validate_issuable_documents(issuable_documents)
            .map_err(PreAuthorizedSessionError::IssuableDocument)?;

        let config_ids = credential_ids_and_documents
            .nonempty_iter()
            .map(|(config_id, _document)| config_id)
            .cloned()
            .collect();

        let token = self
            .write_auth_code_issued_session(AuthCodeIssued {
                grant: Grant::PreAuthorizedCode,
                credential_ids_and_documents,
            })
            .await
            .map_err(PreAuthorizedSessionError::SessionStore)?;

        let credential_offer = CredentialOffer::new_pre_authorized(
            self.issuer_data.metadata.credential_issuer.clone(),
            config_ids,
            token.into(),
        );

        Ok(credential_offer)
    }

    /// Persist a new session that is in the initial [`AuthCodeIssued`] state. This is called both for Pre-Authorized
    /// sessions (by `Isser::new_preauthorized_session()`) and Authorization Code sessions (by
    /// `AuthorizingIssuer::complete_authorization()`).
    pub(crate) async fn write_auth_code_issued_session(
        &self,
        auth_code_issued: AuthCodeIssued,
    ) -> Result<SessionToken, SessionStoreError> {
        let token = SessionToken::new_random();
        let session = SessionState::new(token.clone(), IssuanceData::AuthCodeIssued(Box::new(auth_code_issued)));

        self.sessions.write(session, true).await?;

        Ok(token)
    }

    async fn get_session(&self, code: AuthorizationCode) -> Result<Session<AccessTokenIssued>, CredentialRequestError> {
        self.sessions
            .get(&code.clone().into())
            .await
            .map_err(IssuanceError::SessionStore)?
            .ok_or(IssuanceError::UnknownSession(code))?
            .try_into()
            .map_err(CredentialRequestError::IssuanceError)
    }
}

impl<K, L, S, N> Issuer<K, L, S, N>
where
    N: NonceStore,
{
    pub async fn generate_proof_nonce(&self) -> Result<Nonce, NonceStoreError<N::Error>> {
        let nonce = Nonce::new_random();

        self.proof_nonce_store.store_nonce(nonce.clone()).await?;

        Ok(nonce)
    }
}

impl<K, L, S, N> Issuer<K, L, S, N> {
    pub fn oauth_metadata(&self) -> AuthorizationServerMetadata {
        let issuer_url = self.issuer_data.metadata.credential_issuer.as_base_url();

        AuthorizationServerMetadata {
            authorization_endpoint: Some(issuer_url.join("/issuance/authorize")),
            pushed_authorization_request_endpoint: Some(issuer_url.join("/issuance/par")),
            require_pushed_authorization_requests: true,
            token_endpoint_auth_methods_supported: Some(IndexSet::from([WIA_CLIENT_AUTH_METHOD.to_string()])),
            client_attestation_signing_alg_values_supported: Some(IndexSet::from([JwsAlgorithm::ES256])),
            client_attestation_pop_signing_alg_values_supported: Some(IndexSet::from([JwsAlgorithm::ES256])),
            ..AuthorizationServerMetadata::new(
                self.issuer_data.metadata.credential_issuer.clone(),
                issuer_url.join("issuance/token"),
            )
        }
    }
}

impl<K, L, S, N> Issuer<K, L, S, N>
where
    S: SessionStore<IssuanceData>,
{
    pub async fn process_credential_preview(
        &self,
        access_token: AccessToken,
    ) -> Result<CredentialPreviewResponse, CredentialPreviewError> {
        let code = access_token.code().ok_or(CredentialPreviewError::MalformedToken)?;

        let session: Session<AccessTokenIssued> = self
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

        let credential_previews = session_data
            .prepared_credentials
            .nonempty_iter()
            .map(|state| self.credential_preview_for_credential(state))
            .collect::<Result<_, _>>()?;

        Ok(CredentialPreviewResponse { credential_previews })
    }

    fn credential_preview_for_credential(
        &self,
        credential: &PreparedCredential,
    ) -> Result<CredentialPreview, CredentialPreviewError> {
        let credential_config = self
            .issuer_data
            .get_credential_config_for_prepared_credential(credential)
            .ok_or_else(|| {
                CredentialPreviewError::MissingCredentialConfiguration(credential.credential_configuration_id.clone())
            })?;

        let preview = CredentialPreview {
            config_id: credential.credential_configuration_id.clone(),
            format: credential.format,
            credential_payload: credential.credential_payload.clone(),
            issuer_certificate: credential_config.key_pair.certificate().clone(),
        };

        Ok(preview)
    }
}

impl<K, L, S, N> Issuer<K, L, S, N>
where
    K: EcdsaKey,
    S: SessionStore<IssuanceData>,
{
    /// Process a token request. The session must already exist, populated by a flow-specific
    /// provisioner: either via [`Issuer::new_preauthorized_session`] for the pre-authorized-code
    /// flow, or via [`AuthorizingIssuer::complete_authorization`] for the authorization-code
    /// flow (wallet PKCE is then verified by the `openid4vc` layer at `/token`).
    pub async fn process_token_request(
        &self,
        token_request: TokenRequest,
        dpop: Dpop,
        wia_disclosure: WiaDisclosure,
    ) -> Result<(TokenResponse, String), TokenRequestError> {
        let session_token = token_request.code().clone().into();

        let session = self
            .sessions
            .get(&session_token)
            .await
            .map_err(IssuanceError::SessionStore)?
            .ok_or(TokenRequestError::SessionNotFound)?;

        let result = Session::<AuthCodeIssued>::try_from(session)
            .map_err(TokenRequestError::IssuanceError)?
            .process_token_request(
                &token_request,
                dpop,
                &wia_disclosure,
                &self.issuer_data.server_url,
                &self.issuer_data,
            );

        let (response, next) = match result {
            Ok((response, dpop_nonce, next)) => (Ok((response, dpop_nonce)), next.into()),
            Err(boxed) => {
                let (err, next) = *boxed;
                (Err(err), next.into())
            }
        };

        self.sessions
            .write(next, false)
            .await
            .map_err(|e| TokenRequestError::IssuanceError(IssuanceError::SessionStore(e)))?;

        response
    }
}

impl<K, L, S, N> Issuer<K, L, S, N>
where
    K: EcdsaKey,
    L: StatusListService,
    S: SessionStore<IssuanceData>,
    N: NonceStore,
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
                self.proof_nonce_store.as_ref(),
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
                self.proof_nonce_store.as_ref(),
            )
            .await;

        self.sessions
            .write(next.into(), false)
            .await
            .map_err(IssuanceError::SessionStore)?;

        logged_issuance_result(response)
    }
}

impl<K, L, S, N> Issuer<K, L, S, N>
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

impl<K, L, S, N> Issuer<K, L, S, N> {
    pub(super) fn verify_wia(
        &self,
        wia_disclosure: &WiaDisclosure,
        client_id: Option<&String>,
    ) -> Result<WiaClaims, WiaError> {
        self.issuer_data.verify_wia(wia_disclosure, client_id)
    }
}

impl TryFrom<SessionState<IssuanceData>> for Session<AuthCodeIssued> {
    type Error = IssuanceError;

    fn try_from(value: SessionState<IssuanceData>) -> Result<Self, Self::Error> {
        let IssuanceData::AuthCodeIssued(session_data) = value.data else {
            return Err(IssuanceError::UnexpectedState);
        };
        Ok(Session::<AuthCodeIssued> {
            state: SessionState {
                data: *session_data,
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

/// Outcome of [`Session<AuthCodeIssued>::process_token_request`]: either the access-token response
/// paired with the `AccessTokenIssued` session, or the error paired with the failed session
/// the `openid4vc` layer should persist in its place. The `Err` variant is boxed to keep the size of
/// the `Result` reasonable.
type ProcessTokenRequest =
    Result<(TokenResponse, String, Session<AccessTokenIssued>), Box<(TokenRequestError, Session<Done>)>>;

impl Grant {
    /// Verify that the `grant_type` of the token request matches the grant captured for this session.
    fn verify_grant_type(&self, token_request: &TokenRequest) -> Result<(), TokenRequestError> {
        match (self, &token_request.grant_type) {
            (Grant::PreAuthorizedCode, TokenRequestGrantType::PreAuthorizedCode { .. }) => Ok(()),
            (Grant::AuthorizationCode(_), TokenRequestGrantType::AuthorizationCode { .. }) => Ok(()),
            _ => Err(TokenRequestError::UnexpectedGrantType {
                expected: self.to_string(),
                actual: token_request.grant_type.to_string(),
            }),
        }
    }

    /// Verify the wallet's PKCE `code_verifier` (RFC 7636). `PreAuthorizedCode` carries no PKCE and
    /// passes unconditionally; `AuthorizationCode` requires a `code_verifier` whose S256 challenge
    /// matches the one captured at `/authorize`.
    fn verify_pkce(&self, token_request: &TokenRequest) -> Result<(), TokenRequestError> {
        let Grant::AuthorizationCode(AuthRequestValues { code_challenge, .. }) = self else {
            // Pre-authorized-code grant: no PKCE to verify.
            return Ok(());
        };

        match token_request.code_verifier.as_deref() {
            None => Err(TokenRequestError::MissingCodeVerifier),
            Some(verifier) if S256PkcePair::challenge_for(verifier) == *code_challenge => Ok(()),
            Some(_) => Err(TokenRequestError::PkceVerificationFailed),
        }
    }

    /// Verify the following about the `client_id`:
    ///
    /// - In the Pre-Authorized flow, check that `client_id` is one of the allowed IDs.
    /// - In the Authorization Code flow, check that the `client_id` is exactly the same as the one provided in the
    ///   Authorization Request.
    fn verify_client_id(
        &self,
        client_id: String,
        accepted_wallet_client_ids: &HashSet<String>,
    ) -> Result<(), TokenRequestError> {
        match self {
            Grant::PreAuthorizedCode => {
                if !accepted_wallet_client_ids.contains(&client_id) {
                    return Err(TokenRequestError::UnknownClient(client_id));
                }
            }
            Grant::AuthorizationCode(AuthRequestValues {
                client_id: auth_client_id,
                ..
            }) => {
                if client_id != *auth_client_id {
                    return Err(TokenRequestError::ClientIdMismatch {
                        expected: auth_client_id.clone(),
                        actual: client_id,
                    });
                }
            }
        }

        Ok(())
    }

    /// Verify the `scope` of the [`TokenRequest`], if it is present.
    fn verify_scope(&self, token_request: &TokenRequest) -> Result<(), TokenRequestError> {
        match self {
            Grant::AuthorizationCode(AuthRequestValues {
                scope: request_scope, ..
            }) => {
                // The client has the option of further restricting the requested scope as included in the Authorization
                // Request in the Token Request. We choose not to have the issuer support this restriction, so instead
                // we check that the scope in the Token Request is exactly the same as what was included in the
                // Authorization Request.
                if let Some(scope) = token_request.scope.as_ref()
                    && scope != request_scope
                {
                    return Err(TokenRequestError::ScopeMismatch {
                        expected: request_scope.clone(),
                        actual: scope.clone(),
                    });
                }
            }

            Grant::PreAuthorizedCode => {
                // If the Token Request was Pre-Authorized, we choose not to support scope values at all.
                if let Some(scope) = token_request.scope.as_ref() {
                    return Err(TokenRequestError::PreAuthorizedScopeUnsupported(scope.clone()));
                }
            }
        }

        Ok(())
    }

    /// Verify the `redirect_uri` of the [`TokenRequest`] when in the Authorization Code flow.
    fn verify_redirect_uri(&self, token_request: &TokenRequest) -> Result<(), TokenRequestError> {
        if let Grant::AuthorizationCode(AuthRequestValues {
            redirect_uri: request_redirect_uri,
            ..
        }) = self
        {
            let redirect_uri = token_request
                .redirect_uri
                .as_ref()
                .ok_or(TokenRequestError::MissingRedirectUri)?;

            if redirect_uri != request_redirect_uri {
                return Err(TokenRequestError::RedirectUriMismatch {
                    expected: Box::new(request_redirect_uri.clone()),
                    actual: Box::new(redirect_uri.clone()),
                });
            }
        }

        Ok(())
    }
}

impl Session<AuthCodeIssued> {
    fn process_token_request<K, L>(
        self,
        token_request: &TokenRequest,
        dpop: Dpop,
        wia_disclosure: &WiaDisclosure,
        server_url: &BaseUrl,
        issuer_data: &IssuerData<K, L>,
    ) -> ProcessTokenRequest {
        let result =
            self.validate_and_build_token_response(token_request, dpop, wia_disclosure, server_url, issuer_data);

        self.finalize_token_response(result)
    }

    fn validate_and_build_token_response<K, L>(
        &self,
        token_request: &TokenRequest,
        dpop: Dpop,
        wia_disclosure: &WiaDisclosure,
        server_url: &BaseUrl,
        issuer_data: &IssuerData<K, L>,
    ) -> Result<(TokenResponse, VecNonEmpty<PreparedCredential>, VerifyingKey, String), TokenRequestError> {
        let wia_claims = issuer_data
            .verify_wia(wia_disclosure, token_request.client_id.as_ref())
            .map_err(TokenRequestError::Wia)?;

        let client_id = wia_claims.sub;
        let session_data = self.session_data();

        session_data.grant.verify_grant_type(token_request)?;
        session_data.grant.verify_pkce(token_request)?;
        session_data
            .grant
            .verify_client_id(client_id, &issuer_data.accepted_wallet_client_ids)?;

        if token_request.authorization_details.is_some() {
            return Err(TokenRequestError::AuthorizationDetailsUnsupported);
        }

        session_data.grant.verify_scope(token_request)?;
        session_data.grant.verify_redirect_uri(token_request)?;

        build_token_response(
            token_request.code(),
            dpop,
            server_url,
            session_data.credential_ids_and_documents.clone(),
            issuer_data,
        )
    }

    /// Apply the state transition on a `Session<AuthCodeIssued>` based on the result of
    /// [`build_token_response`]: success → `AccessTokenIssued`, failure → `Done`. The `Err`
    /// variant of the returned [`ProcessTokenRequest`] is boxed for size.
    fn finalize_token_response(
        self,
        result: Result<(TokenResponse, VecNonEmpty<PreparedCredential>, VerifyingKey, String), TokenRequestError>,
    ) -> ProcessTokenRequest {
        match result {
            Ok((token_response, prepared_credentials, dpop_pubkey, dpop_nonce)) => {
                let next = self.transition(AccessTokenIssued {
                    access_token: token_response.access_token.clone(),
                    prepared_credentials,
                    dpop_public_key: dpop_pubkey,
                    dpop_nonce: dpop_nonce.clone(),
                });
                Ok((token_response, dpop_nonce, next))
            }
            Err(err) => {
                let next = self.transition_fail(&err);
                Err(Box::new((err, next)))
            }
        }
    }
}

/// Verify DPoP, prepare credentials from the supplied (pre-provisioned) issuables, and
/// generate a fresh access token + DPoP nonce. Shared by the pre-authorized-code and
/// authorization-code token-request paths.
fn build_token_response<K, L>(
    token_request_auth_code: &AuthorizationCode,
    dpop: Dpop,
    server_url: &BaseUrl,
    credential_ids_and_documents: VecNonEmpty<(CredentialConfigurationId, IssuableDocument)>,
    issuer_data: &IssuerData<K, L>,
) -> Result<(TokenResponse, VecNonEmpty<PreparedCredential>, VerifyingKey, String), TokenRequestError> {
    let dpop_public_key = dpop
        .verify(&server_url.join("token"), &Method::POST, None)
        .map_err(|err| TokenRequestError::IssuanceError(IssuanceError::DpopInvalid(err)))?;

    // The issuer has a choice here to either include `scope` values that refer to the Credential Configurations of the
    // credentials offered or include an `authorization_details` field that includes both the Credential Configuration
    // Identifiers and the individual Credential Identifiers. As the former prohibits issuance of multiple credentials
    // within the same Credential Configuration, we choose the latter.
    let authorization_details = AuthorizationDetails::from_credential_ids_and_identifiers(
        credential_ids_and_documents
            .nonempty_iter()
            .map(|(config_id, document)| (config_id, document.id.to_string())),
    );

    let prepared_credentials = credential_ids_and_documents
        .into_nonempty_iter()
        .map(|(config_id, document)| {
            PreparedCredential::try_new(config_id, document, issuer_data)
                .map_err(TokenRequestError::CredentialConfigNotOffered)
        })
        .collect::<Result<_, _>>()?;

    let dpop_nonce = random_string(32);

    // Note that, in the Authorization Code flow, we assume that the implementer of `AuthorizationCodeFlow` provides all
    // of the credentials that are identified by the scopes that the wallet includes in the Authorization Request.
    // Therefore we do not need to include a `scope` value in the Token Response, as the scope that the wallet requested
    // is never curtailed.
    //
    // In the the Pre-Authorized Code flow we do not allow `scope` values from the wallet, so any scope restriction does
    // not apply.
    let token_response = TokenResponse::new_vci(AccessToken::new(token_request_auth_code), Some(authorization_details));

    Ok((token_response, prepared_credentials, dpop_public_key, dpop_nonce))
}

impl From<Session<AccessTokenIssued>> for SessionState<IssuanceData> {
    fn from(value: Session<AccessTokenIssued>) -> Self {
        SessionState {
            data: IssuanceData::AccessTokenIssued(Box::new(value.state.data)),
            token: value.state.token,
            last_active: value.state.last_active,
        }
    }
}

impl TryFrom<SessionState<IssuanceData>> for Session<AccessTokenIssued> {
    type Error = IssuanceError;

    fn try_from(value: SessionState<IssuanceData>) -> Result<Self, Self::Error> {
        let IssuanceData::AccessTokenIssued(session_data) = value.data else {
            return Err(IssuanceError::UnexpectedState);
        };
        Ok(Session::<AccessTokenIssued> {
            state: SessionState {
                data: *session_data,
                token: value.token,
                last_active: value.last_active,
            },
        })
    }
}

impl Session<AccessTokenIssued> {
    async fn process_credential<K, L, N>(
        self,
        credential_request: CredentialRequest,
        access_token: AccessToken,
        dpop: Dpop,
        issuer_data: &IssuerData<K, L>,
        proof_nonce_store: &N,
    ) -> (Result<CredentialResponse, CredentialRequestError>, Session<Done>)
    where
        K: EcdsaKey,
        N: NonceStore,
        L: StatusListService,
    {
        let result = self
            .process_credential_inner(credential_request, access_token, dpop, issuer_data, proof_nonce_store)
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

    async fn process_credential_inner<K, L, N>(
        &self,
        credential_request: CredentialRequest,
        access_token: AccessToken,
        dpop: Dpop,
        issuer_data: &IssuerData<K, L>,
        proof_nonce_store: &N,
    ) -> Result<CredentialResponse, CredentialRequestError>
    where
        K: EcdsaKey,
        L: StatusListService,
        N: NonceStore,
    {
        let session_data = self.session_data();

        self.check_credential_endpoint_access(&access_token, dpop, &issuer_data.server_url, "credential")?;

        // If we have exactly one credential on offer that matches the credential type that the client is
        // requesting, then we issue that credential.
        // NB: the OpenID4VCI specification leaves open how to make this decision, this is our own behaviour.
        let requested_format = credential_request.credential_type.as_ref().format();
        let offered_creds = session_data
            .prepared_credentials
            .iter()
            .filter(|credential| credential.format == requested_format)
            .collect_vec();

        let credential = match (offered_creds.first(), offered_creds.len()) {
            (Some(credential), 1) => Ok(*credential),
            (_, 0) => Err(CredentialRequestError::CredentialTypeNotOffered(
                credential_request.credential_type.as_ref().to_string(),
            )),
            // If we have more than one credential on offer of the specified credential type then it is not clear which
            // one we should issue; abort
            _ => Err(CredentialRequestError::UseBatchIssuance),
        }?;

        let (holder_pubkey, request_nonce) = credential_request.verify(
            &issuer_data.accepted_wallet_client_ids_vec(),
            &issuer_data.metadata.credential_issuer,
        )?;

        let nonce_status = proof_nonce_store
            .check_nonce_status_and_remove([request_nonce].iter())
            .await
            .map_err(|error| CredentialRequestError::ProofNonceStore(Box::new(error)))?;

        if !matches!(nonce_status, NonceStatus::AllValid) {
            return Err(CredentialRequestError::InvalidNonce);
        }

        let credential_config = issuer_data
            .get_credential_config_for_prepared_credential(credential)
            .ok_or_else(|| {
                CredentialRequestError::MissingCredentialConfiguration(credential.credential_configuration_id.clone())
            })?;

        let status_claim = credential_config
            .status_list
            .obtain_status_claims(
                credential.batch_id,
                credential.credential_payload.expires,
                NonZeroUsize::MIN,
            )
            .await
            .map_err(|err| CredentialRequestError::ObtainStatusClaim(Box::new(err)))?
            .into_first();

        let credential_response = CredentialResponse::new(
            requested_format,
            credential.credential_payload.clone(),
            utc_now_truncated_to_days(),
            &holder_pubkey,
            credential_config,
            status_claim,
        )
        .await?;

        Ok(credential_response)
    }

    async fn process_batch_credential<K, L, N>(
        self,
        credential_requests: CredentialRequests,
        access_token: AccessToken,
        dpop: Dpop,
        issuer_data: &IssuerData<K, L>,
        proof_nonce_store: &N,
    ) -> (Result<CredentialResponses, CredentialRequestError>, Session<Done>)
    where
        K: EcdsaKey,
        L: StatusListService,
        N: NonceStore,
    {
        let result = self
            .process_batch_credential_inner(credential_requests, access_token, dpop, issuer_data, proof_nonce_store)
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

    async fn process_batch_credential_inner<K, L, N>(
        &self,
        credential_requests: CredentialRequests,
        access_token: AccessToken,
        dpop: Dpop,
        issuer_data: &IssuerData<K, L>,
        proof_nonce_store: &N,
    ) -> Result<CredentialResponses, CredentialRequestError>
    where
        K: EcdsaKey,
        L: StatusListService,
        N: NonceStore,
    {
        let session_data = self.session_data();

        self.check_credential_endpoint_access(&access_token, dpop, &issuer_data.server_url, "batch_credential")?;

        let mut request_nonces = Vec::with_capacity(credential_requests.credential_requests.as_ref().len());
        let credentials_and_holder_pubkeys = session_data
            .prepared_credentials
            .iter()
            .map(|credential| {
                // For every credential collect for every copy the verified key
                let copy_count = issuer_data.metadata.batch_size().get();
                let format_pubkeys: VecNonEmpty<_> = (0..copy_count)
                    .map(|_| {
                        let cred_req = credential_requests
                            .credential_requests
                            .as_ref()
                            .get(request_nonces.len())
                            .ok_or(CredentialRequestError::WrongNumberOfCredentialRequests)?;

                        // Verify the assumption that the order of the incoming requests matches exactly
                        // that of the flattened batch_size by matching the requested format.
                        if credential.format != cred_req.credential_type.as_ref().format() {
                            return Err(CredentialRequestError::CredentialTypeMismatch {
                                offered: credential.format,
                                requested: cred_req.credential_type.as_ref().format(),
                            });
                        }

                        let (key, nonce) = cred_req.verify(
                            &issuer_data.accepted_wallet_client_ids_vec(),
                            &issuer_data.metadata.credential_issuer,
                        )?;

                        request_nonces.push(nonce);

                        Ok(key)
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .expect("guaranteerd to be non-empty because copy_count's source is non-zero");

                let credential_config = issuer_data
                    .get_credential_config_for_prepared_credential(credential)
                    .ok_or_else(|| {
                        CredentialRequestError::MissingCredentialConfiguration(
                            credential.credential_configuration_id.clone(),
                        )
                    })?;

                Ok((credential, credential_config, format_pubkeys))
            })
            .collect::<Result<Vec<_>, CredentialRequestError>>()?;

        // Verify that we have consumed all credential requests
        if request_nonces.len() != credential_requests.credential_requests.as_ref().len() {
            return Err(CredentialRequestError::WrongNumberOfCredentialRequests);
        }

        // Check the validity of all of the nonces used, which may be equal to each other.
        let nonce_status = proof_nonce_store
            .check_nonce_status_and_remove(request_nonces.iter())
            .await
            .map_err(|error| CredentialRequestError::ProofNonceStore(Box::new(error)))?;

        if !matches!(nonce_status, NonceStatus::AllValid) {
            return Err(CredentialRequestError::InvalidNonce);
        }

        // Obtain a status claim for every attestation copy, linked to a single batch id per credential
        let status_claims = try_join_all(credentials_and_holder_pubkeys.iter().map(
            |(credential, credential_config, format_pubkeys)| async move {
                let claims = credential_config
                    .status_list
                    .obtain_status_claims(
                        credential.batch_id,
                        credential.credential_payload.expires,
                        format_pubkeys.len(),
                    )
                    .await
                    .map_err(|err| CredentialRequestError::ObtainStatusClaim(Box::new(err)))?;
                if claims.len() != format_pubkeys.len() {
                    return Err(CredentialRequestError::IncorrectNumberOfStatusClaims(
                        credential.credential_payload.attestation_type.clone(),
                    ));
                }
                Ok(claims)
            },
        ))
        .await?;

        // Make sure all credentials are issued with the same `issued_at` timestamp
        let issued_at = utc_now_truncated_to_days();
        let credential_responses = try_join_all(
            credentials_and_holder_pubkeys
                .iter()
                // The claims size is explicitly checked to be equal to the number of copies
                .zip_eq(status_claims)
                .flat_map(|((credential, credential_config, format_pubkeys), claims)| {
                    format_pubkeys.into_iter().zip(claims.into_inner()).map(|(key, claim)| {
                        CredentialResponse::new(
                            credential.format,
                            credential.credential_payload.clone(),
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
    async fn new<K, L>(
        credential_format: Format,
        preview_credential_payload: PreviewableCredentialPayload,
        issued_at: DateTime<Utc>,
        holder_pubkey: &VerifyingKey,
        credential_config: &CredentialConfiguration<K, L>,
        status_claim: StatusClaim,
    ) -> Result<CredentialResponse, CredentialRequestError>
    where
        K: EcdsaKey,
    {
        let payload = CredentialPayload::from_previewable_credential_payload(
            preview_credential_payload,
            issued_at,
            holder_pubkey,
            credential_config.metadata.first_document_integrity().clone(),
            status_claim,
        )?;

        match credential_format {
            Format::MsoMdoc => Self::new_for_mdoc(payload, credential_config).await,
            Format::SdJwt => Self::new_for_sd_jwt(payload, credential_config).await,
        }
    }

    async fn new_for_mdoc<K, L>(
        credential_payload: CredentialPayload,
        credential_config: &CredentialConfiguration<K, L>,
    ) -> Result<CredentialResponse, CredentialRequestError>
    where
        K: EcdsaKey,
    {
        // Construct an mdoc `IssuerSigned` from the contents of `PreviewableCredentialPayload`
        // and the attestation config by signing it.
        let (issuer_signed, _) = credential_payload.into_signed_mdoc(&credential_config.key_pair).await?;

        Ok(CredentialResponse::new_immediate(Credential::new_mdoc(issuer_signed)))
    }

    async fn new_for_sd_jwt<K, L>(
        credential_payload: CredentialPayload,
        credential_config: &CredentialConfiguration<K, L>,
    ) -> Result<CredentialResponse, CredentialRequestError>
    where
        K: EcdsaKey,
    {
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
    use std::assert_matches;
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    use chrono::Timelike;
    use crypto::server_keys::KeyPair;
    use crypto::trust_anchor::TrustAnchors;
    use derive_more::Debug;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use thiserror::Error;
    use tracing_test::traced_test;
    use url::Url;
    use utils::generator::mock::MockTimeGenerator;
    use wscd::mock_remote::MockRemoteWscd;
    use wscd::mock_remote::MockWiaClient;
    use wscd::wscd::WiaClient;

    use super::*;
    use crate::cleanup::CLEANUP_INTERVAL;
    use crate::cleanup::start_cleanup_task;
    use crate::credential::CredentialRequest;
    use crate::credential::CredentialRequestProof;
    use crate::credential::CredentialRequests;
    use crate::credential::CredentialResponse;
    use crate::credential::CredentialResponses;
    use crate::dpop::Dpop;
    use crate::errors::CredentialErrorCode;
    use crate::errors::CredentialPreviewErrorCode;
    use crate::errors::ErrorResponse;
    use crate::errors::RemoteErrorCode;
    use crate::errors::TokenErrorCode;
    use crate::issuable_document::IssuableDocument;
    use crate::issuer_identifier::IssuerIdentifier;
    use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
    use crate::nonce::response::NonceResponse;
    use crate::preview::CredentialPreviewResponse;
    use crate::server_state::MemorySessionStore;
    use crate::server_state::test::memory_session_store_with_mock_time;
    use crate::server_state::test::test_memory_store_with_cleanup_task;
    use crate::test::MockIssuer;
    use crate::test::mock_issuable_documents;
    use crate::test::setup_mock_issuer;
    use crate::test::setup_mock_issuer_attestation_types_and_metadata;
    use crate::token::AccessToken;
    use crate::token::TokenRequest;
    use crate::token::TokenResponse;
    use crate::wallet_issuance::IssuanceSession;
    use crate::wallet_issuance::WalletIssuanceError;
    use crate::wallet_issuance::issuance_session::HttpIssuanceSession;
    use crate::wallet_issuance::issuance_session::VcMessageClient;

    #[tokio::test]
    async fn test_signed_metadata() {
        let (_, metadata) = TypeMetadataDocuments::degree_example();
        let (issuer, _, _) = setup_mock_issuer_attestation_types_and_metadata(
            "https://example.com/".parse().unwrap(),
            vec![(Format::SdJwt, "com.example.degree".to_string(), metadata)],
            Arc::new(MemorySessionStore::default()),
        );

        let time_generator = MockTimeGenerator::default();
        let ttl = Duration::from_secs(300);
        let now = time_generator.generate();
        let signed_metadata = issuer.signed_metadata(ttl, time_generator).await.unwrap();

        let (_, payload) = signed_metadata.into_unverified().dangerous_parse_unverified().unwrap();
        assert_eq!(payload.iss, None);
        assert_eq!(payload.sub.as_ref(), &payload.metadata.credential_issuer);
        assert_eq!(payload.iat, now.into());
        assert_eq!(payload.exp, Some((now + ttl).into()));
    }

    #[derive(Debug, Error, Clone, Eq, PartialEq)]
    #[error("MyError")]
    struct MyError;

    // Note that this needs to be async because `Issuer::try_new()` requires a tokio reactor.
    #[tokio::test]
    async fn test_prepared_credential_try_new() {
        let (_, metadata) = TypeMetadataDocuments::degree_example();
        let (issuer, _, _) = setup_mock_issuer_attestation_types_and_metadata(
            "https://example.com/".parse().unwrap(),
            vec![(Format::SdJwt, "com.example.degree".to_string(), metadata)],
            Arc::new(MemorySessionStore::default()),
        );

        let document = IssuableDocument::new_mock_degree("Education".to_string());

        let PreparedCredential { credential_payload, .. } = PreparedCredential::try_new(
            "com.example.degree_dc+sd-jwt".to_string().into(),
            document,
            &issuer.issuer_data,
        )
        .unwrap();

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

    fn setup_simple_mock_issuer() -> (MockIssuer, TrustAnchors, IssuerIdentifier, KeyPair) {
        let issuer_identifier: IssuerIdentifier = "https://example.com/".parse().unwrap();
        let (issuer, trust_anchor, wia_keypair) = setup_mock_issuer(
            issuer_identifier.clone(),
            NonZeroUsize::MIN,
            Arc::new(MemorySessionStore::default()),
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
        wia_override: Option<WiaDisclosure>,
    }

    impl VcMessageClientStub {
        fn new(issuer: MockIssuer) -> Self {
            Self {
                issuer,
                wrong_access_token: false,
                invalidate_dpop: false,
                invalidate_pop: false,
                wia_override: None,
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
            wia: &WiaDisclosure,
        ) -> Result<(TokenResponse, Option<String>), WalletIssuanceError> {
            let wia = self.wia_override.as_ref().unwrap_or(wia);
            let (token_response, dpop_nonce) = self
                .issuer
                .process_token_request(token_request.clone(), dpop_header.clone(), wia.clone())
                .await
                .map_err(|error| {
                    let error_response = ErrorResponse::<TokenErrorCode>::from(error);

                    WalletIssuanceError::TokenRequest(Box::new(error_response.into()))
                })?;
            Ok((token_response, Some(dpop_nonce)))
        }

        async fn request_credential_preview(
            &self,
            _url: &Url,
            access_token: &AccessToken,
        ) -> Result<CredentialPreviewResponse, WalletIssuanceError> {
            self.issuer
                .process_credential_preview(access_token.clone())
                .await
                .map_err(|error| {
                    let error_response = ErrorResponse::<CredentialPreviewErrorCode>::from(error);

                    WalletIssuanceError::CredentialPreview(Box::new(error_response.into()))
                })
        }

        async fn request_type_metadata(&self, url: Url) -> Result<TypeMetadataDocuments, WalletIssuanceError> {
            let id = url.path_segments().unwrap().next_back().unwrap().to_string().into();
            Ok(self.issuer.type_metadata(&id).unwrap())
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
                .map_err(|error| {
                    let error_response = ErrorResponse::<CredentialErrorCode>::from(error);

                    WalletIssuanceError::CredentialRequest(Box::new(error_response.into()))
                })
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
                .map_err(|error| {
                    let error_response = ErrorResponse::<CredentialErrorCode>::from(error);

                    WalletIssuanceError::CredentialRequest(Box::new(error_response.into()))
                })
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
                .map_err(|error| {
                    let error_response = ErrorResponse::<CredentialErrorCode>::from(error);

                    WalletIssuanceError::CredentialRejection(Box::new(error_response.into()))
                })
        }
    }

    async fn start_and_accept_err(
        message_client: VcMessageClientStub,
        issuer_identifier: IssuerIdentifier,
        trust_anchors: TrustAnchors,
        wia_keypair: KeyPair,
    ) -> WalletIssuanceError {
        let credential_offer = message_client
            .issuer
            .new_preauthorized_session(mock_issuable_documents(NonZeroUsize::MIN))
            .await
            .unwrap();

        let issuer_metadata = message_client.issuer.metadata().clone();
        let oauth_metadata = AuthorizationServerMetadata::new_mock(issuer_identifier);

        let credential_configs = credential_offer
            .credential_configuration_ids
            .into_nonempty_iter()
            .map(|config_id| {
                let config = issuer_metadata
                    .credential_configurations_supported
                    .get(&config_id)
                    .unwrap()
                    .clone();

                (config_id, config)
            })
            .collect();

        let code = credential_offer
            .grants
            .unwrap()
            .pre_authorized_code
            .unwrap()
            .pre_authorized_code;
        let batch_size = issuer_metadata.batch_size().try_into().unwrap();
        let mut session = HttpIssuanceSession::create(
            message_client,
            credential_configs,
            issuer_metadata.credential_issuer,
            issuer_metadata.endpoints,
            batch_size,
            &oauth_metadata.token_endpoint,
            TokenRequest::new_mock_with_pre_authorized_code(code),
            &MockWiaClient::new_with_wia_keypair(wia_keypair),
            &oauth_metadata.issuer,
            &trust_anchors,
        )
        .await
        .unwrap();

        let wscd = MockRemoteWscd::new(vec![]);
        session.accept_issuance(&trust_anchors, &wscd).await.unwrap_err()
    }

    /// Like [`start_and_accept_err`] but for errors that happen at token request time (inside
    /// `HttpIssuanceSession::create`). The `wia_override` on the stub is used to inject a bad WIA;
    /// the `wia_client` passed to `create` is a dummy because the stub ignores it.
    async fn start_token_request_err(
        message_client: VcMessageClientStub,
        issuer_identifier: IssuerIdentifier,
        trust_anchors: TrustAnchors,
        wia_client: &impl WiaClient,
    ) -> WalletIssuanceError {
        let session_token = message_client
            .issuer
            .new_preauthorized_session(mock_issuable_documents(NonZeroUsize::MIN))
            .await
            .unwrap()
            .grants
            .unwrap()
            .pre_authorized_code
            .unwrap()
            .pre_authorized_code;

        let issuer_metadata = message_client.issuer.metadata().clone();
        let oauth_metadata = AuthorizationServerMetadata::new_mock(issuer_identifier);

        let credential_configs = message_client
            .issuer
            .credential_configs()
            .all_configuration_ids()
            .into_nonempty_iter()
            .map(|id| {
                (
                    id.clone(),
                    issuer_metadata.credential_configurations_supported[id].clone(),
                )
            })
            .collect();

        let batch_size = issuer_metadata.batch_size().try_into().unwrap();
        HttpIssuanceSession::create(
            message_client,
            credential_configs,
            issuer_metadata.credential_issuer,
            issuer_metadata.endpoints,
            batch_size,
            &oauth_metadata.token_endpoint,
            TokenRequest::new_mock_with_pre_authorized_code(session_token),
            wia_client,
            &oauth_metadata.issuer,
            &trust_anchors,
        )
        .await
        .err()
        .expect("HttpIssuanceSession::create should have returned an error")
    }

    #[tokio::test]
    async fn token_request_rejects_wia_from_untrusted_issuer() {
        let (issuer, trust_anchor, issuer_identifier, _wia_keypair) = setup_simple_mock_issuer();

        // WIA signed by a freshly generated CA that is not in the issuer's trust anchors.
        let bad_wia = MockWiaClient::new()
            .issue_wia(issuer_identifier.to_string(), None)
            .await
            .unwrap();

        let message_client = VcMessageClientStub {
            wia_override: Some(bad_wia),
            ..VcMessageClientStub::new(issuer)
        };

        let error =
            start_token_request_err(message_client, issuer_identifier, trust_anchor, &MockWiaClient::new()).await;
        assert_matches!(
            error,
            WalletIssuanceError::TokenRequest(err)
                if matches!(err.error, RemoteErrorCode::Known(TokenErrorCode::InvalidClientAttestation))
        );
    }

    #[tokio::test]
    async fn token_request_rejects_wia_with_wrong_audience() {
        let (issuer, trust_anchor, issuer_identifier, wia_keypair) = setup_simple_mock_issuer();

        // WIA signed by the trusted key pair but targeting a different audience.
        let bad_wia = MockWiaClient::new_with_wia_keypair(wia_keypair)
            .issue_wia("https://wrong-issuer.example.com".to_string(), None)
            .await
            .unwrap();

        let message_client = VcMessageClientStub {
            wia_override: Some(bad_wia),
            ..VcMessageClientStub::new(issuer)
        };

        let error =
            start_token_request_err(message_client, issuer_identifier, trust_anchor, &MockWiaClient::new()).await;
        assert_matches!(
            error,
            WalletIssuanceError::TokenRequest(err)
                if matches!(err.error, RemoteErrorCode::Known(TokenErrorCode::InvalidClientAttestation))
        );
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
            WalletIssuanceError::CredentialRequest(err)
                if matches!(err.error, RemoteErrorCode::Known(CredentialErrorCode::InvalidToken))
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
            WalletIssuanceError::CredentialRequest(err)
                if matches!(err.error, RemoteErrorCode::Known(CredentialErrorCode::InvalidCredentialRequest))
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
            WalletIssuanceError::CredentialRequest(err)
                if matches!(err.error, RemoteErrorCode::Known(CredentialErrorCode::InvalidProof))
        );
    }

    #[tokio::test]
    async fn test_cleanup_task() {
        let documents = mock_issuable_documents(NonZeroUsize::MIN);

        let (sessions, mock_time) = memory_session_store_with_mock_time();
        let sessions = Arc::new(sessions);

        let (issuer, _, _) = setup_mock_issuer(
            "https://example.com/".parse().unwrap(),
            NonZeroUsize::MIN,
            sessions.clone(),
        );

        let credential_offer = issuer.new_preauthorized_session(documents).await.unwrap();
        let code = credential_offer
            .grants
            .unwrap()
            .pre_authorized_code
            .unwrap()
            .pre_authorized_code;

        // The Issuer doesn't schedules its own cleanup; start one explicitly so the expired session gets purged.
        let _cleanup_task = start_cleanup_task(CLEANUP_INTERVAL, Arc::new(issuer));

        test_memory_store_with_cleanup_task(sessions, code.into(), &mock_time, CLEANUP_INTERVAL).await;
    }
}
