//! Issuer software, for issuing mdocs to holders using an issuance private key.
//! See [`Server::new_session()`], which takes the mdocs to be issued and returns a [`ServiceEngagement`] to present to
//! the holder.

use core::panic;
use std::{future::Future, sync::Arc, time::Duration};

use chrono::Utc;
use ciborium::value::Value;
use coset::{CoseSign1, HeaderBuilder};
use futures::future::try_join_all;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_bytes::ByteBuf;
use tokio::task::JoinHandle;
use url::Url;

use wallet_common::utils::random_bytes;

use crate::{
    basic_sa_ext::{
        DataToIssueMessage, KeyGenerationResponseMessage, MdocResponses, MobileeIDDocuments,
        RequestKeyGenerationMessage, Response, SparseIssuerAuth, SparseIssuerSigned, StartIssuingMessage, UnsignedMdoc,
        KEY_GEN_RESP_MSG_TYPE, START_ISSUING_MSG_TYPE,
    },
    iso::*,
    issuer_shared::IssuanceError,
    server_keys::{KeyPair, KeyRing},
    server_state::{SessionState, SessionStore, SessionToken, CLEANUP_INTERVAL_SECONDS},
    utils::{
        cose::{ClonePayload, CoseKey, MdocCose, COSE_X5CHAIN_HEADER_LABEL},
        serialization::{cbor_deserialize, cbor_serialize, TaggedBytes},
    },
    Error, Result,
};

#[derive(Debug, Clone)]
enum IssuanceStatus {
    Created,
    Started,
    WaitingForResponse,
    Done,
    Failed,
    Cancelled,
}

use IssuanceStatus::*;

