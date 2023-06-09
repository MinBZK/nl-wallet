use core::panic;

use chrono::Utc;
use ciborium::value::Value;
use coset::{CoseSign1, HeaderBuilder};
use dashmap::DashMap;
use ecdsa::signature::Signer;
use p256::ecdsa::Signature;
use serde::Deserialize;
use serde_bytes::ByteBuf;

use crate::{
    basic_sa_ext::{
        DataToIssueMessage, KeyGenerationResponseMessage, MdocResponses, MobileIDDocuments,
        RequestKeyGenerationMessage, Response, SparseIssuerAuth, SparseIssuerSigned, StartIssuingMessage, UnsignedMdoc,
        KEY_GEN_RESP_MSG_TYPE, START_ISSUING_MSG_TYPE,
    },
    cose::{ClonePayload, MdocCose},
    crypto::random_bytes,
    iso::*,
    issuer_shared::{IssuanceError, SessionToken},
    serialization::{cbor_deserialize, TaggedBytes},
    signer::{EcdsaKey, SecureEcdsaKey},
    Error, Result,
};

pub struct IssuancePrivateKey {
    private_key: ecdsa::SigningKey<p256::NistP256>,
    cert_bts: Vec<u8>,
}

impl IssuancePrivateKey {
    pub fn new(private_key: ecdsa::SigningKey<p256::NistP256>, cert_bts: Vec<u8>) -> IssuancePrivateKey {
        IssuancePrivateKey { private_key, cert_bts }
    }
}

impl Signer<Signature> for IssuancePrivateKey {
    fn try_sign(&self, msg: &[u8]) -> std::result::Result<Signature, ecdsa::Error> {
        self.private_key.try_sign(msg)
    }
}
impl EcdsaKey for IssuancePrivateKey {
    type Error = ecdsa::Error;
    fn verifying_key(&self) -> std::result::Result<p256::ecdsa::VerifyingKey, Self::Error> {
        Ok(self.private_key.verifying_key())
    }
}
impl SecureEcdsaKey for IssuancePrivateKey {}

pub trait IssuanceKeyring {
    fn private_key(&self, doctype: &DocType) -> Option<&IssuancePrivateKey>;
    fn contains_key(&self, doctype: &DocType) -> bool {
        self.private_key(doctype).is_some()
    }
}

#[derive(Debug, Clone)]
enum SessionState {
    Created,
    Started,
    WaitingForResponse,
    Done,
    Failed,
    Cancelled,
}

use SessionState::*;

impl SessionState {
    fn update(&mut self, new_state: SessionState) {
        match self {
            Created => assert!(matches!(new_state, Started | Failed | Cancelled)),
            Started => assert!(matches!(new_state, WaitingForResponse | Failed | Cancelled)),
            WaitingForResponse => assert!(matches!(new_state, Done | Failed | Cancelled)),
            Done => panic!("can't update final state"),
            Failed => panic!("can't update final state"),
            Cancelled => panic!("can't update final state"),
        }
        *self = new_state;
    }
}

pub trait SessionStore {
    fn get(&self, id: &SessionToken) -> Result<SessionData>;
    fn write(&self, session: &SessionData);
}

#[derive(Debug, Default)]
pub struct MemorySessionStore {
    sessions: DashMap<SessionToken, SessionData>,
}

impl MemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }
}

impl SessionStore for MemorySessionStore {
    fn get(&self, token: &SessionToken) -> Result<SessionData> {
        let data = self
            .sessions
            .get(token)
            .ok_or_else(|| Error::from(IssuanceError::UnknownSessionId(token.clone())))?
            .clone();
        Ok(data)
    }

    fn write(&self, session: &SessionData) {
        self.sessions.insert(session.token.clone(), session.clone());
    }
}

pub struct Server<K, S> {
    url: String,
    keys: K,
    sessions: S,
}

impl<K: IssuanceKeyring, S: SessionStore> Server<K, S> {
    /// Construct a new issuance server. The `url` parameter should be the base URL at which the server is
    /// publically reachable; this is included in the [`ServiceEngagement`] that gets sent to the holder.
    pub fn new(url: String, keys: K, session_store: S) -> Self {
        Server {
            url: if url.ends_with('/') { url } else { url + "/" },
            keys,
            sessions: session_store,
        }
    }

