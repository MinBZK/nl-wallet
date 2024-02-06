use std::{collections::HashSet, sync::Arc, time::Duration};

use chrono::Utc;
use futures::future::try_join_all;
use jsonwebtoken::{Algorithm, Validation};
use p256::ecdsa::VerifyingKey;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    server_keys::KeyRing,
    server_state::{SessionState, SessionStore, SessionStoreError, CLEANUP_INTERVAL_SECONDS},
    utils::serialization::CborError,
    IssuerSigned,
};
use wallet_common::{jwt::EcdsaDecodingKey, utils::random_string};

use crate::{
    credential::{
        CredentialRequest, CredentialRequestProof, CredentialRequestProofJwtPayload, CredentialRequests,
        CredentialResponse, CredentialResponses, OPENID4VCI_VC_POP_JWT_TYPE,
    },
    dpop::{Dpop, DpopError},
    jwk::{jwk_to_p256, JwkConversionError},
    token::{
        AttestationPreview, TokenRequest, TokenRequestGrantType, TokenResponse, TokenResponseWithPreviews, TokenType,
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
pub enum Error {
    #[error("session not in expected state")]
    UnexpectedState,
    #[error("unknown session: {0}")]
    UnknownSession(String),
    #[error("failed to retrieve session: {0}")]
    SessionStore(#[from] SessionStoreError),
    #[error("invalid DPoP header: {0}")]
    DpopInvalid(#[source] DpopError),
}

/// Errors that can occur during processing of the token request.
#[derive(Debug, thiserror::Error)]
pub enum TokenRequestError {
    #[error("issuance error: {0}")]
    IssuanceError(#[from] Error),
    #[error("unsupported token request type: must be of type pre-authorized_code")]
    UnsupportedTokenRequestType,
    #[error("failed to get attributes to be issued: {0}")]
    AttributeService(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("no attributes found to be issued")]
    NoAttributes,
}

/// Errors that can occur during handling of the (batch) credential request.
#[derive(Debug, thiserror::Error)]
pub enum CredentialRequestError {
    #[error("issuance error: {0}")]
    IssuanceError(#[from] Error),
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
    CoseKeyConversion(nl_wallet_mdoc::Error),
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
    pub access_token: String,
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
}

#[derive(Debug)]
pub struct Session<S: IssuanceState> {
    pub state: SessionState<S>,
}

#[trait_variant::make(AttributeService: Send)]
pub trait LocalAttributeService {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn attributes(
        &self,
        session: &SessionState<Created>,
        token_request: TokenRequest,
    ) -> Result<Vec<UnsignedMdoc>, Self::Error>;
}

pub struct Issuer<A, K, S> {
    sessions: Arc<S>,
    attr_service: A,
    issuer_data: IssuerData<K>,
    cleanup_task: JoinHandle<()>,
}

/// Fields of the [`Issuer`] needed by the issuance functions.
pub struct IssuerData<K> {
    private_keys: K,

    /// URL identifying the issuer; should host ` /.well-known/openid-credential-issuer`,
    /// and MUST be used by the wallet as `aud` in its PoP JWTs.
    credential_issuer_identifier: Url,

    /// Wallet IDs accepted by this server, MUST be used by the wallet as `iss` in its PoP JWTs.
    accepted_wallet_client_ids: Vec<String>,

    /// URL prefix of the `/token`, `/credential` and `/batch_crededential` endpoints.
    server_url: Url,
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
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    pub fn new(
        sessions: S,
        attr_service: A,
        private_keys: K,
        server_url: &Url,
        wallet_client_ids: Vec<String>,
    ) -> Self {
        let sessions = Arc::new(sessions);

        let issuer_data = IssuerData {
            private_keys,
            credential_issuer_identifier: server_url.join("issuance/").unwrap(),
            accepted_wallet_client_ids: wallet_client_ids,

            // In this implementation, for now the Credential Issuer Identifier also always acts as
            // the public server URL.
            server_url: server_url.join("issuance/").unwrap(),
        };

        Self {
            sessions: Arc::clone(&sessions),
            attr_service,
            issuer_data,
            cleanup_task: sessions.start_cleanup_task(Duration::from_secs(CLEANUP_INTERVAL_SECONDS)),
        }
    }
}

impl<A, K, S> Issuer<A, K, S>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>>,
{
    pub async fn process_token_request(
        &self,
        token_request: TokenRequest,
        dpop: Dpop,
    ) -> Result<(TokenResponseWithPreviews, String), TokenRequestError> {
        let session_token = token_request.code().to_string().into();

        // Retrieve the session from the session store, if present. It need not be, depending on the implementation of the
        // attribute service.
        let session = self
            .sessions
            .get(&session_token)
            .await
            .map_err(|e| TokenRequestError::IssuanceError(e.into()))?
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
            .write(&next)
            .await
            .map_err(|e| TokenRequestError::IssuanceError(e.into()))?;

        response
    }

    pub async fn process_credential(
        &self,
        access_token: &str,
        dpop: Dpop,
        credential_request: CredentialRequest,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        let code = code_from_access_token(access_token)?;
        let session = self
            .sessions
            .get(&code.clone().into())
            .await
            .map_err(|e| CredentialRequestError::IssuanceError(e.into()))?
            .ok_or(CredentialRequestError::IssuanceError(Error::UnknownSession(code)))?;
        let session: Session<WaitingForResponse> = session.try_into().map_err(CredentialRequestError::IssuanceError)?;

        let (response, next) = session
            .process_credential(credential_request, access_token.to_string(), dpop, &self.issuer_data)
            .await;

        self.sessions
            .write(&next.into())
            .await
            .map_err(|e| CredentialRequestError::IssuanceError(e.into()))?;

        response
    }

    pub async fn process_batch_credential(
        &self,
        access_token: &str,
        dpop: Dpop,
        credential_requests: CredentialRequests,
    ) -> Result<CredentialResponses, CredentialRequestError> {
        let code = code_from_access_token(access_token)?;
        let session = self
            .sessions
            .get(&code.clone().into())
            .await
            .map_err(|e| CredentialRequestError::IssuanceError(e.into()))?
            .ok_or(CredentialRequestError::IssuanceError(Error::UnknownSession(code)))?;
        let session: Session<WaitingForResponse> = session.try_into().map_err(CredentialRequestError::IssuanceError)?;

        let (response, next) = session
            .process_batch_credential(credential_requests, access_token.to_string(), dpop, &self.issuer_data)
            .await;

        self.sessions
            .write(&next.into())
            .await
            .map_err(|e| CredentialRequestError::IssuanceError(e.into()))?;

        response
    }

    pub async fn process_reject_issuance(
        &self,
        access_token: &str,
        dpop: Dpop,
        endpoint_name: &str,
    ) -> Result<(), CredentialRequestError> {
        let code = code_from_access_token(access_token)?;
        let session = self
            .sessions
            .get(&code.clone().into())
            .await
            .map_err(|e| CredentialRequestError::IssuanceError(e.into()))?
            .ok_or(CredentialRequestError::IssuanceError(Error::UnknownSession(code)))?;
        let session: Session<WaitingForResponse> = session.try_into().map_err(CredentialRequestError::IssuanceError)?;

        // Check authorization of the request
        let session_data = session.session_data();
        if session_data.access_token != access_token {
            return Err(CredentialRequestError::Unauthorized);
        }

        dpop.verify_expecting_key(
            &session_data.dpop_public_key,
            &self
                .issuer_data
                .credential_issuer_identifier
                .join(endpoint_name)
                .unwrap(),
            &Method::DELETE,
            &Some(access_token.to_string()),
            &Some(session_data.dpop_nonce.clone()),
        )
        .await
        .map_err(|err| CredentialRequestError::IssuanceError(Error::DpopInvalid(err)))?;

        let next = session.transition(Done {
            session_result: SessionResult::Cancelled,
        });

        self.sessions
            .write(&next.into())
            .await
            .map_err(|e| CredentialRequestError::IssuanceError(e.into()))?;

        Ok(())
    }
}

/// Returns the authorization code from an access token.
///
/// The access token should be a random string with the authorization code appended to it, so that we can
/// use the code suffix to retrieve the session from the session store.
fn code_from_access_token(access_token: &str) -> Result<String, CredentialRequestError> {
    let code = access_token
        .get(32..)
        .ok_or(CredentialRequestError::MalformedToken)?
        .to_string();
    Ok(code)
}

impl TryFrom<SessionState<IssuanceData>> for Session<Created> {
    type Error = Error;

    fn try_from(value: SessionState<IssuanceData>) -> Result<Self, Self::Error> {
        let session_data = match value.session_data {
            IssuanceData::Created(state) => state,
            _ => return Err(Error::UnexpectedState),
        };
        Ok(Session::<Created> {
            state: SessionState {
                session_data,
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
        server_url: &Url,
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
                    attestation_previews: response.attestation_previews.clone(),
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
        server_url: &Url,
    ) -> Result<(TokenResponseWithPreviews, VerifyingKey, String), TokenRequestError> {
        if !matches!(
            token_request.grant_type,
            TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code: _ }
        ) {
            return Err(TokenRequestError::UnsupportedTokenRequestType);
        }

        let dpop_public_key = dpop
            .verify(server_url.join("token").unwrap(), Method::POST, None)
            .await
            .map_err(|err| TokenRequestError::IssuanceError(Error::DpopInvalid(err)))?;

        let code = token_request.code().to_string();

        let unsigned_mdocs = attr_service
            .attributes(&self.state, token_request)
            .await
            .map_err(|e| TokenRequestError::AttributeService(Box::new(e)))?;
        if unsigned_mdocs.is_empty() {
            return Err(TokenRequestError::NoAttributes);
        }

        // Append the authorization code, so that when the wallet comes back we can use it to retrieve the session
        let access_token = random_string(32) + &code;
        let c_nonce = random_string(32);
        let dpop_nonce = random_string(32);

        let response = TokenResponseWithPreviews {
            token_response: TokenResponse {
                access_token,
                c_nonce: Some(c_nonce),
                token_type: TokenType::Bearer,
                expires_in: None,
                refresh_token: None,
                scope: None,
                c_nonce_expires_in: None,
                authorization_details: None,
            },
            attestation_previews: unsigned_mdocs
                .iter()
                .map(|unsigned| AttestationPreview::MsoMdoc {
                    unsigned_mdoc: unsigned.clone(),
                })
                .collect(),
        };

        Ok((response, dpop_public_key, dpop_nonce))
    }
}

impl From<Session<WaitingForResponse>> for SessionState<IssuanceData> {
    fn from(value: Session<WaitingForResponse>) -> Self {
        SessionState {
            session_data: IssuanceData::WaitingForResponse(value.state.session_data),
            token: value.state.token,
            last_active: value.state.last_active,
        }
    }
}

impl TryFrom<SessionState<IssuanceData>> for Session<WaitingForResponse> {
    type Error = Error;

    fn try_from(value: SessionState<IssuanceData>) -> Result<Self, Self::Error> {
        let session_data = match value.session_data {
            IssuanceData::WaitingForResponse(state) => state,
            _ => return Err(Error::UnexpectedState),
        };
        Ok(Session::<WaitingForResponse> {
            state: SessionState {
                session_data,
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
        access_token: String,
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
        access_token: String,
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
            &issuer_data.server_url.join("credential").unwrap(),
            &Method::POST,
            &Some(access_token),
            &Some(session_data.dpop_nonce.clone()),
        )
        .await
        .map_err(|err| CredentialRequestError::IssuanceError(Error::DpopInvalid(err)))?;

        // Try to determine which attestation the wallet is requesting
        let unsigned = match credential_request.doctype {
            Some(ref requested_doctype) => {
                let offered_mdocs: Vec<_> = session_data
                    .attestation_previews
                    .iter()
                    .map(|preview| preview.into())
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
                1 => Ok(session_data.attestation_previews.first().unwrap().into()),
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
        access_token: String,
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
        access_token: String,
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
            &issuer_data.server_url.join("batch_credential").unwrap(),
            &Method::POST,
            &Some(access_token),
            &Some(session_data.dpop_nonce.clone()),
        )
        .await
        .map_err(|err| CredentialRequestError::IssuanceError(Error::DpopInvalid(err)))?;

        let credential_responses = try_join_all(
            credential_requests
                .credential_requests
                .iter()
                .zip(session_data.attestation_previews.iter().flat_map(|preview| {
                    itertools::repeat_n::<&UnsignedMdoc>(preview.into(), preview.copy_count() as usize)
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
            session_data: IssuanceData::Done(value.state.session_data),
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
            state: SessionState::<NewT> {
                session_data: new_state,
                token: self.state.token,
                last_active: Utc::now(),
            },
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
        &self.state.session_data
    }
}

pub(crate) async fn verify_pop_and_sign_attestation(
    c_nonce: &str,
    cred_req: &CredentialRequest,
    unsigned_mdoc: UnsignedMdoc,
    issuer_data: &IssuerData<impl KeyRing>,
) -> Result<CredentialResponse, CredentialRequestError> {
    if !matches!(cred_req.format, Format::MsoMdoc) {
        return Err(CredentialRequestError::UnsupportedCredentialFormat(
            cred_req.format.clone(),
        ));
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

    let private_key = issuer_data.private_keys.private_key(&unsigned_mdoc.doc_type).ok_or(
        CredentialRequestError::MissingPrivateKey(unsigned_mdoc.doc_type.clone()),
    )?;
    let (issuer_signed, _) = IssuerSigned::sign(unsigned_mdoc, mdoc_public_key, private_key)
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
        credential_issuer_identifier: &Url,
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
