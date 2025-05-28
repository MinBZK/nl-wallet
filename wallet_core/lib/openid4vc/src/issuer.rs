use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::Infallible;
use std::num::NonZeroU8;
use std::ops::Add;
use std::sync::Arc;
use std::sync::LazyLock;

use chrono::Days;
use chrono::DurationRound;
use chrono::Utc;
use derive_more::AsRef;
use derive_more::Debug;
use derive_more::From;
use futures::future::try_join_all;
use indexmap::IndexMap;
use itertools::Itertools;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Validation;
use p256::ecdsa::VerifyingKey;
use reqwest::Method;
use serde::Deserialize;
use serde::Serialize;
use ssri::Integrity;
use tokio::task::JoinHandle;
use tracing::info;

use attestation_data::credential_payload::CredentialPayload;
use attestation_data::credential_payload::IntoCredentialPayload;
use attestation_data::credential_payload::SdJwtCredentialPayloadError;
use attestation_data::issuable_document::IssuableDocument;
use attestation_data::issuable_document::IssuableDocuments;
use attestation_data::qualification::AttestationQualification;
use crypto::server_keys::KeyPair;
use crypto::utils::random_string;
use crypto::EcdsaKeySend;
use http_utils::urls::BaseUrl;
use http_utils::urls::HttpsUri;
use jwt::credential::JwtCredentialClaims;
use jwt::error::JwkConversionError;
use jwt::error::JwtError;
use jwt::jwk::jwk_to_p256;
use jwt::pop::JwtPopClaims;
use jwt::validations;
use jwt::wte::WteClaims;
use jwt::EcdsaDecodingKey;
use jwt::VerifiedJwt;
use mdoc::holder::MdocCredentialPayloadError;
use mdoc::holder::MdocParts;
use mdoc::unsigned::UnsignedMdoc;
use mdoc::IssuerSigned;
use poa::Poa;
use poa::PoaVerificationError;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataChainError;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use utils::vec_at_least::VecNonEmpty;

use crate::credential::CredentialRequest;
use crate::credential::CredentialRequestProof;
use crate::credential::CredentialRequests;
use crate::credential::CredentialResponse;
use crate::credential::CredentialResponses;
use crate::credential::WteDisclosure;
use crate::credential::OPENID4VCI_VC_POP_JWT_TYPE;
use crate::dpop::Dpop;
use crate::dpop::DpopError;
use crate::metadata;
use crate::metadata::CredentialMetadata;
use crate::metadata::CredentialResponseEncryption;
use crate::metadata::IssuerMetadata;
use crate::oidc;
use crate::server_state::Expirable;
use crate::server_state::HasProgress;
use crate::server_state::Progress;
use crate::server_state::SessionDataType;
use crate::server_state::SessionState;
use crate::server_state::SessionStore;
use crate::server_state::SessionStoreError;
use crate::server_state::SessionToken;
use crate::server_state::WteTracker;
use crate::server_state::CLEANUP_INTERVAL_SECONDS;
use crate::token::AccessToken;
use crate::token::AuthorizationCode;
use crate::token::CredentialPreview;
use crate::token::CredentialPreviewContent;
use crate::token::TokenRequest;
use crate::token::TokenRequestGrantType;
use crate::token::TokenResponse;
use crate::token::TokenResponseWithPreviews;
use crate::token::TokenType;
use crate::Format;

// Errors are structured as follow in this module: the handler for a token request on the one hand, and the handlers for
// the other endpoints on the other hand, have specific error types. (There is also a general error type included by
// both of them for errors that can occur in all endpoints.) The reason for this split in the errors is because per the
// OpenID4VCI and OAuth specs, these endpoints each have to return error codes that are specific to them, i.e., the
// token request endpoint can return error codes that the credential endpoint can't and vice versa, so we want to keep
// the errors separate in the type system here.