    pub fn new_session(&self, docs: Vec<UnsignedMdoc>) -> Result<ServiceEngagement> {
        self.check_keys(&docs)?;

        let challenge = ByteBuf::from(random_bytes(32));
        let session_id: SessionId = ByteBuf::from(random_bytes(32)).into();
        let token = SessionToken::new();
        let request = RequestKeyGenerationMessage {
            e_session_id: session_id.clone(),
            challenge,
            unsigned_mdocs: docs,
        };

        self.sessions.write(&SessionData {
            token: token.clone(),
            request,
            state: Created,
            id: session_id,
        });

        Ok(ServiceEngagement {
            url: (self.url.clone() + &token.0).into(),
            ..Default::default()
        })
    }

    fn check_keys(&self, docs: &[UnsignedMdoc]) -> Result<()> {
        for doc in docs {
            if !self.keys.contains_key(&doc.doc_type) {
                return Err(IssuanceError::MissingPrivateKey(doc.doc_type.clone()).into());
            }
        }
        Ok(())
    }

    /// Process a CBOR-encoded issuance protocol message from the holder.
    pub fn process_message(&self, token: SessionToken, msg: Vec<u8>) -> Result<Box<dyn IssuerResponse>> {
        let (msg_type, session_id) = Self::inspect_message(&msg)?;

        let mut session_data = self.sessions.get(&token)?;
        let mut session = Session {
            sessions: &self.sessions,
            session_data: &mut session_data,
            keys: &self.keys,
            updated: false,
        };

        // If this is not the very first protocol message, the session ID is expected in every message.
        if msg_type != START_PROVISIONING_MSG_TYPE {
            Self::expect_session_id(
                &session_id.ok_or(Error::from(IssuanceError::MissingSessionId))?,
                &session.session_data.id,
            )?;
        }

        // Stop the session if the user wishes.
        if msg_type == REQUEST_END_SESSION_MSG_TYPE {
            let response = session.process_cancel();
            return Ok(Box::new(response));
        }

        // Process the holder's message according to the current session state.
        match session.session_data.state {
            Created => {
                Self::expect_message_type(&msg_type, START_PROVISIONING_MSG_TYPE)?;
                let response = session.process_start(cbor_deserialize(&msg[..])?);
                Ok(Box::new(response))
            }
            Started => {
                Self::expect_message_type(&msg_type, START_ISSUING_MSG_TYPE)?;
                let response = session.process_get_request(cbor_deserialize(&msg[..])?);
                Ok(Box::new(response))
            }
            WaitingForResponse => {
                Self::expect_message_type(&msg_type, KEY_GEN_RESP_MSG_TYPE)?;
                let response = session.process_response(cbor_deserialize(&msg[..])?)?;
                Ok(Box::new(response))
            }
            Done | Failed | Cancelled => Err(IssuanceError::SessionEnded.into()),
        }
    }

    /// Read the following fields from the CBOR-encoded holder message:
    /// - `messageType`: should be present in every message
    /// - `eSessionId`: should be present in every message except the first
    fn inspect_message(msg: &[u8]) -> Result<(String, Option<SessionId>)> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ProtocolMessage {
            message_type: String,
            e_session_id: Option<SessionId>,
        }

        let decoded: ProtocolMessage = cbor_deserialize(msg)?;
        Ok((decoded.message_type, decoded.e_session_id))
    }

    fn expect_message_type(msg_type: &str, expected: &str) -> Result<()> {
        if msg_type == expected {
            Ok(())
        } else {
            Err(IssuanceError::UnexpectedMessageType {
                received: msg_type.to_string(),
                expected: expected.to_string(),
            }
            .into())
        }
    }

    fn expect_session_id(id: &SessionId, expected: &SessionId) -> Result<()> {
        if id == expected {
            Ok(())
        } else {
            Err(IssuanceError::MismatchedSessionIds {
                received: id.clone(),
                expected: expected.clone(),
            }
            .into())
        }
    }
}

#[derive(Debug)]
struct Session<'a, K, S: SessionStore> {
    sessions: &'a S,
    session_data: &'a mut SessionData,
    keys: &'a K,
    updated: bool,
}