impl IssuanceStatus {
    fn update(&mut self, new_state: IssuanceStatus) {
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

pub struct Issuer<K, S> {
    url: Url,
    keys: K,
    sessions: Arc<S>,
    cleanup_task: JoinHandle<()>,
}

impl<K, S> Drop for Issuer<K, S> {
    fn drop(&mut self) {
        // Stop the task at the next .await
        self.cleanup_task.abort();
    }
}

impl<K, S> Issuer<K, S>
where
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>>,
{
    /// Construct a new issuance server. The `url` parameter should be the base URL at which the server is
    /// publically reachable; this is included in the [`ServiceEngagement`] that gets sent to the holder.
    pub fn new(url: Url, keys: K, session_store: S) -> Self
    where
        S: Send + Sync + 'static,
    {
        let sessions = Arc::new(session_store);
        Issuer {
            cleanup_task: sessions
                .clone()
                .start_cleanup_task(Duration::from_secs(CLEANUP_INTERVAL_SECONDS)),
            url,
            keys,
            sessions,
        }
    }

    /// Start a new issuance session for the specified (unsigned) mdocs. Returns the [`ServiceEngagement`] to be
    /// presented to the user.
    pub async fn new_session(&self, docs: Vec<UnsignedMdoc>) -> Result<ServiceEngagement> {
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
            .write(&SessionState::new(
                token.clone(),
                IssuanceData::new(request, session_id),
            ))
            .await
            .map_err(|e| IssuanceError::SessionStore(e.into()))?;

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
    pub async fn process_message(&self, token: SessionToken, msg: &[u8]) -> Result<Vec<u8>> {
        let (msg_type, session_id) = Self::inspect_message(msg)?;

        let mut session_data = self
            .sessions
            .get(&token)
            .await
            .map_err(|e| IssuanceError::SessionStore(e.into()))?
            .ok_or_else(|| Error::from(IssuanceError::UnknownSessionId(token.clone())))?;
        let session = Session {
            sessions: self.sessions.as_ref(),
            session_data: &mut session_data,
            keys: &self.keys,
        };

        // If this is not the very first protocol message, the session ID is expected in every message.
        if msg_type != START_PROVISIONING_MSG_TYPE {
            Self::expect_session_id(
                &session_id.ok_or(Error::from(IssuanceError::MissingSessionId))?,
                &session.session_data.session_data.id,
            )?;
        }

        // Stop the session if the user wishes.
        if msg_type == REQUEST_END_SESSION_MSG_TYPE {
            return Self::handle_cbor(Session::process_cancel, session, msg).await;
        }

        // Process the holder's message according to the current session state.
        match session.session_data.session_data.state {
            Created => {
                Self::expect_message_type(&msg_type, START_PROVISIONING_MSG_TYPE)?;
                Self::handle_cbor(Session::process_start, session, msg).await
            }
            Started => {
                Self::expect_message_type(&msg_type, START_ISSUING_MSG_TYPE)?;
                Self::handle_cbor(Session::process_get_request, session, msg).await
            }
            WaitingForResponse => {
                Self::expect_message_type(&msg_type, KEY_GEN_RESP_MSG_TYPE)?;
                Self::handle_cbor(Session::process_response, session, msg).await
            }
            Done | Failed | Cancelled => Err(IssuanceError::SessionEnded.into()),
        }
    }

    /// For a `Session` method that takes and returns parameters of any type that implement `Serialize`/`Deserialize`,
    /// transparently handle CBOR encoding and decoding of its (return) parameters.
    async fn handle_cbor<'a, V: DeserializeOwned, R: Serialize, F>(
        func: impl FnOnce(Session<'a, K, S>, V) -> F,
        session: Session<'a, K, S>,
        msg_bts: &[u8],
    ) -> Result<Vec<u8>>
    where
        F: Future<Output = Result<R>>,
    {
        Ok(cbor_serialize(&func(session, cbor_deserialize(msg_bts)?).await?)?)
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
struct Session<'a, K, S>
where
    S: SessionStore<Data = SessionState<IssuanceData>>,
{
    sessions: &'a S,
    session_data: &'a mut SessionState<IssuanceData>,
    keys: &'a K,
}

#[derive(Debug, Clone)]
pub struct IssuanceData {
    request: RequestKeyGenerationMessage,
    id: SessionId,
    state: IssuanceStatus,
}

impl IssuanceData {
    fn new(request: RequestKeyGenerationMessage, id: SessionId) -> Self {
        Self {
            request,
            id,
            state: Created,
        }
    }
}

// The `process_` methods process specific issuance protocol messages from the holder.
impl<'a, K: KeyRing, S> Session<'a, K, S>
where
    S: SessionStore<Data = SessionState<IssuanceData>>,
{
    async fn update_state(&mut self, new_state: IssuanceStatus) -> Result<()> {
        self.session_data.session_data.state.update(new_state);
        self.session_data.last_active = Utc::now();
        Ok(self
            .sessions
            .write(self.session_data)
            .await
            .map_err(|e| IssuanceError::SessionStore(Box::new(e)))?)
    }

    async fn process_start(mut self, _: StartProvisioningMessage) -> Result<ReadyToProvisionMessage> {
        self.update_state(Started).await?;
        let response = ReadyToProvisionMessage {
            e_session_id: self.session_data.session_data.request.e_session_id.clone(),
        };
        Ok(response)
    }

    async fn process_get_request(mut self, _: StartIssuingMessage) -> Result<RequestKeyGenerationMessage> {
        self.update_state(WaitingForResponse).await?;
        Ok(self.session_data.session_data.request.clone())
    }

    async fn process_response(mut self, device_response: KeyGenerationResponseMessage) -> Result<DataToIssueMessage> {
        let issuance_result = self.issue(device_response).await;
        match issuance_result {
            Ok(_) => self.update_state(Done).await?,
            Err(_) => self.update_state(Failed).await?,
        }
        issuance_result
    }

    async fn process_cancel(mut self, _: RequestEndSessionMessage) -> Result<EndSessionMessage> {
        self.update_state(Cancelled).await?;
        let response = EndSessionMessage {
            e_session_id: self.session_data.session_data.request.e_session_id.clone(),
            reason: "success".to_string(),
            delay: None,
            sed: None,
        };
        Ok(response)
    }

    async fn issue_cred(&self, unsigned_mdoc: UnsignedMdoc, response: Response) -> Result<SparseIssuerSigned> {
        // Presence of the key in the keyring has already been checked by new_session().
        let private_key = self.keys.private_key(&unsigned_mdoc.doc_type).unwrap();

        let (signed, mso) = IssuerSigned::sign(unsigned_mdoc, response.public_key, private_key).await?;

        let sparse = SparseIssuerSigned {
            randoms: signed
                .name_spaces
                .unwrap_or_default()
                .into_iter()
                .map(|(namespace, attrs)| (namespace, Self::attr_randoms(attrs)))
                .collect(),
            sparse_issuer_auth: SparseIssuerAuth {
                version: mso.version,
                digest_algorithm: mso.digest_algorithm,
                validity_info: mso.validity_info,
                issuer_auth: signed.issuer_auth.clone_without_payload(),
            },
        };

        Ok(sparse)
    }

    fn attr_randoms(attrs: Attributes) -> Vec<ByteBuf> {
        attrs.0.into_iter().map(|attr| attr.0.random).collect()
    }

    async fn issue_creds(
        &self,
        doctype_responses: MdocResponses,
        unsigned: &UnsignedMdoc,
    ) -> Result<Vec<SparseIssuerSigned>> {
        let issue_creds = try_join_all(
            doctype_responses
                .responses
                .into_iter()
                .map(|response| self.issue_cred(unsigned.clone(), response)),
        )
        .await?;

        Ok(issue_creds)
    }

    pub async fn issue(&self, device_response: KeyGenerationResponseMessage) -> Result<DataToIssueMessage> {
        device_response.verify(&self.session_data.session_data.request)?;

        let mut docs = vec![];
        for (responses, unsigned) in device_response
            .mdoc_responses
            .into_iter()
            .zip(&self.session_data.session_data.request.unsigned_mdocs)
        {
            docs.push(MobileeIDDocuments {
                doc_type: unsigned.doc_type.clone(),
                sparse_issuer_signed: self.issue_creds(responses, unsigned).await?,
            })
        }

        let response = DataToIssueMessage {
            e_session_id: self.session_data.session_data.request.e_session_id.clone(),
            mobile_eid_documents: docs,
        };
        Ok(response)
    }
}

impl IssuerSigned {
    pub async fn sign(
        unsigned_mdoc: UnsignedMdoc,
        device_public_key: CoseKey,
        key: &KeyPair,
    ) -> Result<(Self, MobileSecurityObject)> {
        let now = Utc::now();
        let validity = ValidityInfo {
            signed: now.into(),
            valid_from: unsigned_mdoc.valid_from,
            valid_until: unsigned_mdoc.valid_until,
            expected_update: None,
        };

        let doc_type = unsigned_mdoc.doc_type;
        let attrs: IssuerNameSpaces = unsigned_mdoc
            .attributes
            .into_iter()
            .map(|(namespace, attrs)| (namespace, Attributes::from(attrs)))
            .collect();

        let mso = MobileSecurityObject {
            version: MobileSecurityObjectVersion::V1_0,
            digest_algorithm: DigestAlgorithm::SHA256,
            doc_type,
            value_digests: (&attrs).try_into()?,
            device_key_info: device_public_key.into(),
            validity_info: validity,
        };

        let headers = HeaderBuilder::new()
            .value(
                COSE_X5CHAIN_HEADER_LABEL,
                Value::Bytes(key.certificate().as_bytes().to_vec()),
            )
            .build();
        let mso_tagged = mso.into();
        let issuer_auth: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>> =
            MdocCose::sign(&mso_tagged, headers, key, true).await?;

        let issuer_signed = IssuerSigned {
            name_spaces: attrs.into(),
            issuer_auth,
        };

        Ok((issuer_signed, mso_tagged.0))
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use serde_bytes::ByteBuf;
    use wallet_common::utils::random_string;

    use crate::{
        basic_sa_ext::RequestKeyGenerationMessage,
        server_keys::KeyPair,
        server_state::{MemorySessionStore, SessionStore},
    };

    use super::{IssuanceData, KeyRing, SessionState};

    struct EmptyKeyRing;
    impl KeyRing for EmptyKeyRing {
        fn private_key(&self, _: &str) -> Option<&KeyPair> {
            None
        }
    }

    type Server = super::Issuer<EmptyKeyRing, MemorySessionStore<IssuanceData>>;

    const CLEANUP_INTERVAL: Duration = Duration::from_millis(50);

    #[tokio::test]
    async fn session_cleanup() {
        // Construct a `Server`, but not using Server::new() so we can control our own cleanup task
        let sessions = Arc::new(MemorySessionStore::new());
        let server = Server {
            cleanup_task: MemorySessionStore::<IssuanceData>::start_cleanup_task(
                Arc::clone(&sessions),
                CLEANUP_INTERVAL,
            ),
            url: "https://example.com".parse().unwrap(),
            keys: EmptyKeyRing,
            sessions,
        };

        // insert a fresh session
        let session_data = dummy_session_data();
        server.sessions.write(&session_data).await.unwrap();
        assert_eq!(server.sessions.sessions.len(), 1);

        // wait at least one duration: session should still be here
        tokio::time::sleep(2 * CLEANUP_INTERVAL).await;
        assert_eq!(server.sessions.sessions.len(), 1);

        // insert a stale session
        let mut session_data = dummy_session_data();
        session_data.last_active = chrono::Utc::now() - chrono::Duration::hours(1);
        server.sessions.write(&session_data).await.unwrap();
        assert_eq!(server.sessions.sessions.len(), 2);

        // wait at least one duration: stale session should be removed
        tokio::time::sleep(2 * CLEANUP_INTERVAL).await;
        assert_eq!(server.sessions.sessions.len(), 1)
    }

    #[tokio::test]
    async fn cleanup_task_cleanup() {
        let sessions = Arc::new(MemorySessionStore::new());

        {
            // Construct a `Server`, but not using Server::new() so we can keep our sessions reference
            // and control our own cleanup task.
            let server = Server {
                cleanup_task: MemorySessionStore::<IssuanceData>::start_cleanup_task(
                    Arc::clone(&sessions),
                    CLEANUP_INTERVAL,
                ),
                url: "https://example.com".parse().unwrap(),
                keys: EmptyKeyRing,
                sessions: Arc::clone(&sessions),
            };

            // insert a stale session
            let mut session_data = dummy_session_data();
            session_data.last_active = chrono::Utc::now() - chrono::Duration::hours(1);
            server.sessions.write(&session_data).await.unwrap();
            assert_eq!(server.sessions.sessions.len(), 1);

            // Drop the server and its cleanup task before giving the cleanup task a chance to run
        }

        // wait at least one duration: stale session should not be removed since the cleanup task has been stopped.
        tokio::time::sleep(CLEANUP_INTERVAL).await;
        assert_eq!(sessions.sessions.len(), 1)
    }

    fn dummy_session_data() -> SessionState<IssuanceData> {
        SessionState::new(
            random_string(32).into(),
            IssuanceData::new(
                RequestKeyGenerationMessage {
                    e_session_id: "123".to_string().as_bytes().to_vec().into(),
                    challenge: ByteBuf::from(vec![]),
                    unsigned_mdocs: vec![],
                },
                "123".to_string().as_bytes().to_vec().into(),
            ),
        )
    }
}
