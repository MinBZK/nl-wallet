use std::collections::HashMap;
use std::convert::Infallible;
use std::num::NonZeroU8;
use std::ops::Add;
use std::sync::Arc;

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
use tokio::task::JoinHandle;
use tracing::info;
use uuid::Uuid;

use attestation_data::attributes::AttributesError;
use attestation_data::credential_payload::CredentialPayload;
use attestation_data::credential_payload::MdocCredentialPayloadError;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::credential_payload::SdJwtCredentialPayloadError;
use attestation_data::issuable_document::IssuableDocument;
use attestation_types::qualification::AttestationQualification;
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
use jwt::wua::WuaDisclosure;
use jwt::wua::WuaError;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataChainError;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use token_status_list::status_claim::StatusClaim;
use token_status_list::status_service::StatusClaimService;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use wscd::Poa;
use wscd::PoaVerificationError;

use crate::Format;
use crate::credential::CredentialRequest;
use crate::credential::CredentialRequestProof;
use crate::credential::CredentialRequests;
use crate::credential::CredentialResponse;
use crate::credential::CredentialResponses;
use crate::dpop::Dpop;
use crate::dpop::DpopError;
use crate::metadata;
use crate::metadata::CredentialMetadata;
use crate::metadata::CredentialResponseEncryption;
use crate::metadata::IssuerMetadata;
use crate::oidc;
use crate::server_state::CLEANUP_INTERVAL_SECONDS;
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
use crate::token::TokenResponseWithPreviews;
use crate::token::TokenType;

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

    #[error("unsupported token request type: must be of type pre-authorized_code")]
    UnsupportedTokenRequestType,

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

    #[error("incorrect nonce")]
    IncorrectNonce,

    #[error("unsupported JWT: {0}")]
    UnsupportedJwt(#[source] JwtError),

    #[error("JWK conversion error: {0}")]
    JwkConversion(#[from] JwkConversionError),

    #[error("JWT error: {0}")]
    Jwt(#[from] JwtError),

    #[error("missing attestation type config for {0}")]
    MissingAttestationTypeConfiguration(String),

    #[error("failed to sign credential: {0}")]
    CredentialSigning(mdoc::Error),

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

    #[error("error converting CredentialPayload to Mdoc: {0}")]
    MdocConversion(#[from] MdocCredentialPayloadError),

    #[error("error converting CredentialPayload to SD-JWT: {0}")]
    SdJwtConversion(#[from] SdJwtCredentialPayloadError),

    #[error("error verifying WUA: {0}")]
    Wua(#[from] WuaError),

    #[error("error obtaining status claim: {0}")]
    StatusClaimService(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("incorrect number of status claims for attestation_type: {0}")]
    IncorrectNumberOfStatusClaims(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Created {
    pub issuable_documents: Option<VecNonEmpty<IssuableDocument>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitingForResponse {
    pub access_token: AccessToken,
    pub c_nonce: String,
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
    pub copies_per_format: IndexMap<Format, NonZeroU8>,
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

/// Implementations of this trait are responsible for determine the attributes to be issued, given the session and
/// the token request. See for example the [`BrpPidAttributeService`].
#[trait_variant::make(Send)]
pub trait AttributeService {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn attributes(&self, token_request: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Self::Error>;

    async fn oauth_metadata(&self, issuer_url: &BaseUrl) -> Result<oidc::Config, Self::Error>;
}

pub struct TrivialAttributeService;

impl AttributeService for TrivialAttributeService {
    type Error = Infallible;

    async fn attributes(&self, _: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Self::Error> {
        unimplemented!()
    }

    async fn oauth_metadata(&self, issuer_url: &BaseUrl) -> Result<oidc::Config, Self::Error> {
        // TODO (PVW-4257): we don't use the `authorize` and `jwks` endpoint here, but we need to specify them
        // because they are mandatory in an OIDC Provider Metadata document (see
        // <https://openid.net/specs/openid-connect-discovery-1_0.html>).
        // However, OpenID4VCI says that this should return not an OIDC Provider Metadata document but an OAuth
        // Authorization Metadata document instead, see <https://www.rfc-editor.org/rfc/rfc8414.html>, which to
        // a large extent has the same fields but `authorize` and `jwks` are optional there.

        Ok(oidc::Config::new(
            issuer_url.clone(),
            issuer_url.join("authorize"),
            issuer_url.join("token"),
            issuer_url.join("jwks"),
        ))
    }
}

/// Static attestation data shared across all instances of an attestation type. The issuer augments this with an
/// [`IssuableDocument`] to form the attestation.
#[derive(Debug)]
pub struct AttestationTypeConfig<K> {
    #[debug(skip)]
    pub key_pair: KeyPair<K>,
    pub valid_days: Days,
    pub copies_per_format: IndexMap<Format, NonZeroU8>,
    pub issuer_uri: HttpsUri,
    pub status_list_base_url: BaseUrl,
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
        copies_per_format: IndexMap<Format, NonZeroU8>,
        issuer_uri: HttpsUri,
        status_list_base_url: BaseUrl,
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
            status_list_base_url,
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

pub struct Issuer<A, K, S, C> {
    sessions: Arc<S>,
    attr_service: A,
    issuer_data: IssuerData<K>,
    sessions_cleanup_task: JoinHandle<()>,
    status_claim_service: C,
    pub metadata: IssuerMetadata,
}

/// Fields of the [`Issuer`] needed by the issuance functions.
pub struct IssuerData<K> {
    attestation_config: AttestationTypesConfig<K>,
    wua_config: Option<WuaConfig>,

    /// URL identifying the issuer; should host ` /.well-known/openid-credential-issuer`,
    /// and MUST be used by the wallet as `aud` in its PoP JWTs.
    credential_issuer_identifier: BaseUrl,

    /// Wallet IDs accepted by this server, MUST be used by the wallet as `iss` in its PoP JWTs.
    accepted_wallet_client_ids: Vec<String>,

    /// URL prefix of the `/token`, `/credential` and `/batch_crededential` endpoints.
    server_url: BaseUrl,
}

pub struct WuaConfig {
    /// Public key of the WUA issuer.
    pub wua_issuer_pubkey: EcdsaDecodingKey,
}

impl<A, K, S, C> Drop for Issuer<A, K, S, C> {
    fn drop(&mut self) {
        // Stop the tasks at the next .await
        self.sessions_cleanup_task.abort();
    }
}

impl<A, K, S, C> Issuer<A, K, S, C>
where
    A: AttributeService,
    K: EcdsaKeySend,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
    C: StatusClaimService,
{
    #[expect(clippy::too_many_arguments, reason = "Constructor")]
    pub fn new(
        sessions: Arc<S>,
        attr_service: A,
        attestation_config: AttestationTypesConfig<K>,
        server_url: &BaseUrl,
        wallet_client_ids: Vec<String>,
        wua_config: Option<WuaConfig>,
        status_claim_service: C,
    ) -> Self {
        let credential_configurations_supported = attestation_config
            .as_ref()
            .iter()
            .map(|(typ, attestation)| {
                (
                    typ.to_string(),
                    CredentialMetadata::from_sd_jwt_vc_type_metadata(&attestation.metadata),
                )
            })
            .collect();

        let issuer_url = server_url.join_base_url("issuance/");
        let issuer_data = IssuerData {
            attestation_config,
            credential_issuer_identifier: issuer_url.clone(),
            accepted_wallet_client_ids: wallet_client_ids,
            wua_config,

            // In this implementation, for now the Credential Issuer Identifier also always acts as
            // the public server URL.
            server_url: issuer_url.clone(),
        };

        Self {
            sessions: Arc::clone(&sessions),
            attr_service,
            issuer_data,
            status_claim_service,
            sessions_cleanup_task: sessions.start_cleanup_task(CLEANUP_INTERVAL_SECONDS),
            metadata: IssuerMetadata {
                issuer_config: metadata::IssuerData {
                    credential_issuer: issuer_url.clone(),
                    authorization_servers: None,
                    credential_endpoint: issuer_url.join_base_url("/credential"),
                    batch_credential_endpoint: Some(issuer_url.join_base_url("/batch_credential")),
                    deferred_credential_endpoint: None,
                    notification_endpoint: None,
                    credential_response_encryption: CredentialResponseEncryption {
                        alg_values_supported: vec![],
                        enc_values_supported: vec![],
                        encryption_required: false,
                    },
                    credential_identifiers_supported: Some(false),
                    display: None,
                    credential_configurations_supported,
                },
                protected_metadata: None,
            },
        }
    }
}

fn logged_issuance_result<T, E: std::error::Error>(result: Result<T, E>) -> Result<T, E> {
    result
        .inspect(|_| info!("Issuance success"))
        .inspect_err(|error| info!("Issuance error: {error}"))
}

impl<A, K, S, C> Issuer<A, K, S, C>
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
}

impl<A, K, S, C> Issuer<A, K, S, C>
where
    A: AttributeService,
    K: EcdsaKeySend,
    S: SessionStore<IssuanceData>,
    C: StatusClaimService,
{
    pub async fn process_token_request(
        &self,
        token_request: TokenRequest,
        dpop: Dpop,
    ) -> Result<(TokenResponseWithPreviews, String), TokenRequestError> {
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
                &self.issuer_data.credential_issuer_identifier,
                &self.issuer_data.attestation_config,
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
                &self.status_claim_service,
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
                &self.status_claim_service,
            )
            .await;

        self.sessions
            .write(next.into(), false)
            .await
            .map_err(IssuanceError::SessionStore)?;

        logged_issuance_result(response)
    }

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
            &self.issuer_data.credential_issuer_identifier.join(endpoint_name),
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

    pub async fn oauth_metadata(&self) -> Result<oidc::Config, A::Error> {
        self.attr_service
            .oauth_metadata(&self.issuer_data.credential_issuer_identifier)
            .await
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
    pub async fn process_token_request(
        self,
        token_request: TokenRequest,
        accepted_wallet_client_ids: &[String],
        dpop: Dpop,
        attr_service: &impl AttributeService,
        server_url: &BaseUrl,
        attestation_settings: &AttestationTypesConfig<impl EcdsaKeySend>,
    ) -> Result<(TokenResponseWithPreviews, String, Session<WaitingForResponse>), (TokenRequestError, Session<Done>)>
    {
        let result = self
            .process_token_request_inner(token_request, dpop, attr_service, server_url, attestation_settings)
            .await;

        match result {
            Ok((response, ids, dpop_pubkey, dpop_nonce)) => {
                let next = self.transition(WaitingForResponse {
                    access_token: response.token_response.access_token.clone(),
                    c_nonce: response.token_response.c_nonce.as_ref().unwrap().clone(), // field is always set below
                    accepted_wallet_client_ids: accepted_wallet_client_ids.to_vec(),
                    credential_previews: response
                        .credential_previews
                        .clone()
                        .into_iter()
                        // ids are unzipped from token_request issuable_documents which are transformed into previews
                        .zip_eq(ids)
                        .map(|(preview, id)| CredentialPreviewState::from(preview, id))
                        .collect(),
                    dpop_public_key: dpop_pubkey,
                    dpop_nonce: dpop_nonce.clone(),
                });
                Ok((response, dpop_nonce, next))
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

    pub async fn process_token_request_inner(
        &self,
        token_request: TokenRequest,
        dpop: Dpop,
        attr_service: &impl AttributeService,
        server_url: &BaseUrl,
        attestation_settings: &AttestationTypesConfig<impl EcdsaKeySend>,
    ) -> Result<(TokenResponseWithPreviews, VecNonEmpty<Uuid>, VerifyingKey, String), TokenRequestError> {
        if !matches!(
            token_request.grant_type,
            TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code: _ }
        ) {
            return Err(TokenRequestError::UnsupportedTokenRequestType);
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

        let c_nonce = random_string(32);
        let dpop_nonce = random_string(32);

        let response = TokenResponseWithPreviews {
            token_response: TokenResponse::new(AccessToken::new(&code), c_nonce),
            credential_previews: previews,
        };

        Ok((response, ids, dpop_public_key, dpop_nonce))
    }
}

impl TokenResponse {
    pub(crate) fn new(access_token: AccessToken, c_nonce: String) -> Self {
        Self {
            access_token,
            c_nonce: Some(c_nonce),
            token_type: TokenType::DPoP,
            expires_in: None,
            refresh_token: None,
            scope: None,
            c_nonce_expires_in: None,
            authorization_details: None,
        }
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

impl Session<WaitingForResponse> {
    pub async fn process_credential(
        self,
        credential_request: CredentialRequest,
        access_token: AccessToken,
        dpop: Dpop,
        issuer_data: &IssuerData<impl EcdsaKeySend>,
        status_claim_service: &impl StatusClaimService,
    ) -> (Result<CredentialResponse, CredentialRequestError>, Session<Done>) {
        let result = self
            .process_credential_inner(
                credential_request,
                access_token,
                dpop,
                issuer_data,
                status_claim_service,
            )
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
        endpoint: &str,
        issuer_data: &IssuerData<impl EcdsaKeySend>,
    ) -> Result<(), CredentialRequestError> {
        let session_data = self.session_data();

        // Check authorization of the request
        if session_data.access_token != *access_token {
            return Err(CredentialRequestError::Unauthorized);
        }

        // Check that the DPoP is valid and its key matches the one from the Token Request
        dpop.verify_expecting_key(
            &session_data.dpop_public_key,
            &issuer_data.server_url.join(endpoint),
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
    ) -> Result<VerifyingKey, CredentialRequestError> {
        let wua_disclosure = attestations.ok_or(CredentialRequestError::MissingWua)?;

        let wua_pubkey = wua_disclosure.verify(
            &wua_config.wua_issuer_pubkey,
            issuer_identifier,
            &self.state.data.accepted_wallet_client_ids,
            &self.state.data.c_nonce,
        )?;

        Ok(wua_pubkey)
    }

    pub fn verify_wua_and_poa(
        &self,
        attestations: Option<&WuaDisclosure>,
        poa: Option<Poa>,
        attestation_keys: impl Iterator<Item = VerifyingKey>,
        issuer_data: &IssuerData<impl EcdsaKeySend>,
    ) -> Result<(), CredentialRequestError> {
        let issuer_identifier = issuer_data.credential_issuer_identifier.as_ref().as_str();

        let attestation_keys = match &issuer_data.wua_config {
            None => attestation_keys.collect_vec(),
            Some(wua) => {
                let wua_pubkey = self.verify_wua(wua, attestations, issuer_identifier)?;
                attestation_keys.chain([wua_pubkey]).collect_vec()
            }
        };

        poa.ok_or(CredentialRequestError::MissingPoa)?.verify(
            &attestation_keys,
            issuer_identifier,
            &issuer_data.accepted_wallet_client_ids,
            &self.state.data.c_nonce,
        )?;

        Ok(())
    }

    pub async fn process_credential_inner(
        &self,
        credential_request: CredentialRequest,
        access_token: AccessToken,
        dpop: Dpop,
        issuer_data: &IssuerData<impl EcdsaKeySend>,
        status_claim_service: &impl StatusClaimService,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        let session_data = self.session_data();

        self.check_credential_endpoint_access(&access_token, dpop, "credential", issuer_data)?;

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

        let holder_pubkey = credential_request.verify(&session_data.c_nonce, issuer_data)?;

        self.verify_wua_and_poa(
            credential_request.attestations.as_ref(),
            credential_request.poa,
            [holder_pubkey].into_iter(),
            issuer_data,
        )?;

        let attestation_type = &preview.credential_payload.attestation_type;
        let config = issuer_data
            .attestation_config
            .as_ref()
            .get(attestation_type)
            .ok_or_else(|| CredentialRequestError::MissingAttestationTypeConfiguration(attestation_type.to_owned()))?;

        let status_claim = status_claim_service
            .obtain_status_claims(
                &preview.credential_payload.attestation_type,
                preview.batch_id,
                config.status_list_base_url.clone(),
                preview.credential_payload.expires,
                1.try_into().unwrap(),
            )
            .await
            .map_err(|err| CredentialRequestError::StatusClaimService(Box::new(err)))?
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

    pub async fn process_batch_credential(
        self,
        credential_requests: CredentialRequests,
        access_token: AccessToken,
        dpop: Dpop,
        issuer_data: &IssuerData<impl EcdsaKeySend>,
        status_claim_service: &impl StatusClaimService,
    ) -> (Result<CredentialResponses, CredentialRequestError>, Session<Done>) {
        let result = self
            .process_batch_credential_inner(
                credential_requests,
                access_token,
                dpop,
                issuer_data,
                status_claim_service,
            )
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

    async fn process_batch_credential_inner(
        &self,
        credential_requests: CredentialRequests,
        access_token: AccessToken,
        dpop: Dpop,
        issuer_data: &IssuerData<impl EcdsaKeySend>,
        status_claim_service: &impl StatusClaimService,
    ) -> Result<CredentialResponses, CredentialRequestError> {
        let session_data = self.session_data();

        self.check_credential_endpoint_access(&access_token, dpop, "batch_credential", issuer_data)?;

        let mut cred_req_index = 0;
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
                            .get(cred_req_index)
                            .ok_or(CredentialRequestError::WrongNumberOfCredentialRequests)?;
                        cred_req_index += 1;

                        // Verify the assumption that the order of the incoming requests matches exactly
                        // that of the flattened copies_per_format by matching the requested format.
                        if format != cred_req.credential_type.as_ref().format() {
                            return Err(CredentialRequestError::CredentialTypeMismatch {
                                offered: format,
                                requested: cred_req.credential_type.as_ref().format(),
                            });
                        }
                        let key = cred_req.verify(&session_data.c_nonce, issuer_data)?;
                        Ok((format, key))
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .unwrap();

                let attestation_type = &preview.credential_payload.attestation_type;
                let config = issuer_data
                    .attestation_config
                    .as_ref()
                    .get(attestation_type)
                    .ok_or_else(|| {
                        CredentialRequestError::MissingAttestationTypeConfiguration(attestation_type.to_string())
                    })?;

                // copies_per_format has a NonZeroU8 value in AttestationConfig (source)
                Ok((preview, config, format_pubkeys))
            })
            .collect::<Result<Vec<_>, CredentialRequestError>>()?;

        // Verify that we have consumed all credential requests
        if cred_req_index != credential_requests.credential_requests.as_ref().len() {
            return Err(CredentialRequestError::WrongNumberOfCredentialRequests);
        }

        self.verify_wua_and_poa(
            credential_requests.attestations.as_ref(),
            credential_requests.poa,
            previews_and_holder_pubkeys
                .iter()
                .flat_map(|(_, _, format_pubkeys)| format_pubkeys.iter().map(|(_, key)| *key)),
            issuer_data,
        )?;

        // Obtain a status claim for every attestation copy, linked to a single batch id per preview
        let status_claims = try_join_all(previews_and_holder_pubkeys.iter().map(
            |(preview, config, format_pubkeys)| async move {
                let claims = status_claim_service
                    .obtain_status_claims(
                        &preview.credential_payload.attestation_type,
                        preview.batch_id,
                        config.status_list_base_url.clone(),
                        preview.credential_payload.expires,
                        format_pubkeys.len(),
                    )
                    .await
                    .map_err(|err| CredentialRequestError::StatusClaimService(Box::new(err)))?;
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
                .zip_eq(status_claims.into_iter())
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
        c_nonce: &str,
        issuer_data: &IssuerData<impl EcdsaKeySend>,
    ) -> Result<VerifyingKey, CredentialRequestError> {
        let holder_pubkey = self
            .proof
            .as_ref()
            .ok_or(CredentialRequestError::MissingCredentialRequestPoP)?
            .verify(
                c_nonce,
                &issuer_data.accepted_wallet_client_ids,
                &issuer_data.credential_issuer_identifier,
            )?;

        Ok(holder_pubkey)
    }
}

impl CredentialResponse {
    async fn new(
        credential_format: Format,
        preview_credential_payload: PreviewableCredentialPayload,
        issued_at: DateTime<Utc>,
        holder_pubkey: &VerifyingKey,
        attestation_config: &AttestationTypeConfig<impl EcdsaKeySend>,
        _status_claim: StatusClaim,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        let payload = CredentialPayload::from_previewable_credential_payload(
            preview_credential_payload,
            issued_at,
            holder_pubkey,
            &attestation_config.metadata,
            attestation_config.first_metadata_integrity.clone(),
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

        Ok(CredentialResponse::MsoMdoc {
            credential: Box::new(issuer_signed),
        })
    }

    async fn new_for_sd_jwt(
        credential_payload: CredentialPayload,
        attestation_config: &AttestationTypeConfig<impl EcdsaKeySend + Sized>,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        let signed_sd_jwt = credential_payload
            .into_signed_sd_jwt(&attestation_config.metadata, &attestation_config.key_pair)
            .await?;

        Ok(CredentialResponse::SdJwt {
            credential: signed_sd_jwt.into_unverified(),
        })
    }
}

impl CredentialRequestProof {
    pub fn verify(
        &self,
        nonce: &str,
        accepted_wallet_client_ids: &[impl ToString],
        credential_issuer_identifier: &BaseUrl,
    ) -> Result<VerifyingKey, CredentialRequestError> {
        let CredentialRequestProof::Jwt { jwt } = self;

        let mut validation_options = Validation::new(Algorithm::ES256);
        validation_options.set_required_spec_claims(&["iss", "aud"]);
        validation_options.set_issuer(accepted_wallet_client_ids);
        validation_options.set_audience(&[credential_issuer_identifier]);

        let (header, payload) = jwt
            .parse_and_verify_with_jwk(&validation_options)
            .map_err(CredentialRequestError::UnsupportedJwt)?;

        if payload.nonce.as_deref() != Some(nonce) {
            return Err(CredentialRequestError::IncorrectNonce);
        }

        Ok(header.verifying_key()?)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Timelike;
    use derive_more::Debug;
    use thiserror::Error;
    use tracing_test::traced_test;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::x509::generate::mock::generate_issuer_mock_with_registration;
    use crypto::server_keys::generate::Ca;

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
            IndexMap::from_iter([(Format::MsoMdoc, NonZeroU8::new(1).unwrap())]),
            "https://example.com".parse().unwrap(),
            "https://cdn.example.com/tsl".parse().unwrap(),
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
}