impl<'a, K, S: SessionStore> Drop for Session<'a, K, S> {
    fn drop(&mut self) {
        if self.updated {
            self.sessions.write(self.session_data);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionData {
    request: RequestKeyGenerationMessage,
    state: SessionState,
    id: SessionId,
    token: SessionToken,
}

// The `process_` methods process specific issuance protocol messages from the holder.
impl<'a, K: IssuanceKeyring, S: SessionStore> Session<'a, K, S> {
    fn update_state(&mut self, new_state: SessionState) {
        self.session_data.state.update(new_state);
        self.updated = true;
    }

    fn process_start(&mut self, _: StartProvisioningMessage) -> ReadyToProvisionMessage {
        self.update_state(Started);
        ReadyToProvisionMessage {
            e_session_id: self.session_data.request.e_session_id.clone(),
        }
    }

    fn process_get_request(&mut self, _: StartIssuingMessage) -> RequestKeyGenerationMessage {
        self.update_state(WaitingForResponse);
        self.session_data.request.clone()
    }

    fn process_response(&mut self, device_response: KeyGenerationResponseMessage) -> Result<DataToIssueMessage> {
        let issuance_result = self.issue(&device_response);
        match issuance_result {
            Ok(_) => self.update_state(Done),
            Err(_) => self.update_state(Failed),
        }
        issuance_result
    }

    fn process_cancel(&mut self) -> EndSessionMessage {
        self.update_state(Cancelled);
        EndSessionMessage {
            e_session_id: self.session_data.request.e_session_id.clone(),
            reason: "success".to_string(),
            delay: None,
            sed: None,
        }
    }

    fn issue_cred(&self, unsigned_mdoc: &UnsignedMdoc, response: &Response) -> Result<SparseIssuerSigned> {
        let attrs = unsigned_mdoc
            .attributes
            .clone()
            .into_iter()
            .map(|(namespace, attrs)| Ok((namespace, Attributes::try_from(attrs)?)))
            .collect::<Result<IssuerNameSpaces>>()?;

        let now = Utc::now();
        let validity = ValidityInfo {
            signed: now.into(),
            valid_from: now.into(),
            valid_until: unsigned_mdoc.valid_until.clone(),
            expected_update: None,
        };
        let mso = MobileSecurityObject {
            version: "1.0".to_string(),
            digest_algorithm: "SHA-256".to_string(),
            doc_type: unsigned_mdoc.doc_type.clone(),
            value_digests: (&attrs).try_into()?,
            device_key_info: response.public_key.clone().into(),
            validity_info: validity.clone(),
        };

        // Presence of the key in the keyring has already been checked by new_session().
        let key = self.keys.private_key(&unsigned_mdoc.doc_type).unwrap();

        let headers = HeaderBuilder::new()
            .value(33, Value::Bytes(key.cert_bts.clone()))
            .build();
        let cose: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>> = MdocCose::sign(&mso.into(), headers, key)?;

        let signed = SparseIssuerSigned {
            randoms: attrs
                .into_iter()
                .map(|(namespace, attrs)| (namespace, Self::attr_randoms(attrs)))
                .collect(),
            sparse_issuer_auth: SparseIssuerAuth {
                version: "1.0".to_string(),
                digest_algorithm: "SHA-256".to_string(),
                validity_info: validity,
                issuer_auth: cose.clone_without_payload(),
            },
        };
        Ok(signed)
    }

    fn attr_randoms(attrs: Attributes) -> Vec<ByteBuf> {
        attrs.0.into_iter().map(|attr| attr.0.random).collect()
    }

    fn issue_creds(
        &self,
        doctype_responses: &MdocResponses,
        unsigned: &UnsignedMdoc,
    ) -> Result<Vec<SparseIssuerSigned>> {
        doctype_responses
            .responses
            .iter()
            .map(|response| self.issue_cred(unsigned, response))
            .collect()
    }

    pub fn issue(&self, device_response: &KeyGenerationResponseMessage) -> Result<DataToIssueMessage> {
        device_response.verify(&self.session_data.request)?;

        let docs = device_response
            .mdoc_responses
            .iter()
            .zip(&self.session_data.request.unsigned_mdocs)
            .map(|(responses, unsigned)| {
                let docs = MobileIDDocuments {
                    doc_type: unsigned.doc_type.clone(),
                    sparse_issuer_signed: self.issue_creds(responses, unsigned)?,
                };
                Ok(docs)
            })
            .collect::<Result<Vec<MobileIDDocuments>>>()?;

        let response = DataToIssueMessage {
            e_session_id: self.session_data.request.e_session_id.clone(),
            mobile_id_documents: docs,
        };
        Ok(response)
    }
}

/// Response messages in the protocol from the issuer to the holder.
// We use `typetag::serde` so that we can handle instances of `dyn IssuerResponse` in the code above;
// this isn't possible if we would just write `pub trait IssuerResponse: Serialize` because the `Serialize` trait
// is not "object safe".
#[typetag::serde(tag = "type")]
pub trait IssuerResponse: std::fmt::Debug {}

#[typetag::serde]
impl IssuerResponse for ReadyToProvisionMessage {}

#[typetag::serde]
impl IssuerResponse for RequestKeyGenerationMessage {}

#[typetag::serde]
impl IssuerResponse for DataToIssueMessage {}

#[typetag::serde]
impl IssuerResponse for EndSessionMessage {}
