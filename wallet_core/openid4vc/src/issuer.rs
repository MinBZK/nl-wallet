use std::{collections::HashSet, sync::Arc, time::Duration};

use async_trait::async_trait;
use base64::prelude::*;
use chrono::Utc;
use futures::future::try_join_all;
use jsonwebtoken::{Algorithm, Validation};
use p256::ecdsa::VerifyingKey;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    server_keys::KeyRing,
    server_state::{SessionState, SessionStore, SessionStoreError, CLEANUP_INTERVAL_SECONDS},
    utils::serialization::{cbor_serialize, CborError},
    IssuerSigned,
};
use wallet_common::utils::random_string;

use crate::{
    credential::{
        CredentialRequest, CredentialRequestProof, CredentialRequestProofJwtPayload, CredentialRequests,
        CredentialResponse, CredentialResponses, OPENID4VCI_VC_POP_JWT_TYPE,
    },
    jwk_to_p256,
    jwt::EcdsaDecodingKey,
    token::{TokenRequest, TokenRequestGrantType, TokenResponse, TokenResponseWithPreviews, TokenType},
    Format, JwkConversionError,
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
}

/// Errors that can occur during processing of the token request.
#[derive(Debug, thiserror::Error)]
pub enum TokenRequestError {
    #[error(transparent)]
    IssuanceError(#[from] Error),
    #[error("unsupported token request type: must be of type pre-authorized_code")]
    UnsupportedTokenRequestType,
    #[error("failed to get attributes to be issued: {0}")]
    AttributeService(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

/// Errors that can occur during handling of the (batch) credential request.
#[derive(Debug, thiserror::Error)]
pub enum CredentialRequestError {
    #[error(transparent)]
    IssuanceError(#[from] Error),
    #[error("unauthorized: incorrect access token")]
    Unauthorized,
    #[error("malformed access token")]
    MalformedToken,
    #[error("too many credentials to be issued, use /batch_credential instead")]
    UseBatchIssuance,
    #[error("unsupported credential format: {0:?}")]
    UnsupportedCredentialFormat(Format),
    #[error("missing JWK")]
    MissingJwk,
    #[error("incorrect nonce")]
    IncorrectNonce,
    #[error("unsupported JWT algorithm: expected {expected}, found {found}")]
    UnsupportedJwtAlgorithm { expected: String, found: String },
    #[error("JWT decoding failed: {0}")]
    JwtDecodingFailed(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    JwkConversion(#[from] JwkConversionError),
    #[error("failed to convert P256 public key to COSE key: {0}")]
    CoseKeyConversion(nl_wallet_mdoc::Error),
    #[error("missing issuance private key for doctype {0}")]
    MissingPrivateKey(String),
    #[error("failed to sign attestation: {0}")]
    AttestationSigning(nl_wallet_mdoc::Error),
    #[error(transparent)]
    CborSerialization(#[from] CborError),
    #[error("JSON serialization failed: {0}")]
    JsonSerialization(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Created {
    pub unsigned_mdocs: Option<Vec<UnsignedMdoc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WaitingForResponse {
    pub access_token: String,
    pub c_nonce: String,
    pub unsigned_mdocs: Vec<UnsignedMdoc>,
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

#[async_trait]
pub trait AttributeService: Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;
    type Settings;

    async fn new(settings: &Self::Settings) -> Result<Self, Self::Error>
    where
        Self: Sized;

    async fn attributes(
        &self,
        session: &SessionState<Created>,
        token_request: TokenRequest,
    ) -> Result<Vec<UnsignedMdoc>, Self::Error>;
}

pub struct Issuer<A, K, S> {
    sessions: Arc<S>,
    attr_service: A,
    private_keys: K,
    credential_issuer_identifier: Url,
    wallet_client_ids: Vec<String>,
    cleanup_task: JoinHandle<()>,
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
        credential_issuer_identifier: Url,
        wallet_client_ids: Vec<String>,
    ) -> Self {
        let sessions = Arc::new(sessions);
        Self {
            sessions: sessions.clone(),
            attr_service,
            private_keys,
            credential_issuer_identifier,
            wallet_client_ids,
            cleanup_task: sessions.start_cleanup_task(Duration::from_secs(CLEANUP_INTERVAL_SECONDS)),
        }
    }

    pub async fn process_token_request(
        &self,
        token_request: TokenRequest,
    ) -> Result<TokenResponseWithPreviews, TokenRequestError> {
        let code = token_request.code();

        // Retrieve the session from the session store, if present. It need not be, depending on the implementation of the
        // attribute service.
        let session = self
            .sessions
            .get(&code.clone().into())
            .await
            .map_err(|e| TokenRequestError::IssuanceError(e.into()))?
            .unwrap_or(SessionState::<IssuanceData>::new(
                code.clone().into(),
                IssuanceData::Created(Created { unsigned_mdocs: None }),
            ));
        let session = Session::<Created>::from_enum(session).map_err(TokenRequestError::IssuanceError)?;

        let result = session.process_token_request(token_request, &self.attr_service).await;

        let (response, next) = match result {
            Ok((response, next)) => (Ok(response), next.into_enum()),
            Err((err, next)) => (Err(err), next.into_enum()),
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
        credential_request: CredentialRequest,
    ) -> Result<CredentialResponse, CredentialRequestError> {
        let code = code_from_access_token(access_token)?;
        let session = self
            .sessions
            .get(&code.clone().into())
            .await
            .map_err(|e| CredentialRequestError::IssuanceError(e.into()))?
            .ok_or(CredentialRequestError::IssuanceError(Error::UnknownSession(code)))?;
        let session =
            Session::<WaitingForResponse>::from_enum(session).map_err(CredentialRequestError::IssuanceError)?;

        let (response, next) = session
            .process_credential(
                credential_request,
                access_token.to_string(),
                &self.private_keys,
                &self.credential_issuer_identifier,
                &self.wallet_client_ids,
            )
            .await;

        self.sessions
            .write(&next.into_enum())
            .await
            .map_err(|e| CredentialRequestError::IssuanceError(e.into()))?;

        response
    }

    pub async fn process_batch_credential(
        &self,
        access_token: &str,
        credential_requests: CredentialRequests,
    ) -> Result<CredentialResponses, CredentialRequestError> {
        let code = code_from_access_token(access_token)?;
        let session = self
            .sessions
            .get(&code.clone().into())
            .await
            .map_err(|e| CredentialRequestError::IssuanceError(e.into()))?
            .ok_or(CredentialRequestError::IssuanceError(Error::UnknownSession(code)))?;
        let session =
            Session::<WaitingForResponse>::from_enum(session).map_err(CredentialRequestError::IssuanceError)?;

        let (response, next) = session
            .process_batch_credential(
                credential_requests,
                access_token.to_string(),
                &self.private_keys,
                &self.credential_issuer_identifier,
                &self.wallet_client_ids,
            )
            .await;

        self.sessions
            .write(&next.into_enum())
            .await
            .map_err(|e| CredentialRequestError::IssuanceError(e.into()))?;

        response
    }

    pub async fn process_reject_issuance(&self, access_token: &str) -> Result<(), CredentialRequestError> {
        let code = code_from_access_token(access_token)?;
        let session = self
            .sessions
            .get(&code.clone().into())
            .await
            .map_err(|e| CredentialRequestError::IssuanceError(e.into()))?
            .ok_or(CredentialRequestError::IssuanceError(Error::UnknownSession(code)))?;
        let session =
            Session::<WaitingForResponse>::from_enum(session).map_err(CredentialRequestError::IssuanceError)?;

        // Check authorization of the request
        if session.session_data().access_token != access_token {
            return Err(CredentialRequestError::Unauthorized);
        }

        let next = session.transition(Done {
            session_result: SessionResult::Cancelled,
        });

        self.sessions
            .write(&next.into_enum())
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

impl Session<Created> {
    pub fn from_enum(session: SessionState<IssuanceData>) -> Result<Self, Error> {
        let session_data = match session.session_data {
            IssuanceData::Created(state) => state,
            _ => return Err(Error::UnexpectedState),
        };
        Ok(Session::<Created> {
            state: SessionState {
                session_data,
                token: session.token,
                last_active: session.last_active,
            },
        })
    }

    pub async fn process_token_request(
        self,
        token_request: TokenRequest,
        attr_service: &impl AttributeService,
    ) -> Result<(TokenResponseWithPreviews, Session<WaitingForResponse>), (TokenRequestError, Session<Done>)> {
        let result = self.process_token_request_inner(token_request, attr_service).await;

        match result {
            Ok(response) => {
                let next = self.transition(WaitingForResponse {
                    access_token: response.token_response.access_token.clone(),
                    c_nonce: response.token_response.c_nonce.as_ref().unwrap().clone(), // field is always set below
                    unsigned_mdocs: response.attestation_previews.clone(),
                });
                Ok((response, next))
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
        attr_service: &impl AttributeService,
    ) -> Result<TokenResponseWithPreviews, TokenRequestError> {
        if !matches!(
            token_request.grant_type,
            TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code: _ }
        ) {
            return Err(TokenRequestError::UnsupportedTokenRequestType);
        }

        let code = token_request.code();
        let unsigned_mdocs = attr_service
            .attributes(&self.state, token_request)
            .await
            .map_err(|e| TokenRequestError::AttributeService(Box::new(e)))?;

        // Append the authorization code, so that when the wallet comes back we can use it to retrieve the session
        let access_token = random_string(32) + &code;
        let c_nonce = random_string(32);

        let response = TokenResponseWithPreviews {
            token_response: TokenResponse {
                access_token: access_token.clone(),
                c_nonce: Some(c_nonce.clone()),
                token_type: TokenType::Bearer,
                expires_in: None,
                refresh_token: None,
                scope: None,
                c_nonce_expires_in: None,
                authorization_details: None,
            },
            attestation_previews: unsigned_mdocs.clone(),
        };

        Ok(response)
    }
}

impl Session<WaitingForResponse> {
    pub fn into_enum(self) -> SessionState<IssuanceData> {
        SessionState {
            session_data: IssuanceData::WaitingForResponse(self.state.session_data),
            token: self.state.token,
            last_active: self.state.last_active,
        }
    }

    pub fn from_enum(session: SessionState<IssuanceData>) -> Result<Self, Error> {
        let session_data = match session.session_data {
            IssuanceData::WaitingForResponse(state) => state,
            _ => return Err(Error::UnexpectedState),
        };
        Ok(Session::<WaitingForResponse> {
            state: SessionState {
                session_data,
                token: session.token,
                last_active: session.last_active,
            },
        })
    }

    pub async fn process_credential(
        self,
        credential_request: CredentialRequest,
        access_token: String,
        private_keys: &impl KeyRing,
        credential_issuer_identifier: &Url,
        accepted_wallet_client_ids: &[impl ToString],
    ) -> (Result<CredentialResponse, CredentialRequestError>, Session<Done>) {
        let result = self
            .process_credential_inner(
                credential_request,
                access_token,
                private_keys,
                credential_issuer_identifier,
                accepted_wallet_client_ids,
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

    pub async fn process_credential_inner(
        &self,
        credential_request: CredentialRequest,
        access_token: String,
        private_keys: &impl KeyRing,
        credential_issuer_identifier: &Url,
        accepted_wallet_client_ids: &[impl ToString],
    ) -> Result<CredentialResponse, CredentialRequestError> {
        let session_data = self.session_data();

        // Check authorization of the request
        if session_data.access_token != access_token {
            return Err(CredentialRequestError::Unauthorized);
        }

        // In the pre-authorized code flow, the credential request offers no way for the wallet to refer to a specific
        // offered credential that it wants to accept. So for now, we simply proceed only if there is a single
        // attestation to be issued so that there can be no ambiguity.
        if session_data.unsigned_mdocs.len() != 1 {
            return Err(CredentialRequestError::UseBatchIssuance);
        }

        let credential_response = verify_pop_and_sign_attestation(
            private_keys,
            session_data.c_nonce.clone(),
            &credential_request,
            session_data.unsigned_mdocs.first().unwrap(), // safe because we checked above that this exists
            credential_issuer_identifier,
            accepted_wallet_client_ids,
        )
        .await?;

        Ok(credential_response)
    }

    pub async fn process_batch_credential(
        self,
        credential_requests: CredentialRequests,
        access_token: String,
        private_keys: &impl KeyRing,
        credential_issuer_identifier: &Url,
        accepted_wallet_client_ids: &[impl ToString],
    ) -> (Result<CredentialResponses, CredentialRequestError>, Session<Done>) {
        let result = self
            .process_batch_credential_inner(
                credential_requests,
                access_token,
                private_keys,
                credential_issuer_identifier,
                accepted_wallet_client_ids,
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
        access_token: String,
        private_keys: &impl KeyRing,
        credential_issuer_identifier: &Url,
        accepted_wallet_client_ids: &[impl ToString],
    ) -> Result<CredentialResponses, CredentialRequestError> {
        let session_data = self.session_data();

        // Check authorization of the request
        if session_data.access_token != access_token {
            return Err(CredentialRequestError::Unauthorized);
        }

        let credential_responses = try_join_all(
            credential_requests
                .credential_requests
                .iter()
                .zip(
                    session_data
                        .unsigned_mdocs
                        .iter()
                        .flat_map(|unsigned| std::iter::repeat(unsigned).take(unsigned.copy_count as usize)),
                )
                .map(|(cred_req, unsigned_mdoc)| async {
                    verify_pop_and_sign_attestation(
                        private_keys,
                        session_data.c_nonce.clone(),
                        cred_req,
                        unsigned_mdoc,
                        credential_issuer_identifier,
                        accepted_wallet_client_ids,
                    )
                    .await
                }),
        )
        .await?;

        Ok(CredentialResponses { credential_responses })
    }
}

impl Session<Done> {
    pub fn into_enum(self) -> SessionState<IssuanceData> {
        SessionState {
            session_data: IssuanceData::Done(self.state.session_data),
            token: self.state.token,
            last_active: self.state.last_active,
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
    private_keys: &impl KeyRing,
    c_nonce: String,
    cred_req: &CredentialRequest,
    unsigned_mdoc: &UnsignedMdoc,
    credential_issuer_identifier: &Url,
    accepted_wallet_client_ids: &[impl ToString],
) -> Result<CredentialResponse, CredentialRequestError> {
    if !matches!(cred_req.format, Format::MsoMdoc) {
        return Err(CredentialRequestError::UnsupportedCredentialFormat(
            cred_req.format.clone(),
        ));
    }

    let pubkey = cred_req
        .proof
        .verify(c_nonce, accepted_wallet_client_ids, credential_issuer_identifier)?;
    let mdoc_public_key = (&pubkey)
        .try_into()
        .map_err(CredentialRequestError::CoseKeyConversion)?;

    let (issuer_signed, _) =
        IssuerSigned::sign(
            unsigned_mdoc.clone(),
            mdoc_public_key,
            private_keys.private_key(&unsigned_mdoc.doc_type).as_ref().ok_or(
                CredentialRequestError::MissingPrivateKey(unsigned_mdoc.doc_type.clone()),
            )?,
        )
        .await
        .map_err(CredentialRequestError::AttestationSigning)?;

    Ok(CredentialResponse {
        format: Format::MsoMdoc,
        credential: serde_json::to_value(BASE64_URL_SAFE_NO_PAD.encode(cbor_serialize(&issuer_signed)?))?,
    })
}

impl CredentialRequestProof {
    pub fn verify(
        &self,
        nonce: String,
        accepted_wallet_client_ids: &[impl ToString],
        credential_issuer_identifier: &Url,
    ) -> Result<VerifyingKey, CredentialRequestError> {
        let jwt = match self {
            CredentialRequestProof::Jwt { jwt } => jwt,
        };
        let header = jsonwebtoken::decode_header(&jwt.0)?;
        let verifying_key = jwk_to_p256(&header.jwk.ok_or(CredentialRequestError::MissingJwk)?)?;

        let mut validation_options = Validation::new(Algorithm::ES256);
        validation_options.required_spec_claims = HashSet::from(["iss".to_string(), "aud".to_string()]);
        validation_options.set_issuer(accepted_wallet_client_ids);
        validation_options.set_audience(&[credential_issuer_identifier]);
        let token_data = jsonwebtoken::decode::<CredentialRequestProofJwtPayload>(
            &jwt.0,
            &EcdsaDecodingKey::from(verifying_key).0,
            &validation_options,
        )?;

        if token_data.header.typ != Some(OPENID4VCI_VC_POP_JWT_TYPE.to_string()) {
            return Err(CredentialRequestError::UnsupportedJwtAlgorithm {
                expected: OPENID4VCI_VC_POP_JWT_TYPE.to_string(),
                found: token_data.header.typ.unwrap_or_default(),
            });
        }
        if token_data.claims.nonce != nonce {
            return Err(CredentialRequestError::IncorrectNonce);
        }

        Ok(verifying_key)
    }
}
