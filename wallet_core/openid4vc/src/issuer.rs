use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use base64::prelude::*;
use chrono::Utc;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

use crate::{
    credential::{CredentialRequest, CredentialRequests, CredentialResponse, CredentialResponses},
    token::{TokenRequest, TokenRequestGrantType, TokenResponse, TokenResponseWithPreviews, TokenType},
    Format,
};
use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    server_keys::KeyRing,
    server_state::{SessionState, SessionStore, CLEANUP_INTERVAL_SECONDS},
    utils::serialization::cbor_serialize,
    IssuerSigned,
};
use url::Url;
use wallet_common::utils::random_string;

#[derive(Debug, thiserror::Error)]
pub enum Error {} // TODO proper error handling

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
    type Error: std::fmt::Debug;
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

    pub async fn process_token_request(&self, token_request: TokenRequest) -> Result<TokenResponseWithPreviews, Error> {
        if !matches!(
            token_request.grant_type,
            TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code: _ }
        ) {
            panic!("token request must be of type pre-authorized_code");
        }

        let code = token_request.code();

        // Retrieve the session from the session store, if present. It need not be, depending on the implementation of the
        // attribute service.
        let session =
            self.sessions
                .get(&code.clone().into())
                .await
                .unwrap()
                .unwrap_or(SessionState::<IssuanceData>::new(
                    code.clone().into(),
                    IssuanceData::Created(Created { unsigned_mdocs: None }),
                ));
        let session = Session::<Created>::from_enum(session).unwrap();

        let result = session.process_token_request(token_request, &self.attr_service).await;

        let (response, next) = match result {
            Ok((response, next)) => (Ok(response), next.into_enum()),
            Err((err, next)) => (Err(err), next.into_enum()),
        };

        self.sessions.write(&next).await.unwrap();

        response
    }

    pub async fn process_credential(
        &self,
        access_token: &str,
        credential_request: CredentialRequest,
    ) -> Result<CredentialResponse, Error> {
        let code = code_from_access_token(access_token);
        let session = self.sessions.get(&code.into()).await.unwrap().unwrap(); // TODO
        let session = Session::<WaitingForResponse>::from_enum(session).unwrap(); // TODO

        let (response, next) = session
            .process_credential(
                credential_request,
                access_token.to_string(),
                &self.private_keys,
                &self.credential_issuer_identifier,
                &self.wallet_client_ids,
            )
            .await;

        self.sessions.write(&next.into_enum()).await.unwrap();

        response
    }

    pub async fn process_batch_credential(
        &self,
        access_token: &str,
        credential_requests: CredentialRequests,
    ) -> Result<CredentialResponses, Error> {
        let code = code_from_access_token(access_token);
        let session = self.sessions.get(&code.into()).await.unwrap().unwrap(); // TODO
        let session = Session::<WaitingForResponse>::from_enum(session).unwrap(); // TODO

        let (response, next) = session
            .process_batch_credential(
                credential_requests,
                access_token.to_string(),
                &self.private_keys,
                &self.credential_issuer_identifier,
                &self.wallet_client_ids,
            )
            .await;

        self.sessions.write(&next.into_enum()).await.unwrap();

        response
    }

    pub async fn process_reject_issuance(&self, access_token: &str) -> Result<(), Error> {
        let code = code_from_access_token(access_token);
        let session = self.sessions.get(&code.into()).await.unwrap().unwrap(); // TODO
        let session = Session::<WaitingForResponse>::from_enum(session).unwrap(); // TODO

        // Check authorization of the request
        if session.session_data().access_token != access_token {
            panic!("wrong access token")
        }

        let next = session.transition(Done {
            session_result: SessionResult::Cancelled,
        });

        self.sessions.write(&next.into_enum()).await.unwrap();

        Ok(())
    }
}

