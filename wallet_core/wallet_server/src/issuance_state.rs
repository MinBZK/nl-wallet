use base64::prelude::*;
use chrono::Utc;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc, server_keys::KeyRing, server_state::SessionState, utils::serialization::cbor_serialize,
    IssuerSigned,
};
use openid4vc::{
    credential::{CredentialRequest, CredentialRequests, CredentialResponse, CredentialResponses},
    token::{TokenRequest, TokenResponse, TokenResponseWithPreviews, TokenType},
};
use wallet_common::utils::random_string;

use crate::{
    issuer::{code_to_access_token, AttributeService},
    verifier::Error,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Created {
    pub code: String,
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
        code_hash_key: &[u8],
    ) -> Result<(TokenResponseWithPreviews, Session<WaitingForResponse>), (Error, Session<Done>)> {
        let code = token_request.code();
        let unsigned_mdocs = attr_service.attributes(&self.state, token_request).await.unwrap(); // TODO

        let access_token = code_to_access_token(code_hash_key, &code);
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

    pub async fn process_response(
        self,
        credential_requests: CredentialRequests,
        authorization_header: String,
        private_keys: &impl KeyRing,
    ) -> (Result<CredentialResponses, Error>, Session<Done>) {
        let session_data = self.session_data();

        // Sanity check
        if session_data.access_token != authorization_header {
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
                    sign_attestation(private_keys, session_data.c_nonce.clone(), cred_req, unsigned_mdoc).await
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

pub(crate) async fn sign_attestation(
    private_keys: &impl KeyRing,
    c_nonce: String,
    cred_req: &CredentialRequest,
    unsigned_mdoc: &UnsignedMdoc,
) -> Result<CredentialResponse, Error> {
    assert!(matches!(cred_req.format, openid4vc::Format::MsoMdoc));
    let pubkey = cred_req
        .proof
        .verify(c_nonce, "wallet_name".to_string(), "audience".to_string())
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
        format: openid4vc::Format::MsoMdoc,
        credential: serde_json::to_value(
            BASE64_URL_SAFE_NO_PAD.encode(cbor_serialize(&issuer_signed).unwrap()), // TODO
        )
        .unwrap(),
    })
}