/// Errors that can occur during processing of any of the endpoints.
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

    #[error("missing JWK")]
    MissingJwk,

    #[error("incorrect nonce")]
    IncorrectNonce,

    #[error(
        "unsupported JWT algorithm: expected {}, found {}",
        expected,
        found.as_ref().unwrap_or(&"<None>".to_string())
    )]
    UnsupportedJwtAlgorithm { expected: String, found: Option<String> },

    #[error("JWT decoding failed: {0}")]
    JwtDecodingFailed(#[from] jsonwebtoken::errors::Error),

    #[error("JWK conversion error: {0}")]
    JwkConversion(#[from] JwkConversionError),

    #[error("JWT error: {0}")]
    Jwt(#[from] JwtError),

    #[error("missing issuance private key for doctype {0}")]
    MissingPrivateKey(String),

    #[error("failed to sign credential: {0}")]
    CredentialSigning(mdoc::Error),

    #[error("mismatch between rquested and offered doctypes")]
    CredentialTypeMismatch,

    #[error("missing credential request proof of possession")]
    MissingCredentialRequestPoP,

    #[error("missing WTE")]
    MissingWte,

    #[error("error checking WTE usage status: {0}")]
    WteTracking(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("WTE has already been used")]
    WteAlreadyUsed,

    #[error("missing PoA")]
    MissingPoa,

    #[error("error verifying PoA: {0}")]
    PoaVerification(#[from] PoaVerificationError),

    #[error("error converting CredentialPayload to Mdoc: {0}")]
    MdocConversion(#[from] MdocCredentialPayloadError),

    #[error("error converting CredentialPayload to SD-JWT: {0}")]
    SdJwtConversion(#[from] SdJwtCredentialPayloadError),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Created {
    pub issuable_documents: Option<VecNonEmpty<IssuableDocument>>,
}

// TODO (PVW-4109): Once the unsigned mdoc header no longer embeds the type metadata, the state stored for the
//                  credentials can be reduced to the `CredentialPayload`s and the copies per format. Storing
//                  the entire preview is unnecessary, as this also includes the type metadata documents and
//                  the issuer certificate. Both of these are already present in the issuer configuration.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WaitingForResponse {
    pub access_token: AccessToken,
    pub c_nonce: String,
    pub accepted_wallet_client_ids: Vec<String>,
    pub credential_previews: Vec<CredentialPreview>,
    pub dpop_public_key: VerifyingKey,
    pub dpop_nonce: String,
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
    pub attestation_qualification: AttestationQualification,
    pub metadata_documents: TypeMetadataDocuments,
    first_metadata_integrity: Integrity,
    metadata: NormalizedTypeMetadata,
}

impl<K> AttestationTypeConfig<K> {
    /// Create a new [`AttestationTypeConfig`] and decode and validate the type metadata documents.
    pub fn try_new(
        attestation_type: &str,
        key_pair: KeyPair<K>,
        valid_days: Days,
        copies_per_format: IndexMap<Format, NonZeroU8>,
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

pub struct Issuer<A, K, S, W> {
    sessions: Arc<S>,
    attr_service: A,
    issuer_data: IssuerData<K, W>,
    sessions_cleanup_task: JoinHandle<()>,
    wte_cleanup_task: Option<JoinHandle<()>>,
    pub metadata: IssuerMetadata,
}

/// Fields of the [`Issuer`] needed by the issuance functions.
pub struct IssuerData<K, W> {
    attestation_config: AttestationTypesConfig<K>,
    wte_config: Option<WteConfig<W>>,

    /// URL identifying the issuer; should host ` /.well-known/openid-credential-issuer`,
    /// and MUST be used by the wallet as `aud` in its PoP JWTs.
    credential_issuer_identifier: BaseUrl,

    /// Wallet IDs accepted by this server, MUST be used by the wallet as `iss` in its PoP JWTs.
    accepted_wallet_client_ids: Vec<String>,

    /// URL prefix of the `/token`, `/credential` and `/batch_crededential` endpoints.
    server_url: BaseUrl,
}

pub struct WteConfig<W> {
    /// Public key of the WTE issuer.
    pub wte_issuer_pubkey: EcdsaDecodingKey,

    /// Tracks recently seen WTEs.
    pub wte_tracker: Arc<W>,
}

impl<A, K, S, W> Drop for Issuer<A, K, S, W> {
    fn drop(&mut self) {
        // Stop the tasks at the next .await
        self.sessions_cleanup_task.abort();
        if let Some(ref wte_cleanup_task) = self.wte_cleanup_task {
            wte_cleanup_task.abort();
        }
    }
}

impl<A, K, S, W> Issuer<A, K, S, W>
where
    A: AttributeService,
    K: EcdsaKeySend,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
    W: WteTracker + Send + Sync + 'static,
{
    pub fn new(
        sessions: Arc<S>,
        attr_service: A,
        attestation_config: AttestationTypesConfig<K>,
        server_url: &BaseUrl,
        wallet_client_ids: Vec<String>,
        wte_config: Option<WteConfig<W>>,
    ) -> Self {
        let wte_tracker = wte_config
            .as_ref()
            .map(|wte_config| Arc::clone(&wte_config.wte_tracker));

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
            wte_config,

            // In this implementation, for now the Credential Issuer Identifier also always acts as
            // the public server URL.
            server_url: issuer_url.clone(),
        };

        Self {
            sessions: Arc::clone(&sessions),
            attr_service,
            issuer_data,
            sessions_cleanup_task: sessions.start_cleanup_task(CLEANUP_INTERVAL_SECONDS),
            wte_cleanup_task: wte_tracker.map(|wte_tracker| wte_tracker.start_cleanup_task(CLEANUP_INTERVAL_SECONDS)),
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

impl<A, K, S, W> Issuer<A, K, S, W>
where
    S: SessionStore<IssuanceData>,
{
    pub async fn new_session(&self, to_issue: IssuableDocuments) -> Result<SessionToken, SessionStoreError> {
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

impl<A, K, S, W> Issuer<A, K, S, W>
where
    A: AttributeService,
    K: EcdsaKeySend,
    S: SessionStore<IssuanceData>,
    W: WteTracker,
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
            .process_credential(credential_request, access_token, dpop, &self.issuer_data)
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
            .process_batch_credential(credential_requests, access_token, dpop, &self.issuer_data)
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

impl Session<Created> {
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
            Ok((response, dpop_pubkey, dpop_nonce)) => {
                let next = self.transition(WaitingForResponse {
                    access_token: response.token_response.access_token.clone(),
                    c_nonce: response.token_response.c_nonce.as_ref().unwrap().clone(), // field is always set below
                    accepted_wallet_client_ids: accepted_wallet_client_ids.to_vec(),
                    credential_previews: response.credential_previews.clone().into_inner(),
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

    fn credential_preview_from_issuable_document(
        document: IssuableDocument,
        attestation_data: &AttestationTypeConfig<impl EcdsaKeySend>,
    ) -> CredentialPreview {
        // Truncate the current time to only include the date part, so that all issued credentials on a single
        // day day have the same `iat` and `exp` field
        let now = Utc::now()
            .duration_trunc(chrono::Duration::days(1))
            .expect("should never exceed Unix time bounds");
        let valid_until = now.add(attestation_data.valid_days);

        let credential_payload = document.into_previewable_credential_payload(
            now,
            valid_until,
            attestation_data.issuer_uri.clone(),
            attestation_data.attestation_qualification,
        );

        CredentialPreview {
            content: CredentialPreviewContent {
                copies_per_format: attestation_data.copies_per_format.clone(),
                credential_payload,
                issuer_certificate: attestation_data.key_pair.certificate().clone(),
            },
            type_metadata: attestation_data.metadata_documents.clone(),
        }
    }

    pub async fn process_token_request_inner(
        &self,
        token_request: TokenRequest,
        dpop: Dpop,
        attr_service: &impl AttributeService,
        server_url: &BaseUrl,
        attestation_settings: &AttestationTypesConfig<impl EcdsaKeySend>,
    ) -> Result<(TokenResponseWithPreviews, VerifyingKey, String), TokenRequestError> {
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

        let previews = issuables
            .into_inner()
            .into_iter()
            .map(|document| {
                let attestation_data = attestation_settings
                    .as_ref()
                    .get(document.attestation_type())
                    .ok_or_else(|| {
                        TokenRequestError::CredentialTypeNotOffered(document.attestation_type().to_string())
                    })?;

                let preview = Self::credential_preview_from_issuable_document(document, attestation_data);

                Ok(preview)
            })
            .collect::<Result<Vec<_>, TokenRequestError>>()?
            .try_into()
            .unwrap();

        let c_nonce = random_string(32);
        let dpop_nonce = random_string(32);

        let response = TokenResponseWithPreviews {
            token_response: TokenResponse::new(AccessToken::new(&code), c_nonce),
            credential_previews: previews,
        };

        Ok((response, dpop_public_key, dpop_nonce))
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
        issuer_data: &IssuerData<impl EcdsaKeySend, impl WteTracker>,
    ) -> (Result<CredentialResponse, CredentialRequestError>, Session<Done>) {
        let result = self
            .process_credential_inner(credential_request, access_token, dpop, issuer_data)
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
        dpop: &Dpop,
        endpoint: &str,
        issuer_data: &IssuerData<impl EcdsaKeySend, impl WteTracker>,
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

    async fn verify_wte(
        &self,
        wte_config: &WteConfig<impl WteTracker>,
        attestations: Option<WteDisclosure>,
        issuer_identifier: &str,
    ) -> Result<VerifyingKey, CredentialRequestError> {
        let wte_disclosure = attestations.ok_or(CredentialRequestError::MissingWte)?;

        let (verified_wte, wte_pubkey) = wte_disclosure.verify(
            &wte_config.wte_issuer_pubkey,
            issuer_identifier,
            &self.state.data.accepted_wallet_client_ids,
            &self.state.data.c_nonce,
        )?;

        // Check that the WTE is fresh
        if wte_config
            .wte_tracker
            .track_wte(&verified_wte)
            .await
            .map_err(|err| CredentialRequestError::WteTracking(Box::new(err)))?
        {
            return Err(CredentialRequestError::WteAlreadyUsed);
        }

        Ok(wte_pubkey)
    }

    pub async fn verify_wte_and_poa(
        &self,
        attestations: Option<WteDisclosure>,
        poa: Option<Poa>,
        attestation_keys: impl Iterator<Item = VerifyingKey>,
        issuer_data: &IssuerData<impl EcdsaKeySend, impl WteTracker>,
    ) -> Result<(), CredentialRequestError> {
        let issuer_identifier = issuer_data.credential_issuer_identifier.as_ref().as_str();

        let attestation_keys = match &issuer_data.wte_config {
            None => attestation_keys.collect_vec(),
            Some(wte) => {
                let wte_pubkey = self.verify_wte(wte, attestations, issuer_identifier).await?;
                attestation_keys.chain([wte_pubkey]).collect_vec()
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
        issuer_data: &IssuerData<impl EcdsaKeySend, impl WteTracker>,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        let session_data = self.session_data();

        self.check_credential_endpoint_access(&access_token, &dpop, "credential", issuer_data)?;

        // If we have exactly one credential on offer that matches the credential type that the client is
        // requesting, then we issue that credential.
        // NB: the OpenID4VCI specification leaves open how to make this decision, this is our own behaviour.
        let requested_format = credential_request.credential_type.as_ref().format();
        let offered_creds = session_data
            .credential_previews
            .iter()
            .filter(|preview| preview.content.copies_per_format.contains_key(&requested_format))
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

        self.verify_wte_and_poa(
            credential_request.attestations,
            credential_request.poa,
            [holder_pubkey].into_iter(),
            issuer_data,
        )
        .await?;

        let credential_response =
            CredentialResponse::new(requested_format, preview.clone(), holder_pubkey, issuer_data).await?;

        Ok(credential_response)
    }

    pub async fn process_batch_credential(
        self,
        credential_requests: CredentialRequests,
        access_token: AccessToken,
        dpop: Dpop,
        issuer_data: &IssuerData<impl EcdsaKeySend, impl WteTracker>,
    ) -> (Result<CredentialResponses, CredentialRequestError>, Session<Done>) {
        let result = self
            .process_batch_credential_inner(credential_requests, access_token, dpop, issuer_data)
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
        issuer_data: &IssuerData<impl EcdsaKeySend, impl WteTracker>,
    ) -> Result<CredentialResponses, CredentialRequestError> {
        let session_data = self.session_data();

        self.check_credential_endpoint_access(&access_token, &dpop, "batch_credential", issuer_data)?;

        let previews_and_holder_pubkeys =
            credential_requests
                .credential_requests
                .iter()
                .zip(session_data.credential_previews.iter().flat_map(|preview| {
                    preview.content.copies_per_format.iter().flat_map(|(format, copies)| {
                        itertools::repeat_n((*format, preview.clone()), copies.get().into())
                    })
                }))
                .map(|(cred_req, (format, preview))| {
                    // Verify the assumption that the order of the incoming requests matches exactly
                    // that of the flattened copies_per_format by matching the requested format.
                    if format != cred_req.credential_type.as_ref().format() {
                        return Err(CredentialRequestError::CredentialTypeMismatch);
                    }

                    let key = cred_req.verify(&session_data.c_nonce, issuer_data)?;

                    Ok((preview, format, key))
                })
                .collect::<Result<Vec<_>, CredentialRequestError>>()?;

        self.verify_wte_and_poa(
            credential_requests.attestations,
            credential_requests.poa,
            previews_and_holder_pubkeys.iter().map(|(_, _, key)| *key),
            issuer_data,
        )
        .await?;

        let credential_responses = try_join_all(
            previews_and_holder_pubkeys
                .into_iter()
                .map(|(preview, format, key)| CredentialResponse::new(format, preview, key, issuer_data)),
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

impl CredentialPreview {
    /// Returns an identifier for the issuer private key with which this credential is to be issued.
    /// The issuer will need to have a private key under this identifier in its [`KeyRing`].
    fn issuer_key_identifier(&self) -> &str {
        self.content.credential_payload.attestation_type.as_str()
    }
}

impl CredentialRequest {
    fn verify(
        &self,
        c_nonce: &str,
        issuer_data: &IssuerData<impl EcdsaKeySend, impl WteTracker>,
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
        preview: CredentialPreview,
        holder_pubkey: VerifyingKey,
        issuer_data: &IssuerData<impl EcdsaKeySend, impl WteTracker>,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        // Get the correct `AttestationTypeConfig` for this attestation type.
        let key_id = preview.issuer_key_identifier();
        let attestation_config = issuer_data
            .attestation_config
            .as_ref()
            .get(key_id)
            .ok_or(CredentialRequestError::MissingPrivateKey(key_id.to_string()))?;

        match credential_format {
            Format::MsoMdoc => Self::new_for_mdoc(preview, &holder_pubkey, attestation_config).await,
            Format::SdJwt => Self::new_for_sd_jwt(preview, &holder_pubkey, attestation_config).await,
            other => Err(CredentialRequestError::CredentialTypeNotOffered(other.to_string())),
        }
    }

    async fn new_for_mdoc(
        preview: CredentialPreview,
        holder_pubkey: &VerifyingKey,
        attestation_config: &AttestationTypeConfig<impl EcdsaKeySend + Sized>,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        // Construct an mdoc `IssuerSigned` from the contents of `PreviewableCredentialPayload`
        // and the attestation config by signing it.
        let unsigned_mdoc: UnsignedMdoc = preview.content.credential_payload.try_into()?;
        let attributes = unsigned_mdoc.attributes.clone().into_inner();

        let (issuer_signed, mso) = IssuerSigned::sign(
            unsigned_mdoc,
            attestation_config.first_metadata_integrity.clone(),
            holder_pubkey,
            &attestation_config.key_pair,
        )
        .await
        .map_err(CredentialRequestError::CredentialSigning)?;

        // As a last check, convert the `IssuerSigned` back to a full `CredentialPayload`
        // and validate it against the normalized metadata for this attestation.
        let _ = MdocParts::new(attributes, mso).into_credential_payload(&attestation_config.metadata)?;

        Ok(CredentialResponse::MsoMdoc {
            credential: Box::new(issuer_signed.into()),
        })
    }

    async fn new_for_sd_jwt(
        preview: CredentialPreview,
        holder_pubkey: &VerifyingKey,
        attestation_config: &AttestationTypeConfig<impl EcdsaKeySend + Sized>,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        let payload = CredentialPayload::from_previewable_credential_payload(
            preview.content.credential_payload,
            Utc::now().into(),
            holder_pubkey,
            &attestation_config.metadata,
        )?;

        let sd_jwt = payload
            .into_sd_jwt(
                &attestation_config.metadata,
                holder_pubkey,
                &attestation_config.key_pair,
            )
            .await?;

        Ok(CredentialResponse::SdJwt {
            credential: sd_jwt.to_string(),
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
        let jwt = match self {
            CredentialRequestProof::Jwt { jwt } => jwt,
        };
        let header = jsonwebtoken::decode_header(&jwt.0)?;
        let verifying_key = jwk_to_p256(&header.jwk.ok_or(CredentialRequestError::MissingJwk)?)?;

        let mut validation_options = Validation::new(Algorithm::ES256);
        validation_options.required_spec_claims = HashSet::default();
        validation_options.set_issuer(accepted_wallet_client_ids);
        validation_options.set_audience(&[credential_issuer_identifier]);

        // We use `jsonwebtoken` crate directly instead of our `Jwt` because we need to inspect the header
        let token_data = jsonwebtoken::decode::<JwtPopClaims>(
            &jwt.0,
            &EcdsaDecodingKey::from(&verifying_key).0,
            &validation_options,
        )?;

        if token_data.header.typ != Some(OPENID4VCI_VC_POP_JWT_TYPE.to_string()) {
            return Err(CredentialRequestError::UnsupportedJwtAlgorithm {
                expected: OPENID4VCI_VC_POP_JWT_TYPE.to_string(),
                found: token_data.header.typ,
            });
        }
        if token_data.claims.nonce.as_deref() != Some(nonce) {
            return Err(CredentialRequestError::IncorrectNonce);
        }

        Ok(verifying_key)
    }
}

// Returns the JWS validations for WTE verification.
//
// NOTE: the returned validation allows for no clock drift: time-based claims such as `exp` are validated
// without leeway. There must be no clock drift between the WTE issuer and the caller.
pub static WTE_JWT_VALIDATIONS: LazyLock<Validation> = LazyLock::new(|| {
    let mut validations = validations();
    validations.leeway = 0;

    // Enforce presence of exp, meaning it is also verified since `validations().validate_exp` is `true` by default.
    // Note that the PID issuer and the issuer of the WTE (the WP) have a mutual trust relationship with each other
    // in which they jointly ensure, through the WTE, that each wallet can obtain at most one PID. Therefore the PID
    // issuer, which runs this code, trusts the WP to set `exp` to a reasonable value (the `WTE_EXPIRY` constant).
    validations.set_required_spec_claims(&["exp"]);

    validations
});

impl WteDisclosure {
    fn verify(
        self,
        issuer_public_key: &EcdsaDecodingKey,
        expected_aud: &str,
        accepted_wallet_client_ids: &[String],
        expected_nonce: &str,
    ) -> Result<(VerifiedJwt<JwtCredentialClaims<WteClaims>>, VerifyingKey), CredentialRequestError> {
        let verified_jwt = VerifiedJwt::try_new(self.0, issuer_public_key, &WTE_JWT_VALIDATIONS)?;
        let wte_pubkey = jwk_to_p256(&verified_jwt.payload().confirmation.jwk)?;

        let mut validations = validations();
        validations.set_audience(&[expected_aud]);
        validations.set_issuer(accepted_wallet_client_ids);
        let wte_disclosure_claims = self.1.parse_and_verify(&(&wte_pubkey).into(), &validations)?;

        if wte_disclosure_claims.nonce.as_deref() != Some(expected_nonce) {
            return Err(CredentialRequestError::IncorrectNonce);
        }

        Ok((verified_jwt, wte_pubkey))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Timelike;
    use derive_more::Debug;
    use thiserror::Error;
    use tracing_test::traced_test;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::x509::generate::mock::generate_issuer_mock;
    use crypto::server_keys::generate::Ca;

    use super::*;

    #[derive(Debug, Error, Clone, Eq, PartialEq)]
    #[error("MyError")]
    struct MyError;

    #[test]
    fn test_credential_preview_from_issuable_document() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuance_keypair = generate_issuer_mock(&ca, IssuerRegistration::new_mock().into()).unwrap();
        let document = IssuableDocument::new_mock();
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
            AttestationQualification::default(),
            TypeMetadataDocuments::degree_example().1,
        )
        .unwrap();

        let preview = Session::<Created>::credential_preview_from_issuable_document(document, &config);
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