/// The access token should be a random string with the authorization code appended to it, so that we can
/// use the code suffix to retrieve the session from the session store. If what the user provided in the
/// authorization header is shorter than that, we can just use unwrap_or_default(), since no session will
/// ever be index by the empty string.
fn code_from_access_token(access_token: &str) -> String {
    access_token.get(32..).unwrap_or_default().to_string()
}

impl Session<Created> {
    pub fn from_enum(session: SessionState<IssuanceData>) -> Result<Self, Error> {
        let session_data = match session.session_data {
            IssuanceData::Created(state) => state,
            _ => panic!("incorrect state"),
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
    ) -> Result<(TokenResponseWithPreviews, Session<WaitingForResponse>), (Error, Session<Done>)> {
        let code = token_request.code();
        let unsigned_mdocs = attr_service.attributes(&self.state, token_request).await.unwrap(); // TODO

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

        let next = self.transition(WaitingForResponse {
            access_token,
            c_nonce,
            unsigned_mdocs,
        });

        Ok((response, next))
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
            _ => panic!("incorrect state"),
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
    ) -> (Result<CredentialResponse, Error>, Session<Done>) {
        let session_data = self.session_data();

        // Check authorization of the request
        if session_data.access_token != access_token {
            panic!("wrong access token")
        }

        // In the pre-authorized code flow, the credential request offers no way for the wallet to refer to a specific
        // offered credential that it wants to accept. So for now, we simply proceed only if there is a single
        // attestation to be issued so that there can be no ambiguity.
        if session_data.unsigned_mdocs.len() != 1 {
            panic!("too many credentials to be issued, use /batch_credential instead")
        }

        let credential_response = verify_pop_and_sign_attestation(
            private_keys,
            session_data.c_nonce.clone(),
            &credential_request,
            session_data.unsigned_mdocs.first().unwrap(),
            credential_issuer_identifier,
            accepted_wallet_client_ids,
        )
        .await
        .unwrap();

        // Transition the session to done. This means the client won't be able to reuse its access token in
        // more requests to this endpoint. (The OpenID4VCI and OAuth specs allow reuse of access tokens, but don't
        // forbid that a server doesn't allow that.)
        let next = self.transition(Done {
            session_result: SessionResult::Done,
        });

        (Ok(credential_response), next)
    }

    pub async fn process_batch_credential(
        self,
        credential_requests: CredentialRequests,
        access_token: String,
        private_keys: &impl KeyRing,
        credential_issuer_identifier: &Url,
        accepted_wallet_client_ids: &[impl ToString],
    ) -> (Result<CredentialResponses, Error>, Session<Done>) {
        let session_data = self.session_data();

        // Check authorization of the request
        if session_data.access_token != access_token {
            panic!("wrong access token")
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
        .await
        .unwrap();

        // Transition the session to done. This means the client won't be able to reuse its access token in
        // more requests to this endpoint. (The OpenID4VCI and OAuth specs allow reuse of access tokens, but don't
        // forbid that a server doesn't allow that.)
        let next = self.transition(Done {
            session_result: SessionResult::Done,
        });

        (Ok(CredentialResponses { credential_responses }), next)
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
) -> Result<CredentialResponse, Error> {
    assert!(matches!(cred_req.format, Format::MsoMdoc));

    let pubkey = cred_req
        .proof
        .verify(c_nonce, accepted_wallet_client_ids, credential_issuer_identifier)
        .unwrap(); // TODO
    let mdoc_public_key = (&pubkey).try_into().unwrap();

    let (issuer_signed, _) = IssuerSigned::sign(
        unsigned_mdoc.clone(),
        mdoc_public_key,
        private_keys.private_key(&unsigned_mdoc.doc_type).as_ref().unwrap(),
    )
    .await
    .unwrap(); // TODO

    Ok(CredentialResponse {
        format: Format::MsoMdoc,
        credential: serde_json::to_value(
            BASE64_URL_SAFE_NO_PAD.encode(cbor_serialize(&issuer_signed).unwrap()), // TODO
        )
        .unwrap(),
    })
}
