use core::panic;

use chrono::Utc;
use ciborium::value::Value;
use coset::{CoseSign1, HeaderBuilder};
use dashmap::DashMap;
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
    issuer_shared::IssuanceError,
    serialization::{cbor_deserialize, TaggedBytes},
    Error, Result,
};

#[derive(Debug)]
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
            Created => assert!(matches!(new_state, Started | Done | Failed | Cancelled)),
            Started => assert!(matches!(new_state, WaitingForResponse | Done | Failed | Cancelled)),
            WaitingForResponse => assert!(matches!(new_state, Done | Failed | Cancelled)),
            Done => panic!("can't update final state"),
            Failed => panic!("can't update final state"),
            Cancelled => panic!("can't update final state"),
        }
        *self = new_state;
    }
}

struct Session {
    issuer: Issuer,
    state: SessionState,
}

/// An issuance session. The `process_` methods process specific issuance protocol messages from the holder.
impl Session {
    fn process_start(&mut self, _: StartProvisioningMessage) -> ReadyToProvisionMessage {
        self.state.update(Started);
        ReadyToProvisionMessage {
            e_session_id: self.issuer.request.e_session_id.clone(),
        }
    }

    fn process_get_request(&mut self, _: StartIssuingMessage) -> RequestKeyGenerationMessage {
        self.state.update(WaitingForResponse);
        self.issuer.request.clone()
    }

    fn process_response(&mut self, device_response: KeyGenerationResponseMessage) -> Result<DataToIssueMessage> {
        let issuance_result = self.issuer.issue(&device_response);
        match issuance_result {
            Ok(_) => self.state.update(Done),
            Err(_) => self.state.update(Failed),
        }
        issuance_result
    }

    fn process_cancel(&mut self) -> EndSessionMessage {
        self.state.update(Cancelled);
        EndSessionMessage {
            e_session_id: self.issuer.request.e_session_id.clone(),
            reason: "success".to_string(),
            delay: None,
            sed: None,
        }
    }
}

#[derive(Default)]
pub struct Server {
    sessions: DashMap<SessionId, Session>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            sessions: DashMap::new(),
        }
    }

    pub fn new_session(
        &mut self,
        docs: Vec<UnsignedMdoc>,
        private_key: ecdsa::SigningKey<p256::NistP256>,
        cert_bts: Vec<u8>,
    ) -> SessionId {
        let challenge = ByteBuf::from(random_bytes(32));
        let session_id: SessionId = ByteBuf::from(random_bytes(32)).into();
        let request = RequestKeyGenerationMessage {
            e_session_id: session_id.clone(),
            challenge,
            unsigned_mdocs: docs,
        };
        self.sessions.insert(
            session_id.clone(),
            Session {
                state: Created,
                issuer: Issuer {
                    private_key,
                    cert_bts,
                    request,
                },
            },
        );
        session_id
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

    /// Process a CBOR-encoded issuance protocol message from the holder.
    pub fn process_message(&self, id: SessionId, msg: Vec<u8>) -> Result<Box<dyn IssuerResponse>> {
        let (msg_type, session_id) = Self::inspect_message(&msg)?;
        if msg_type != START_PROVISIONING_MSG_TYPE {
            Self::expect_session_id(&session_id.ok_or(Error::from(IssuanceError::MissingSessionId))?, &id)?;
        }

        let mut session = self
            .sessions
            .get_mut(&id)
            .ok_or(Error::from(IssuanceError::UnknownSessionId(id)))?;

        if msg_type == REQUEST_END_SESSION_MSG_TYPE {
            return Ok(Box::new(session.process_cancel()));
        }

        match session.state {
            Created => {
                Self::expect_message_type(&msg_type, START_PROVISIONING_MSG_TYPE)?;
                Ok(Box::new(session.process_start(cbor_deserialize(&msg[..])?)))
            }
            Started => {
                Self::expect_message_type(&msg_type, START_ISSUING_MSG_TYPE)?;
                Ok(Box::new(session.process_get_request(cbor_deserialize(&msg[..])?)))
            }
            WaitingForResponse => {
                Self::expect_message_type(&msg_type, KEY_GEN_RESP_MSG_TYPE)?;
                Ok(Box::new(session.process_response(cbor_deserialize(&msg[..])?)?))
            }
            Done | Failed | Cancelled => Err(IssuanceError::SessionEnded.into()),
        }
    }
}

pub struct Issuer {
    private_key: ecdsa::SigningKey<p256::NistP256>,
    cert_bts: Vec<u8>,
    request: RequestKeyGenerationMessage,
}

impl Issuer {
    pub fn new(
        request: RequestKeyGenerationMessage,
        private_key: ecdsa::SigningKey<p256::NistP256>,
        cert_bts: Vec<u8>,
    ) -> Issuer {
        Issuer {
            request,
            cert_bts,
            private_key,
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

        let headers = HeaderBuilder::new()
            .value(33, Value::Bytes(self.cert_bts.clone()))
            .build();
        let cose: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>> =
            MdocCose::sign(&mso.into(), headers, &self.private_key)?;

        Ok(SparseIssuerSigned {
            randoms: attrs
                .into_iter()
                .map(|(namespace, attrs)| (namespace, attrs.0.into_iter().map(|attr| attr.0.random).collect()))
                .collect(),
            sparse_issuer_auth: SparseIssuerAuth {
                version: "1.0".to_string(),
                digest_algorithm: "SHA-256".to_string(),
                validity_info: validity,
                issuer_auth: cose.clone_without_payload(),
            },
        })
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
        device_response.verify(&self.request)?;

        let docs = device_response
            .mdoc_responses
            .iter()
            .zip(&self.request.unsigned_mdocs)
            .map(|(responses, unsigned)| {
                Ok(MobileIDDocuments {
                    doc_type: unsigned.doc_type.clone(),
                    sparse_issuer_signed: self.issue_creds(responses, unsigned)?,
                })
            })
            .collect::<Result<Vec<MobileIDDocuments>>>()?;

        Ok(DataToIssueMessage {
            e_session_id: self.request.e_session_id.clone(),
            mobile_id_documents: docs,
        })
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
