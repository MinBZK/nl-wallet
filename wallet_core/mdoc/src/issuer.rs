//! Issuer software, for issuing mdocs to holders using an issuance private key.
//! See [`Server::new_session()`], which takes the mdocs to be issued and returns a [`ServiceEngagement`] to present to
//! the holder.

use core::panic;
use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Local, Utc};
use ciborium::value::Value;
use coset::{CoseSign1, HeaderBuilder};
use dashmap::DashMap;
use p256::{
    ecdsa::{signature::Signer, Signature, SigningKey},
    pkcs8::DecodePrivateKey,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_bytes::ByteBuf;
use tokio::{task::JoinHandle, time};
use url::Url;

use wallet_common::{
    keys::{EcdsaKey, SecureEcdsaKey},
    utils::random_bytes,
};

use crate::{
    basic_sa_ext::{
        DataToIssueMessage, KeyGenerationResponseMessage, MdocResponses, MobileeIDDocuments,
        RequestKeyGenerationMessage, Response, SparseIssuerAuth, SparseIssuerSigned, StartIssuingMessage, UnsignedMdoc,
        KEY_GEN_RESP_MSG_TYPE, START_ISSUING_MSG_TYPE,
    },
    iso::*,
    issuer_shared::{IssuanceError, SessionToken},
    utils::{
        cose::{ClonePayload, MdocCose, COSE_X5CHAIN_HEADER_LABEL},
        serialization::{cbor_deserialize, cbor_serialize, TaggedBytes},
        x509::Certificate,
    },
    Error, Result,
};

pub struct PrivateKey {
    private_key: SigningKey,
    cert_bts: Certificate,
}

impl PrivateKey {
    pub fn new(private_key: SigningKey, cert_bts: Certificate) -> PrivateKey {
        PrivateKey { private_key, cert_bts }
    }

    pub fn from_der(private_key: &[u8], cert: &[u8]) -> Result<PrivateKey> {
        let key = Self::new(
            SigningKey::from_pkcs8_der(private_key).map_err(IssuanceError::DerPrivateKey)?,
            Certificate::from(cert),
        );
        Ok(key)
    }
}

impl Signer<Signature> for PrivateKey {
    fn try_sign(&self, msg: &[u8]) -> std::result::Result<Signature, p256::ecdsa::Error> {
        self.private_key.try_sign(msg)
    }
}
impl EcdsaKey for PrivateKey {
    type Error = p256::ecdsa::Error;
    fn verifying_key(&self) -> std::result::Result<p256::ecdsa::VerifyingKey, Self::Error> {
        Ok(*self.private_key.verifying_key())
    }
}
impl SecureEcdsaKey for PrivateKey {}

pub trait KeyRing {
    fn private_key(&self, doctype: &DocType) -> Option<&PrivateKey>;
    fn contains_key(&self, doctype: &DocType) -> bool {
        self.private_key(doctype).is_some()
    }
}

/// An implementation of [`KeyRing`] containing a single key.
pub struct SingleKeyRing(pub PrivateKey);

impl KeyRing for SingleKeyRing {
    fn private_key(&self, _: &DocType) -> Option<&PrivateKey> {
        Some(&self.0)
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
    fn cleanup(&self);
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

/// After this amount of inactivity, a session should be cleaned up.
const SESSION_EXPIRY_MINUTES: u64 = 5;

/// The cleanup task that removes stale sessions runs every so often.
const CLEANUP_INTERVAL_SECONDS: u64 = 10;

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

    fn cleanup(&self) {
        let now = Local::now();
        let cutoff = chrono::Duration::minutes(SESSION_EXPIRY_MINUTES as i64);
        self.sessions.retain(|_, session| now - session.last_active < cutoff);
    }
}

pub struct Server<K, S> {
    url: Url,
    keys: K,
    sessions: Arc<S>,
    cleanup_task: JoinHandle<()>,
}

impl<K, S> Drop for Server<K, S> {
    fn drop(&mut self) {
        // Stop the task at the next .await
        self.cleanup_task.abort();
    }
}

impl<K: KeyRing, S: SessionStore + Send + Sync + 'static> Server<K, S> {
    /// Construct a new issuance server. The `url` parameter should be the base URL at which the server is
    /// publically reachable; this is included in the [`ServiceEngagement`] that gets sent to the holder.
    pub fn new(url: Url, keys: K, session_store: S) -> Self {
        let sessions = Arc::new(session_store);
        Server {
            cleanup_task: Self::start_cleanup_task(
                Arc::clone(&sessions),
                Duration::from_secs(CLEANUP_INTERVAL_SECONDS),
            ),
            url,
            keys,
            sessions,
        }
    }

    fn start_cleanup_task(sessions: Arc<S>, interval: Duration) -> JoinHandle<()> {
        let mut interval = time::interval(interval);
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                sessions.cleanup();
            }
        })
    }

    /// Start a new issuance session for the specified (unsigned) mdocs. Returns the [`ServiceEngagement`] to be
    /// presented to the user.
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

        self.sessions
            .write(&SessionData::new(token.clone(), session_id, request));

        let url = self.url.join(&token.0).unwrap(); // token is alphanumeric so this will always succeed
        Ok(ServiceEngagement {
            url: Some(url),
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
    pub fn process_message(&self, token: SessionToken, msg: &[u8]) -> Result<Vec<u8>> {
        let (msg_type, session_id) = Self::inspect_message(msg)?;

        let mut session_data = self.sessions.get(&token)?;
        let session = Session {
            sessions: self.sessions.as_ref(),
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
            return Self::handle_cbor(Session::process_cancel, session, msg);
        }

        // Process the holder's message according to the current session state.
        match session.session_data.state {
            Created => {
                Self::expect_message_type(&msg_type, START_PROVISIONING_MSG_TYPE)?;
                Self::handle_cbor(Session::process_start, session, msg)
            }
            Started => {
                Self::expect_message_type(&msg_type, START_ISSUING_MSG_TYPE)?;
                Self::handle_cbor(Session::process_get_request, session, msg)
            }
            WaitingForResponse => {
                Self::expect_message_type(&msg_type, KEY_GEN_RESP_MSG_TYPE)?;
                Self::handle_cbor(Session::process_response, session, msg)
            }
            Done | Failed | Cancelled => Err(IssuanceError::SessionEnded.into()),
        }
    }

    /// For a `Session` method that takes and returns parameters of any type that implement `Serialize`/`Deserialize`,
    /// transparently handle CBOR encoding and decoding of its (return) parameters.
    fn handle_cbor<'a, V: DeserializeOwned, R: Serialize>(
        func: impl Fn(Session<'a, K, S>, V) -> Result<R>,
        session: Session<'a, K, S>,
        msg_bts: &[u8],
    ) -> Result<Vec<u8>> {
        Ok(cbor_serialize(&func(session, cbor_deserialize(msg_bts)?)?)?)
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
    last_active: DateTime<Local>,
}

impl SessionData {
    fn new(token: SessionToken, id: SessionId, request: RequestKeyGenerationMessage) -> SessionData {
        SessionData {
            token,
            request,
            id,
            state: Created,
            last_active: Local::now(),
        }
    }
}

// The `process_` methods process specific issuance protocol messages from the holder.
impl<'a, K: KeyRing, S: SessionStore> Session<'a, K, S> {
    fn update_state(&mut self, new_state: SessionState) {
        self.session_data.state.update(new_state);
        self.session_data.last_active = Local::now();
        self.updated = true;
    }

    fn process_start(mut self, _: StartProvisioningMessage) -> Result<ReadyToProvisionMessage> {
        self.update_state(Started);
        let response = ReadyToProvisionMessage {
            e_session_id: self.session_data.request.e_session_id.clone(),
        };
        Ok(response)
    }

    fn process_get_request(mut self, _: StartIssuingMessage) -> Result<RequestKeyGenerationMessage> {
        self.update_state(WaitingForResponse);
        Ok(self.session_data.request.clone())
    }

    fn process_response(mut self, device_response: KeyGenerationResponseMessage) -> Result<DataToIssueMessage> {
        let issuance_result = self.issue(&device_response);
        match issuance_result {
            Ok(_) => self.update_state(Done),
            Err(_) => self.update_state(Failed),
        }
        issuance_result
    }

    fn process_cancel(mut self, _: RequestEndSessionMessage) -> Result<EndSessionMessage> {
        self.update_state(Cancelled);
        let response = EndSessionMessage {
            e_session_id: self.session_data.request.e_session_id.clone(),
            reason: "success".to_string(),
            delay: None,
            sed: None,
        };
        Ok(response)
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
            valid_from: unsigned_mdoc.valid_from.clone(),
            valid_until: unsigned_mdoc.valid_until.clone(),
            expected_update: None,
        };
        let mso = MobileSecurityObject {
            version: MobileSecurityObjectVersion::V1_0,
            digest_algorithm: DigestAlgorithm::SHA256,
            doc_type: unsigned_mdoc.doc_type.clone(),
            value_digests: (&attrs).try_into()?,
            device_key_info: response.public_key.clone().into(),
            validity_info: validity.clone(),
        };

        // Presence of the key in the keyring has already been checked by new_session().
        let key = self.keys.private_key(&unsigned_mdoc.doc_type).unwrap();

        let headers = HeaderBuilder::new()
            .value(
                COSE_X5CHAIN_HEADER_LABEL,
                Value::Bytes(key.cert_bts.as_bytes().to_vec()),
            )
            .build();
        let cose: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>> = MdocCose::sign(&mso.into(), headers, key)?;

        let signed = SparseIssuerSigned {
            randoms: attrs
                .into_iter()
                .map(|(namespace, attrs)| (namespace, Self::attr_randoms(attrs)))
                .collect(),
            sparse_issuer_auth: SparseIssuerAuth {
                version: MobileSecurityObjectVersion::V1_0,
                digest_algorithm: DigestAlgorithm::SHA256,
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
                let docs = MobileeIDDocuments {
                    doc_type: unsigned.doc_type.clone(),
                    sparse_issuer_signed: self.issue_creds(responses, unsigned)?,
                };
                Ok(docs)
            })
            .collect::<Result<Vec<MobileeIDDocuments>>>()?;

        let response = DataToIssueMessage {
            e_session_id: self.session_data.request.e_session_id.clone(),
            mobile_eid_documents: docs,
        };
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use serde_bytes::ByteBuf;
    use wallet_common::utils::random_string;

    use crate::{basic_sa_ext::RequestKeyGenerationMessage, DocType};

    use super::{KeyRing, MemorySessionStore, PrivateKey, SessionData, SessionStore};

    struct EmptyKeyRing;
    impl KeyRing for EmptyKeyRing {
        fn private_key(&self, _: &DocType) -> Option<&PrivateKey> {
            None
        }
    }

    type Server = super::Server<EmptyKeyRing, MemorySessionStore>;

    const CLEANUP_INTERVAL: Duration = Duration::from_millis(50);

    #[tokio::test]
    async fn session_cleanup() {
        // Construct a `Server`, but not using Server::new() so we can control our own cleanup task
        let sessions = Arc::new(MemorySessionStore::new());
        let server = Server {
            cleanup_task: Server::start_cleanup_task(Arc::clone(&sessions), CLEANUP_INTERVAL),
            url: "https://example.com".parse().unwrap(),
            keys: EmptyKeyRing,
            sessions,
        };

        // insert a fresh session
        let session_data = dummy_session_data();
        server.sessions.write(&session_data);
        assert_eq!(server.sessions.sessions.len(), 1);

        // wait at least one duration: session should still be here
        tokio::time::sleep(CLEANUP_INTERVAL).await;
        assert_eq!(server.sessions.sessions.len(), 1);

        // insert a stale session
        let mut session_data = dummy_session_data();
        session_data.last_active = chrono::Local::now() - chrono::Duration::hours(1);
        server.sessions.write(&session_data);
        assert_eq!(server.sessions.sessions.len(), 2);

        // wait at least one duration: stale session should be removed
        tokio::time::sleep(CLEANUP_INTERVAL).await;
        assert_eq!(server.sessions.sessions.len(), 1)
    }

    #[tokio::test]
    async fn cleanup_task_cleanup() {
        let sessions = Arc::new(MemorySessionStore::new());

        {
            // Construct a `Server`, but not using Server::new() so we can keep our sessions reference
            // and control our own cleanup task.
            let server = Server {
                cleanup_task: Server::start_cleanup_task(Arc::clone(&sessions), CLEANUP_INTERVAL),
                url: "https://example.com".parse().unwrap(),
                keys: EmptyKeyRing,
                sessions: Arc::clone(&sessions),
            };

            // insert a stale session
            let mut session_data = dummy_session_data();
            session_data.last_active = chrono::Local::now() - chrono::Duration::hours(1);
            server.sessions.write(&session_data);
            assert_eq!(server.sessions.sessions.len(), 1);

            // Drop the server and its cleanup task before giving the cleanup task a chance to run
        }

        // wait at least one duration: stale session should not be removed since the cleanup task has been stopped.
        tokio::time::sleep(CLEANUP_INTERVAL).await;
        assert_eq!(sessions.sessions.len(), 1)
    }

    fn dummy_session_data() -> SessionData {
        SessionData::new(
            random_string(32).into(),
            "123".to_string().as_bytes().to_vec().into(),
            RequestKeyGenerationMessage {
                e_session_id: "123".to_string().as_bytes().to_vec().into(),
                challenge: ByteBuf::from(vec![]),
                unsigned_mdocs: vec![],
            },
        )
    }
}