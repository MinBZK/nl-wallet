use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use futures::future::try_join_all;
use jsonwebtoken::{Algorithm, Validation};
use p256::ecdsa::VerifyingKey;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tracing::info;

use nl_wallet_mdoc::{
    server_keys::KeyRing,
    unsigned::UnsignedMdoc,
    utils::{crypto::CryptoError, serialization::CborError},
    IssuerSigned,
};
use wallet_common::{config::wallet_config::BaseUrl, jwt::EcdsaDecodingKey, nonempty::NonEmpty, utils::random_string};

use crate::{
    credential::{
        CredentialRequest, CredentialRequestProof, CredentialRequestProofJwtPayload, CredentialRequests,
        CredentialResponse, CredentialResponses, OPENID4VCI_VC_POP_JWT_TYPE,
    },
    dpop::{Dpop, DpopError},
    jwt::{jwk_to_p256, JwkConversionError},
    metadata::{self, CredentialResponseEncryption, IssuerMetadata},
    oidc,
    server_state::{
        Expirable, HasProgress, Progress, SessionState, SessionStore, SessionStoreError, CLEANUP_INTERVAL_SECONDS,
    },
    token::{
        AccessToken, AttestationPreview, AuthorizationCode, TokenRequest, TokenRequestGrantType, TokenResponse,
        TokenResponseWithPreviews, TokenType,
    },
    Format,
};

/* Errors are structured as follow in this module: the handler for a token request on the one hand, and the handlers for
the other endpoints on the other hand, have specific error types. (There is also a general error type included by both
of them for errors that can occur in all endpoints.) The reason for this split in the errors is because per the
OpenID4VCI and OAuth specs, these endpoints each have to return error codes that are specific to them, i.e., the token
request endpoint can return error codes that the credential endpoint can't and vice versa, so we want to keep the errors
separate in the type system here. */

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
    #[error("doctype not offered")]
    DoctypeNotOffered(String),
    #[error("credential request ambiguous, use /batch_credential instead")]
    UseBatchIssuance,
    #[error("unsupported credential format: {0:?}")]
    UnsupportedCredentialFormat(Format),
    #[error("missing JWK")]
    MissingJwk,
    #[error("incorrect nonce")]
    IncorrectNonce,
    #[error("unsupported JWT algorithm: expected {}, found {}", expected, found.as_ref().unwrap_or(&"<None>".to_string()))]
    UnsupportedJwtAlgorithm { expected: String, found: Option<String> },
    #[error("JWT decoding failed: {0}")]
    JwtDecodingFailed(#[from] jsonwebtoken::errors::Error),
    #[error("JWK conversion error: {0}")]
    JwkConversion(#[from] JwkConversionError),
    #[error("failed to convert P256 public key to COSE key: {0}")]
    CoseKeyConversion(CryptoError),
    #[error("missing issuance private key for doctype {0}")]
    MissingPrivateKey(String),
    #[error("failed to sign attestation: {0}")]
    AttestationSigning(nl_wallet_mdoc::Error),
    #[error("CBOR error: {0}")]
    CborSerialization(#[from] CborError),
    #[error("JSON serialization failed: {0}")]
    JsonSerialization(#[from] serde_json::Error),
    #[error("mismatch between rquested and offered doctypes")]
    DoctypeMismatch,
    #[error("missing credential request proof of possession")]
    MissingCredentialRequestPoP,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Created {
    pub attestation_previews: Option<Vec<AttestationPreview>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WaitingForResponse {
    pub access_token: AccessToken,
    pub c_nonce: String,
    pub attestation_previews: Vec<AttestationPreview>,
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
    WaitingForResponse(WaitingForResponse),
    Done(Done),
}

impl HasProgress for IssuanceData {
    fn progress(&self) -> Progress {
        match self {
            Self::Created(_) | Self::WaitingForResponse(_) => Progress::Active,
            Self::Done(done) => Progress::Finished {
                has_succeeded: matches!(done.session_result, SessionResult::Done { .. }),
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
        })
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
///
/// A future implementation of this trait is expected to enable generic issuance of attributes as follows:
/// - The owner of the issuance server determines the attributes to be issued and sends those to the issuance server.
/// - The issuance server creates a new `SessionState<Created>` instance, puts that in its session store, and returns an
///   authorization code that is to be forwarded to the wallet.
/// - The wallet contacts the issuance server with the authorization code.
/// - The issuance server looks up the `SessionState<Created>` from its session store and feeds that to the future
///   implementation of this trait.
/// - That implementation of this trait returns the attributes to be issued out of the `SessionState<Created>`.
#[trait_variant::make(AttributeService: Send)]
pub trait LocalAttributeService {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn attributes(
        &self,
        session: &SessionState<Created>,
        token_request: TokenRequest,
    ) -> Result<NonEmpty<Vec<AttestationPreview>>, Self::Error>;

    async fn oauth_metadata(&self, issuer_url: &BaseUrl) -> Result<oidc::Config, Self::Error>;
}

pub struct Issuer<A, K, S> {
    sessions: Arc<S>,
    attr_service: A,
    issuer_data: IssuerData<K>,
    cleanup_task: JoinHandle<()>,
    pub metadata: IssuerMetadata,
}

/// Fields of the [`Issuer`] needed by the issuance functions.
pub struct IssuerData<K> {
    private_keys: K,

    /// URL identifying the issuer; should host ` /.well-known/openid-credential-issuer`,
    /// and MUST be used by the wallet as `aud` in its PoP JWTs.
    credential_issuer_identifier: BaseUrl,

    /// Wallet IDs accepted by this server, MUST be used by the wallet as `iss` in its PoP JWTs.
    accepted_wallet_client_ids: Vec<String>,

    /// URL prefix of the `/token`, `/credential` and `/batch_crededential` endpoints.
    server_url: BaseUrl,
}

impl<A, K, S> Drop for Issuer<A, K, S> {
    fn drop(&mut self) {
        // Stop the task at the next .await
        self.cleanup_task.abort();
    }
}

impl<A, K, S> Issuer<A, K, S>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
{
    pub fn new(
        sessions: S,
        attr_service: A,
        private_keys: K,
        server_url: &BaseUrl,
        wallet_client_ids: Vec<String>,
    ) -> Self {
        let sessions = Arc::new(sessions);

        let issuer_url = server_url.join_base_url("issuance/");
        let issuer_data = IssuerData {
            private_keys,
            credential_issuer_identifier: issuer_url.clone(),
            accepted_wallet_client_ids: wallet_client_ids,

            // In this implementation, for now the Credential Issuer Identifier also always acts as
            // the public server URL.
            server_url: issuer_url.clone(),
        };

        Self {
            sessions: Arc::clone(&sessions),
            attr_service,
            issuer_data,
            cleanup_task: sessions.start_cleanup_task(CLEANUP_INTERVAL_SECONDS),
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
                    credential_configurations_supported: HashMap::new(),
                },
                signed_metadata: None,
            },
        }
    }
}

fn logged_issuance_result<T, E: std::error::Error>(result: Result<T, E>) -> Result<T, E> {
    result
        .inspect(|_| info!("Issuance Success"))
        .inspect_err(|error| info!("Issuance Error: {error}"))
}

impl<A, K, S> Issuer<A, K, S>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<IssuanceData>,
{
    pub async fn process_token_request(
        &self,
        token_request: TokenRequest,
        dpop: Dpop,
    ) -> Result<(TokenResponseWithPreviews, String), TokenRequestError> {
        let session_token = token_request.code().clone().into();

        // Retrieve the session from the session store, if present. It need not be, depending on the implementation of the
        // attribute service.
        let session = self
            .sessions
            .get(&session_token)
            .await
            .map_err(IssuanceError::SessionStore)?
            .unwrap_or(SessionState::<IssuanceData>::new(
                session_token,
                IssuanceData::Created(Created {
                    attestation_previews: None,
                }),
            ));
        let session: Session<Created> = session.try_into().map_err(TokenRequestError::IssuanceError)?;

        let result = session
            .process_token_request(
                token_request,
                dpop,
                &self.attr_service,
                &self.issuer_data.credential_issuer_identifier,
            )
            .await;

        let (response, next) = match result {
            Ok((response, dpop_nonce, next)) => (Ok((response, dpop_nonce)), next.into()),
            Err((err, next)) => (Err(err), next.into()),
        };

        self.sessions
            .write(next, true)
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
        dpop: Dpop,
        attr_service: &impl AttributeService,
        server_url: &BaseUrl,
    ) -> Result<(TokenResponseWithPreviews, String, Session<WaitingForResponse>), (TokenRequestError, Session<Done>)>
    {
        let result = self
            .process_token_request_inner(token_request, dpop, attr_service, server_url)
            .await;

        match result {
            Ok((response, dpop_pubkey, dpop_nonce)) => {
                let next = self.transition(WaitingForResponse {
                    access_token: response.token_response.access_token.clone(),
                    c_nonce: response.token_response.c_nonce.as_ref().unwrap().clone(), // field is always set below
                    attestation_previews: response.attestation_previews.clone().into_inner(),
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

    pub async fn process_token_request_inner(
        &self,
        token_request: TokenRequest,
        dpop: Dpop,
        attr_service: &impl AttributeService,
        server_url: &BaseUrl,
    ) -> Result<(TokenResponseWithPreviews, VerifyingKey, String), TokenRequestError> {
        if !matches!(
            token_request.grant_type,
            TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code: _ }
        ) {
            return Err(TokenRequestError::UnsupportedTokenRequestType);
        }

        let dpop_public_key = dpop
            .verify(server_url.join("token"), Method::POST, None)
            .map_err(|err| TokenRequestError::IssuanceError(IssuanceError::DpopInvalid(err)))?;

        let code = token_request.code().clone();

        let previews = attr_service
            .attributes(&self.state, token_request)
            .await
            .map_err(|e| TokenRequestError::AttributeService(Box::new(e)))?;

        let c_nonce = random_string(32);
        let dpop_nonce = random_string(32);

        let response = TokenResponseWithPreviews {
            token_response: TokenResponse::new(AccessToken::new(&code), c_nonce),
            attestation_previews: previews,
        };

        Ok((response, dpop_public_key, dpop_nonce))
    }
}

impl TokenResponse {
    pub(crate) fn new(access_token: AccessToken, c_nonce: String) -> TokenResponse {
        TokenResponse {
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
            data: IssuanceData::WaitingForResponse(value.state.data),
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
                data: session_data,
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
        issuer_data: &IssuerData<impl KeyRing>,
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

    pub async fn process_credential_inner(
        &self,
        credential_request: CredentialRequest,
        access_token: AccessToken,
        dpop: Dpop,
        issuer_data: &IssuerData<impl KeyRing>,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        let session_data = self.session_data();

        // Check authorization of the request
        if session_data.access_token != access_token {
            return Err(CredentialRequestError::Unauthorized);
        }

        dpop.verify_expecting_key(
            &session_data.dpop_public_key,
            &issuer_data.server_url.join("credential"),
            &Method::POST,
            Some(&access_token),
            Some(&session_data.dpop_nonce),
        )
        .map_err(|err| CredentialRequestError::IssuanceError(IssuanceError::DpopInvalid(err)))?;

        // Try to determine which attestation the wallet is requesting:
        // - If it names a doctype and we are offering a single attestation of that doctype, return that.
        // - If it names no doctype and we are offering a single attestation, return that.
        // NB: the OpenID4VCI specification leaves open how to make this determination, this is our own behaviour.
        let unsigned = match credential_request.doctype {
            Some(ref requested_doctype) => {
                let offered_mdocs: Vec<_> = session_data
                    .attestation_previews
                    .iter()
                    .map(AsRef::as_ref)
                    .filter(|unsigned: &&UnsignedMdoc| unsigned.doc_type == *requested_doctype)
                    .collect();
                match offered_mdocs.len() {
                    1 => Ok(*offered_mdocs.first().unwrap()),
                    0 => Err(CredentialRequestError::DoctypeNotOffered(requested_doctype.clone())),
                    // If we have more than one mdoc on offer of the specified doctype then it is not clear which one
                    // we should issue; abort
                    _ => Err(CredentialRequestError::UseBatchIssuance),
                }
            }
            None => match session_data.attestation_previews.len() {
                1 => Ok(session_data.attestation_previews.first().unwrap().as_ref()),
                _ => Err(CredentialRequestError::UseBatchIssuance),
            },
        }?;

        let credential_response = verify_pop_and_sign_attestation(
            &session_data.c_nonce,
            &credential_request,
            unsigned.clone(),
            issuer_data,
        )
        .await?;

        Ok(credential_response)
    }

    pub async fn process_batch_credential(
        self,
        credential_requests: CredentialRequests,
        access_token: AccessToken,
        dpop: Dpop,
        issuer_data: &IssuerData<impl KeyRing>,
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
        issuer_data: &IssuerData<impl KeyRing>,
    ) -> Result<CredentialResponses, CredentialRequestError> {
        let session_data = self.session_data();

        // Check authorization of the request
        if session_data.access_token != access_token {
            return Err(CredentialRequestError::Unauthorized);
        }

        dpop.verify_expecting_key(
            &session_data.dpop_public_key,
            &issuer_data.server_url.join("batch_credential"),
            &Method::POST,
            Some(&access_token),
            Some(&session_data.dpop_nonce),
        )
        .map_err(|err| CredentialRequestError::IssuanceError(IssuanceError::DpopInvalid(err)))?;

        let credential_responses = try_join_all(
            credential_requests
                .credential_requests
                .as_ref()
                .iter()
                .zip(session_data.attestation_previews.iter().flat_map(|preview| {
                    itertools::repeat_n::<&UnsignedMdoc>(preview.as_ref(), preview.copy_count().into())
                }))
                .map(|(cred_req, unsigned_mdoc)| async move {
                    verify_pop_and_sign_attestation(&session_data.c_nonce, cred_req, unsigned_mdoc.clone(), issuer_data)
                        .await
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

pub(crate) async fn verify_pop_and_sign_attestation(
    c_nonce: &str,
    cred_req: &CredentialRequest,
    unsigned_mdoc: UnsignedMdoc,
    issuer_data: &IssuerData<impl KeyRing>,
) -> Result<CredentialResponse, CredentialRequestError> {
    if !matches!(cred_req.format, Format::MsoMdoc) {
        return Err(CredentialRequestError::UnsupportedCredentialFormat(cred_req.format));
    }

    if *cred_req
        .doctype
        .as_ref()
        .ok_or(CredentialRequestError::DoctypeMismatch)?
        != unsigned_mdoc.doc_type
    {
        return Err(CredentialRequestError::DoctypeMismatch);
    }

    let pubkey = cred_req
        .proof
        .as_ref()
        .ok_or(CredentialRequestError::MissingCredentialRequestPoP)?
        .verify(
            c_nonce,
            &issuer_data.accepted_wallet_client_ids,
            &issuer_data.credential_issuer_identifier,
        )?;
    let mdoc_public_key = (&pubkey)
        .try_into()
        .map_err(CredentialRequestError::CoseKeyConversion)?;

    let private_key =
        issuer_data
            .private_keys
            .key_pair(&unsigned_mdoc.doc_type)
            .ok_or(CredentialRequestError::MissingPrivateKey(
                unsigned_mdoc.doc_type.clone(),
            ))?;
    let issuer_signed = IssuerSigned::sign(unsigned_mdoc, mdoc_public_key, private_key)
        .await
        .map_err(CredentialRequestError::AttestationSigning)?;

    Ok(CredentialResponse::MsoMdoc {
        credential: issuer_signed.into(),
    })
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
        let token_data = jsonwebtoken::decode::<CredentialRequestProofJwtPayload>(
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
        if token_data
            .claims
            .nonce
            .as_ref()
            .ok_or(CredentialRequestError::IncorrectNonce)?
            != nonce
        {
            return Err(CredentialRequestError::IncorrectNonce);
        }

        Ok(verifying_key)
    }
}

#[cfg(test)]
mod tests {
    use thiserror::Error;
    use tracing_test::traced_test;

    use super::*;

    #[derive(Debug, Error, Clone, Eq, PartialEq)]
    #[error("MyError")]
    struct MyError;

    #[traced_test]
    #[test]
    fn test_logged_issuance_result() {
        let mut input: Result<String, MyError>;

        assert!(!logs_contain("Issuance Success"));
        input = Ok("Alright".into());
        let result = logged_issuance_result(input.clone());
        assert_eq!(result, input);
        assert!(logs_contain("Issuance Success"));

        assert!(!logs_contain("Issuance Error: MyError"));
        input = Err(MyError);
        let result = logged_issuance_result(input.clone());
        assert_eq!(result, input);
        assert!(logs_contain("Issuance Error: MyError"));
    }
}
