//! RP software, for verifying mdoc disclosures, see [`DeviceResponse::verify()`].

use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use indexmap::IndexMap;
use p256::SecretKey;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use strum;
use tokio::task::JoinHandle;
use url::Url;
use webpki::TrustAnchor;

use wallet_common::{
    account::serialization::DerSecretKey,
    generator::{Generator, TimeGenerator},
    trust_anchor::OwnedTrustAnchor,
    utils,
};

use crate::{
    basic_sa_ext::Entry,
    identifiers::{AttributeIdentifier, AttributeIdentifierHolder},
    iso::*,
    server_keys::{KeyPair, KeyRing},
    server_state::{SessionState, SessionStore, SessionStoreError, SessionToken, CLEANUP_INTERVAL_SECONDS},
    utils::{
        cose::{self, ClonePayload, MdocCose},
        crypto::{cbor_digest, dh_hmac_key, SessionKey, SessionKeyUser},
        serialization::{cbor_deserialize, cbor_hex, cbor_serialize, CborSeq, TaggedBytes},
        x509::CertificateUsage,
    },
    Error, Result, SessionData,
};

/// Attributes of an mdoc that was disclosed in a [`DeviceResponse`], as computed by [`DeviceResponse::verify()`].
/// Grouped per namespace.
pub type DocumentDisclosedAttributes = IndexMap<NameSpace, Vec<Entry>>;
/// All attributes that were disclosed in a [`DeviceResponse`], as computed by [`DeviceResponse::verify()`].
pub type DisclosedAttributes = IndexMap<DocType, DocumentDisclosedAttributes>;

#[derive(thiserror::Error, Debug)]
pub enum VerificationError {
    #[error("errors in device response: {0:#?}")]
    DeviceResponseErrors(Vec<DocumentError>),
    #[error("unexpected status: {0}")]
    UnexpectedStatus(u64),
    #[error("no documents found in device response")]
    NoDocuments,
    #[error("inconsistent doctypes: document contained {document}, mso contained {mso}")]
    WrongDocType { document: DocType, mso: DocType },
    #[error("namespace {0} not found in mso")]
    MissingNamespace(NameSpace),
    #[error("digest ID {0} not found in mso")]
    MissingDigestID(DigestID),
    #[error("attribute verification failed: did not hash to the value in the MSO")]
    AttributeVerificationFailed,
    #[error("missing ephemeral key")]
    EphemeralKeyMissing,
    #[error("validity error: {0}")]
    Validity(#[from] ValidityError),
    #[error("missing OriginInfo in engagement: {0}")]
    MissingOriginInfo(usize),
    #[error("incorrect OriginInfo in engagement")]
    IncorrectOriginInfo,
    #[error("unexpected input: session is done")]
    UnexpectedInput,
    #[error("unknown certificate")]
    UnknownCertificate(String),
    #[error("unknown session ID: {0}")]
    UnknownSessionId(SessionToken),
    #[error("no ItemsRequest: can't request a disclosure of 0 attributes")]
    NoItemsRequests,
    #[error("attributes mismatch: {0:?}")]
    MissingAttributes(Vec<AttributeIdentifier>),
    #[error("error with sessionstore: {0}")]
    SessionStore(SessionStoreError),
    #[error("disclosed attributes requested for disclosure session with status: {0}")]
    SessionNotDone(StatusResponse),
    #[error("transcript hash '{0:?}' does not match expected")]
    TranscriptHashMismatch(Option<Vec<u8>>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemsRequests(pub Vec<ItemsRequest>);
impl From<Vec<ItemsRequest>> for ItemsRequests {
    fn from(value: Vec<ItemsRequest>) -> Self {
        Self(value)
    }
}

/// A disclosure session. `S` must implement [`DisclosureState`] and is the state that the session is in.
/// The session progresses through the possible states using a state engine that uses the typestate pattern:
/// for each state `S`, `Session<S>` has its own state transition method that consume the previous state.
#[derive(Debug)]
struct Session<S> {
    state: SessionState<S>,
}

/// State for a session that has just been created.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Created {
    items_requests: ItemsRequests,
    session_type: SessionType,
    usecase_id: String,
    return_url_used: bool,
    ephemeral_privkey: DerSecretKey,
    #[serde(with = "cbor_hex")]
    reader_engagement: ReaderEngagement,
}

/// State for a session that is waiting for the user's disclosure, i.e., the device has read our
/// [`ReaderEngagement`] and contacted us at the session URL.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WaitingForResponse {
    #[allow(unused)] // TODO write function that matches this field against the disclosed attributes
    items_requests: ItemsRequests,
    return_url_used: bool,
    their_key: SessionKey,
    ephemeral_privkey: DerSecretKey,
    #[serde(with = "cbor_hex")]
    session_transcript: SessionTranscript,
}

/// State for a session that has ended (for any reason).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Done {
    session_result: SessionResult,
}

/// The outcome of a session: the disclosed attributes if they have been sucessfully received and verified.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE", tag = "status")]
pub enum SessionResult {
    Done {
        disclosed_attributes: DisclosedAttributes,
        transcript_hash: Option<Vec<u8>>,
    },
    Failed {
        error: String,
    },
    Cancelled,
}

/// Disclosure session states for use as `T` in `Session<T>`.
pub trait DisclosureState {}

impl DisclosureState for Created {}
impl DisclosureState for WaitingForResponse {}
impl DisclosureState for Done {}

/// Disclosure-specific session data, of any state, for storing in a session store.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DisclosureData {
    Created(Created),
    WaitingForResponse(WaitingForResponse),
    Done(Done),
}

impl From<SessionState<Created>> for SessionState<DisclosureData> {
    fn from(value: SessionState<Created>) -> Self {
        SessionState {
            session_data: DisclosureData::Created(value.session_data),
            token: value.token,
            last_active: value.last_active,
        }
    }
}

impl From<SessionState<WaitingForResponse>> for SessionState<DisclosureData> {
    fn from(value: SessionState<WaitingForResponse>) -> Self {
        SessionState {
            session_data: DisclosureData::WaitingForResponse(value.session_data),
            token: value.token,
            last_active: value.last_active,
        }
    }
}

impl From<SessionState<Done>> for SessionState<DisclosureData> {
    fn from(value: SessionState<Done>) -> Self {
        SessionState {
            session_data: DisclosureData::Done(value.session_data),
            token: value.token,
            last_active: value.last_active,
        }
    }
}

/// status without the underlying data
#[derive(Debug, strum::Display, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE", tag = "status")]
pub enum StatusResponse {
    Created,
    WaitingForResponse,
    Done,
    Failed,
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SessionType {
    // Using Universal Link
    SameDevice,
    /// Using QR code
    CrossDevice,
}

pub struct Verifier<K, S> {
    url: Url,
    keys: K,
    sessions: Arc<S>,
    cleanup_task: JoinHandle<()>,
    trust_anchors: Vec<OwnedTrustAnchor>,
}

impl<K, S> Drop for Verifier<K, S> {
    fn drop(&mut self) {
        // Stop the task at the next .await
        self.cleanup_task.abort();
    }
}

impl<K, S> Verifier<K, S>
where
    K: KeyRing,
    S: SessionStore<Data = SessionState<DisclosureData>>,
{
    /// Create a new [`Verifier`].
    ///
    /// - `url` is the URL at which the server is publically reachable; this is included in the [`ReaderEngagement`]
    ///   returned to the wallet.
    /// - `keys` contains for each usecase a certificate and corresponding private key for use in RP authentication.
    /// - `sessions` will contain all sessions.
    /// - `trust_anchors` contains self-signed X509 CA certificates acting as trust anchor for the mdoc verification:
    ///   the mdoc verification function [`Document::verify()`] returns true if the mdoc verifies against one of these CAs.
    pub fn new(url: Url, keys: K, sessions: S, trust_anchors: Vec<OwnedTrustAnchor>) -> Self
    where
        S: Send + Sync + 'static,
    {
        let sessions = Arc::new(sessions);
        Self {
            url,
            keys,
            cleanup_task: sessions
                .clone()
                .start_cleanup_task(Duration::from_secs(CLEANUP_INTERVAL_SECONDS)),
            sessions,
            trust_anchors,
        }
    }

    /// Start a new disclosure session. Returns a [`ReaderEngagement`] instance that should be put in a QR
    /// or Universal Link or `mdoc://` URI.
    ///
    /// - `items_requests` contains the attributes to be requested.
    /// - `usecase_id` should point to an existing item in the `certificates` parameter.
    pub async fn new_session(
        &self,
        items_requests: ItemsRequests,
        session_type: SessionType,
        usecase_id: String,
        return_url_used: bool,
    ) -> Result<(SessionToken, ReaderEngagement)> {
        if !self.keys.contains_key(&usecase_id) {
            return Err(VerificationError::UnknownCertificate(usecase_id.clone()).into());
        }

        if items_requests.0.is_empty() {
            return Err(VerificationError::NoItemsRequests.into());
        }

        let (session_token, reader_engagement, session_state) =
            Session::<Created>::new(items_requests, session_type, usecase_id, return_url_used, &self.url)?;
        self.sessions
            .write(&session_state.state.into())
            .await
            .map_err(VerificationError::SessionStore)?;
        Ok((session_token, reader_engagement))
    }

    /// Process a disclosure protocol message of the wallet.
    ///
    /// - `msg` is the received protocol message.
    /// - `token` is the session token as parsed from the URL.
    pub async fn process_message(&self, msg: &[u8], token: SessionToken) -> Result<SessionData> {
        let state = self
            .sessions
            .get(&token)
            .await
            .map_err(VerificationError::SessionStore)?
            .ok_or_else(|| Error::from(VerificationError::UnknownSessionId(token.clone())))?;

        let (response, next) = match state.session_data {
            DisclosureData::Created(session_data) => {
                let session = Session::<Created> {
                    state: SessionState {
                        session_data,
                        token: state.token,
                        last_active: state.last_active,
                    },
                };
                let (response, session) = session
                    .process_device_engagement(cbor_deserialize(msg)?, &self.keys)
                    .await;
                match session {
                    Ok(next) => Ok((response, next.state.into())),
                    Err(next) => Ok((response, next.state.into())),
                }
            }
            DisclosureData::WaitingForResponse(session_data) => {
                let session = Session::<WaitingForResponse> {
                    state: SessionState {
                        session_data,
                        token: state.token,
                        last_active: state.last_active,
                    },
                };
                let (response, session) = session.process_response(
                    cbor_deserialize(msg)?,
                    self.trust_anchors
                        .iter()
                        .map(Into::<TrustAnchor<'_>>::into)
                        .collect::<Vec<_>>()
                        .as_slice(),
                );
                Ok((response, session.state.into()))
            }
            DisclosureData::Done(_) => Err(Error::from(VerificationError::UnexpectedInput)),
        }?;

        self.sessions
            .write(&next)
            .await
            .map_err(VerificationError::SessionStore)?;

        Ok(response)
    }

    pub async fn status(&self, session_id: &SessionToken) -> Result<StatusResponse> {
        match self
            .sessions
            .get(session_id)
            .await
            .map_err(VerificationError::SessionStore)?
            .ok_or(VerificationError::UnknownSessionId(session_id.clone()))?
            .session_data
        {
            DisclosureData::Created(_) => Ok(StatusResponse::Created),
            DisclosureData::WaitingForResponse(_) => Ok(StatusResponse::WaitingForResponse),
            DisclosureData::Done(Done {
                session_result: SessionResult::Done { .. },
            }) => Ok(StatusResponse::Done),
            DisclosureData::Done(Done {
                session_result: SessionResult::Failed { .. },
            }) => Ok(StatusResponse::Failed),
            DisclosureData::Done(Done {
                session_result: SessionResult::Cancelled { .. },
            }) => Ok(StatusResponse::Cancelled),
        }
    }

    /// Returns the disclosed attributes for a session with status `Done` and an error otherwise
    pub async fn disclosed_attributes(
        &self,
        session_id: &SessionToken,
        transcript_hash: Option<Vec<u8>>,
    ) -> Result<DisclosedAttributes> {
        match self
            .sessions
            .get(session_id)
            .await
            .map_err(VerificationError::SessionStore)?
            .ok_or(VerificationError::UnknownSessionId(session_id.clone()))?
            .session_data
        {
            DisclosureData::Created(_) => Err(VerificationError::SessionNotDone(StatusResponse::Created).into()),
            DisclosureData::WaitingForResponse(_) => {
                Err(VerificationError::SessionNotDone(StatusResponse::WaitingForResponse).into())
            }
            DisclosureData::Done(Done { session_result }) => match session_result {
                SessionResult::Failed { .. } => Err(VerificationError::SessionNotDone(StatusResponse::Failed).into()),
                SessionResult::Cancelled { .. } => {
                    Err(VerificationError::SessionNotDone(StatusResponse::Cancelled).into())
                }
                SessionResult::Done {
                    transcript_hash: None,
                    disclosed_attributes,
                } => Ok(disclosed_attributes),
                SessionResult::Done {
                    transcript_hash: Some(hash),
                    disclosed_attributes,
                } if transcript_hash.as_ref().is_some_and(|h| h == &hash) => Ok(disclosed_attributes),
                SessionResult::Done {
                    transcript_hash: Some(_),
                    ..
                } => Err(VerificationError::TranscriptHashMismatch(transcript_hash.to_owned()).into()),
            },
        }
    }
}

// Implementation of the typestate state engine follows.

// Transitioning functions and helpers valid for any state
impl<T: DisclosureState> Session<T> {
    fn transition_fail(self, error: Error) -> Session<Done> {
        self.transition(Done {
            session_result: SessionResult::Failed {
                error: error.to_string(),
            },
        })
    }

    fn transition_abort(self, status: SessionStatus) -> Session<Done> {
        self.transition(Done {
            session_result: status.into(),
        })
    }

    /// Transition `self` to a new state, consuming the old state, also updating the `last_active` timestamp.
    fn transition<NewT: DisclosureState>(self, new_state: NewT) -> Session<NewT> {
        Session {
            state: SessionState::<NewT> {
                session_data: new_state,
                token: self.state.token,
                last_active: Utc::now(),
            },
        }
    }

    fn state(&self) -> &T {
        &self.state.session_data
    }
}

impl Session<Created> {
    /// Create a new disclosure session.
    fn new(
        items_requests: ItemsRequests,
        session_type: SessionType,
        usecase_id: String,
        return_url_used: bool,
        base_url: &Url,
    ) -> Result<(SessionToken, ReaderEngagement, Session<Created>)> {
        let session_token = SessionToken::new();
        let url = base_url.join(&session_token.0).unwrap(); // token is alphanumeric so this will always succeed
        let (reader_engagement, ephemeral_privkey) = ReaderEngagement::new_reader_engagement(url)?;
        let session = Session::<Created> {
            state: SessionState::new(
                session_token.clone(),
                Created {
                    items_requests,
                    session_type,
                    usecase_id,
                    return_url_used,
                    ephemeral_privkey: ephemeral_privkey.into(),
                    reader_engagement: reader_engagement.clone(),
                },
            ),
        };

        Ok((session_token, reader_engagement, session))
    }

    /// Process the device's [`DeviceEngagement`],
    /// returning a response to answer the device with and the next session state.
    async fn process_device_engagement(
        self,
        device_engagement: DeviceEngagement,
        keys: &impl KeyRing,
    ) -> (
        SessionData,
        std::result::Result<Session<WaitingForResponse>, Session<Done>>,
    ) {
        let (response, next) = match self.process_device_engagement_inner(&device_engagement, keys).await {
            Ok((response, items_requests, their_key, ephemeral_privkey, session_transcript)) => (
                response,
                Ok(self.transition_wait_for_response(items_requests, their_key, ephemeral_privkey, session_transcript)),
            ),
            Err(e) => (SessionData::new_decoding_error(), Err(self.transition_fail(e))),
        };

        (response, next)
    }

    // Helper function that returns ordinary errors instead of `Session<...>`
    async fn process_device_engagement_inner(
        &self,
        device_engagement: &DeviceEngagement,
        keys: &impl KeyRing,
    ) -> Result<(SessionData, ItemsRequests, SessionKey, SecretKey, SessionTranscript)> {
        Self::verify_origin_infos(&device_engagement.0.origin_infos)?;

        // Compute the session transcript whose CBOR serialization acts as the challenge throughout the protocol
        let session_transcript = SessionTranscript::new(
            self.state().session_type,
            &self.state().reader_engagement,
            device_engagement,
        )
        .unwrap();

        let cert_pair = keys
            .private_key(&self.state().usecase_id)
            .ok_or_else(|| VerificationError::UnknownCertificate(self.state().usecase_id.clone()))?;

        let device_request = self.new_device_request(&session_transcript, cert_pair).await?;

        // Compute the AES keys with which we and the device encrypt responses
        let their_pubkey = device_engagement
            .0
            .security
            .as_ref()
            .ok_or(VerificationError::EphemeralKeyMissing)?
            .try_into()?;
        let our_key = SessionKey::new(
            &self.state().ephemeral_privkey.0,
            &their_pubkey,
            &session_transcript,
            SessionKeyUser::Reader,
        )?;
        let their_key = SessionKey::new(
            &self.state().ephemeral_privkey.0,
            &their_pubkey,
            &session_transcript,
            SessionKeyUser::Device,
        )?;

        let response = SessionData::serialize_and_encrypt(&device_request, &our_key)?;

        Ok((
            response,
            self.state().items_requests.clone(),
            their_key,
            self.state().ephemeral_privkey.clone().0,
            session_transcript,
        ))
    }

    fn verify_origin_infos(origin_infos: &[OriginInfo]) -> Result<()> {
        if origin_infos.len() != 2 {
            return Err(VerificationError::IncorrectOriginInfo.into());
        }

        // We ignore the referrer URL contained in OriginInfoType::Website for now, since it is not always
        // possible for the wallet to reliably determine the referrer URL, so we can't enforce it here to be equal
        // to something.
        // TODO: implement this once we have decided on a sensible thing to do with this.
        if origin_infos[0].cat != OriginInfoDirection::Received
            || !matches!(origin_infos[0].typ, OriginInfoType::Website(_))
        {
            return Err(VerificationError::IncorrectOriginInfo.into());
        }

        if origin_infos[1]
            != (OriginInfo {
                cat: OriginInfoDirection::Delivered,
                typ: OriginInfoType::MessageData,
            })
        {
            return Err(VerificationError::IncorrectOriginInfo.into());
        }

        Ok(())
    }

    fn transition_wait_for_response(
        self,
        items_requests: ItemsRequests,
        their_key: SessionKey,
        ephemeral_privkey: SecretKey,
        session_transcript: SessionTranscript,
    ) -> Session<WaitingForResponse> {
        let return_url_used = self.state.session_data.return_url_used;
        self.transition(WaitingForResponse {
            items_requests,
            their_key,
            return_url_used,
            ephemeral_privkey: ephemeral_privkey.into(),
            session_transcript,
        })
    }

    async fn new_device_request(
        &self,
        session_transcript: &SessionTranscript,
        private_key: &KeyPair,
    ) -> Result<DeviceRequest> {
        let doc_requests = try_join_all(self.state().items_requests.0.iter().map(|items_request| async {
            let reader_auth = ReaderAuthenticationKeyed {
                reader_auth_string: Default::default(),
                session_transcript: session_transcript.clone(),
                items_request_bytes: items_request.clone().into(),
            };
            let cose = MdocCose::<_, ReaderAuthenticationBytes>::sign(
                &TaggedBytes(CborSeq(reader_auth)),
                cose::new_certificate_header(private_key.certificate()),
                private_key,
                false,
            )
            .await?;
            let cose = MdocCose::from(cose.0);
            let doc_request = DocRequest {
                items_request: items_request.clone().into(),
                reader_auth: Some(cose),
            };
            Result::<DocRequest>::Ok(doc_request)
        }))
        .await?;

        Ok(DeviceRequest {
            version: DeviceRequestVersion::V1_0,
            doc_requests,
        })
    }
}

impl Session<WaitingForResponse> {
    /// Process the user's encrypted [`DeviceResponse`], i.e. its disclosure,
    /// returning a response to answer the device with and the next session state.
    fn process_response(
        self,
        session_data: SessionData,
        trust_anchors: &[TrustAnchor],
    ) -> (SessionData, Session<Done>) {
        // Abort if user wants to abort
        if let Some(status) = session_data.status {
            return (SessionData::new_termination(), self.transition_abort(status));
        };

        let (response, next) = match self.process_response_inner(&session_data, trust_anchors) {
            Ok((response, disclosed_attributes, transcript_hash)) => {
                (response, self.transition_finish(disclosed_attributes, transcript_hash))
            }
            Err(e) => (SessionData::new_decoding_error(), self.transition_fail(e)),
        };

        (response, next)
    }

    // Helper function that returns ordinary errors instead of `Session<Done>`
    fn process_response_inner(
        &self,
        session_data: &SessionData,
        trust_anchors: &[TrustAnchor],
    ) -> Result<(SessionData, DisclosedAttributes, Option<Vec<u8>>)> {
        let device_response: DeviceResponse = session_data.decrypt_and_deserialize(&self.state().their_key)?;

        let disclosed_attributes = device_response.verify(
            Some(&self.state().ephemeral_privkey.0),
            &self.state().session_transcript,
            &TimeGenerator,
            trust_anchors,
        )?;
        self.state().items_requests.match_against_response(&device_response)?;

        let response = SessionData {
            data: None,
            status: Some(SessionStatus::Termination),
        };

        let transcript_hash = self
            .state
            .session_data
            .return_url_used
            .then(|| {
                cbor_serialize(&TaggedBytes(&self.state.session_data.session_transcript)).map(|b| utils::sha256(&b))
            })
            .transpose()?;

        Ok((response, disclosed_attributes, transcript_hash))
    }

    fn transition_finish(
        self,
        disclosed_attributes: DisclosedAttributes,
        transcript_hash: Option<Vec<u8>>,
    ) -> Session<Done> {
        self.transition(Done {
            session_result: SessionResult::Done {
                disclosed_attributes,
                transcript_hash,
            },
        })
    }
}

impl ReaderEngagement {
    pub fn new_reader_engagement(session_url: Url) -> Result<(ReaderEngagement, SecretKey)> {
        let privkey = SecretKey::random(&mut OsRng);

        let engagement = Engagement {
            version: EngagementVersion::V1_0,
            security: Some((&privkey.public_key()).try_into()?),
            connection_methods: Some(vec![ConnectionMethodKeyed {
                typ: ConnectionMethodType::RestApi,
                version: ConnectionMethodVersion::RestApi,
                connection_options: RestApiOptionsKeyed {
                    uri: session_url.clone(),
                }
                .into(),
            }
            .into()]),
            origin_infos: vec![],
        };

        Ok((engagement.into(), privkey))
    }
}

impl From<SessionStatus> for SessionResult {
    fn from(status: SessionStatus) -> Self {
        match status {
            SessionStatus::EncryptionError => SessionResult::Failed {
                error: "client encryption error".to_string(),
            },
            SessionStatus::DecodingError => SessionResult::Failed {
                error: "client decoding error".to_string(),
            },
            SessionStatus::Termination => SessionResult::Cancelled,
        }
    }
}

impl ItemsRequests {
    /// Checks that all `requested` attributes are disclosed in this [`DeviceResponse`].
    pub fn match_against_response(&self, device_response: &DeviceResponse) -> Result<()> {
        let not_found: Vec<_> = self
            .0
            .iter()
            .enumerate()
            .flat_map(|(i, items_request)| {
                device_response
                    .documents
                    .as_ref()
                    .and_then(|docs| docs.get(i))
                    .map_or_else(
                        // If the entire document is missing then all requested attributes are missing
                        || items_request.attribute_identifiers().into_iter().collect(),
                        |doc| items_request.match_against_issuer_signed(doc),
                    )
            })
            .collect();

        if not_found.is_empty() {
            Ok(())
        } else {
            Err(VerificationError::MissingAttributes(not_found).into())
        }
    }
}

impl DeviceResponse {
    /// Verify a [`DeviceResponse`], returning the verified attributes, grouped per doctype and namespace.
    ///
    /// # Arguments
    /// - `eph_reader_key` - the ephemeral reader public key in case the mdoc is authentication with a MAC.
    /// - `device_authentication_bts` - the [`DeviceAuthenticationBytes`] acting as the challenge, i.e., that have
    ///   to be signed by the holder.
    /// - `time` - a generator of the current time.
    /// - `trust_anchors` - trust anchors against which verification is done.
    pub fn verify(
        &self,
        eph_reader_key: Option<&SecretKey>,
        session_transcript: &SessionTranscript,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<DisclosedAttributes> {
        if let Some(errors) = &self.document_errors {
            return Err(VerificationError::DeviceResponseErrors(errors.clone()).into());
        }
        if self.status != 0 {
            // TODO section 8.3.2.1.2.3
            return Err(VerificationError::UnexpectedStatus(self.status).into());
        }
        if self.documents.is_none() {
            return Err(VerificationError::NoDocuments.into());
        }

        let mut attrs = IndexMap::new();
        for doc in self.documents.as_ref().unwrap() {
            let (doc_type, doc_attrs) = doc.verify(eph_reader_key, session_transcript, time, trust_anchors)?;
            if doc_type != doc.doc_type {
                return Err(VerificationError::WrongDocType {
                    document: doc.doc_type.clone(),
                    mso: doc_type,
                }
                .into());
            }
            attrs.insert(doc_type, doc_attrs);
        }

        Ok(attrs)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ValidityError {
    #[error("validity parsing failed: {0}")]
    ParsingFailed(#[from] chrono::ParseError),
    #[error("not yet valid: valid from {0}")]
    NotYetValid(String),
    #[error("expired at {0}")]
    Expired(String),
}

/// Indicate how a [`ValidityInfo`] should be verified against the current date.
pub enum ValidityRequirement {
    /// The [`ValidityInfo`] must not be expired, but it is allowed to be not yet valid.
    AllowNotYetValid,
    /// The [`ValidityInfo`] must be valid now and not be expired.
    Valid,
}

impl ValidityInfo {
    pub fn verify_is_valid_at(
        &self,
        time: DateTime<Utc>,
        validity: ValidityRequirement,
    ) -> std::result::Result<(), ValidityError> {
        if matches!(validity, ValidityRequirement::Valid) && time < DateTime::<Utc>::try_from(&self.valid_from)? {
            Err(ValidityError::NotYetValid(self.valid_from.0 .0.clone()))
        } else if time > DateTime::<Utc>::try_from(&self.valid_until)? {
            Err(ValidityError::Expired(self.valid_from.0 .0.clone()))
        } else {
            Ok(())
        }
    }
}

impl IssuerSigned {
    pub fn verify(
        &self,
        validity: ValidityRequirement,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<(DocumentDisclosedAttributes, MobileSecurityObject)> {
        let TaggedBytes(mso) =
            self.issuer_auth
                .verify_against_trust_anchors(CertificateUsage::Mdl, time, trust_anchors)?;

        mso.validity_info
            .verify_is_valid_at(time.generate(), validity)
            .map_err(VerificationError::Validity)?;

        let attrs = self
            .name_spaces
            .as_ref()
            .unwrap_or(&IndexMap::new())
            .iter()
            .map(|(namespace, items)| Ok((namespace.to_string(), mso.verify_attrs_in_namespace(items, namespace)?)))
            .collect::<Result<_>>()?;

        Ok((attrs, mso))
    }
}

impl MobileSecurityObject {
    fn verify_attrs_in_namespace(&self, attrs: &Attributes, namespace: &NameSpace) -> Result<Vec<Entry>> {
        attrs
            .0
            .iter()
            .map(|item| {
                self.verify_attr_digest(namespace, item)?;
                Ok(Entry {
                    name: item.0.element_identifier.clone(),
                    value: item.0.element_value.clone(),
                })
            })
            .collect::<Result<_>>()
    }

    /// Given an `IssuerSignedItem` i.e. an attribute, verify that its digest is correctly included in the MSO.
    fn verify_attr_digest(&self, namespace: &NameSpace, item: &IssuerSignedItemBytes) -> Result<()> {
        let digest_id = item.0.digest_id;
        let digest = self
            .value_digests
            .0
            .get(namespace)
            .ok_or_else(|| VerificationError::MissingNamespace(namespace.clone()))?
            .0
            .get(&digest_id)
            .ok_or_else(|| VerificationError::MissingDigestID(digest_id))?;
        if *digest != cbor_digest(item)? {
            return Err(VerificationError::AttributeVerificationFailed.into());
        }
        Ok(())
    }
}

impl Document {
    pub fn verify(
        &self,
        eph_reader_key: Option<&SecretKey>,
        session_transcript: &SessionTranscript,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<(DocType, DocumentDisclosedAttributes)> {
        let (attrs, mso) = self
            .issuer_signed
            .verify(ValidityRequirement::Valid, time, trust_anchors)?;

        let session_transcript_bts = cbor_serialize(&TaggedBytes(session_transcript))?;
        let device_authentication =
            DeviceAuthentication::from_session_transcript(session_transcript.clone(), self.doc_type.clone());
        let device_authentication_bts = cbor_serialize(&TaggedBytes(device_authentication))?;

        let device_key = (&mso.device_key_info.device_key).try_into()?;
        match &self.device_signed.device_auth {
            DeviceAuth::DeviceSignature(sig) => {
                sig.clone_with_payload(device_authentication_bts.to_vec())
                    .verify(&device_key)?;
            }
            DeviceAuth::DeviceMac(mac) => {
                let mac_key = dh_hmac_key(
                    eph_reader_key.ok_or_else(|| VerificationError::EphemeralKeyMissing)?,
                    &device_key.into(),
                    &session_transcript_bts,
                    "EMacKey",
                    32,
                )?;
                mac.clone_with_payload(device_authentication_bts.to_vec())
                    .verify(&mac_key)?;
            }
        }

        Ok((mso.doc_type, attrs))
    }
}

impl ItemsRequest {
    /// Returns requested attributes, if any, that are not present in the `issuer_signed`.
    pub fn match_against_issuer_signed(&self, document: &Document) -> Vec<AttributeIdentifier> {
        let document_identifiers = document.issuer_signed_attribute_identifiers();
        self.attribute_identifiers()
            .into_iter()
            .filter(|attribute| !document_identifiers.contains(attribute))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use chrono::{Duration, Utc};
    use indexmap::IndexMap;
    use rstest::rstest;

    use wallet_common::trust_anchor::DerTrustAnchor;

    use crate::{
        examples::{
            Example, Examples, IsoCertTimeGenerator, EXAMPLE_ATTR_NAME, EXAMPLE_ATTR_VALUE, EXAMPLE_DOC_TYPE,
            EXAMPLE_NAMESPACE,
        },
        identifiers::AttributeIdentifierHolder,
        server_keys::{KeyPair, SingleKeyRing},
        server_state::MemorySessionStore,
        test::{self, DebugCollapseBts},
        utils::{
            crypto::{SessionKey, SessionKeyUser},
            reader_auth::ReaderRegistration,
            serialization::cbor_serialize,
        },
        verifier::{
            SessionType, ValidityError,
            ValidityRequirement::{AllowNotYetValid, Valid},
            VerificationError, Verifier,
        },
        DeviceAuthenticationBytes, DeviceEngagement, DeviceRequest, DeviceResponse, Error, ItemsRequest, SessionData,
        SessionStatus, SessionTranscript, ValidityInfo,
    };

    use super::{AttributeIdentifier, ItemsRequests};

    fn new_validity_info(add_from_days: i64, add_until_days: i64) -> ValidityInfo {
        let now = Utc::now();
        ValidityInfo {
            signed: now.into(),
            valid_from: now.add(Duration::days(add_from_days)).into(),
            valid_until: now.add(Duration::days(add_until_days)).into(),
            expected_update: None,
        }
    }

    #[test]
    fn validity_info() {
        let now = Utc::now();

        let validity = new_validity_info(-1, 1);
        validity.verify_is_valid_at(now, Valid).unwrap();
        validity.verify_is_valid_at(now, AllowNotYetValid).unwrap();

        let validity = new_validity_info(-2, -1);
        assert!(matches!(
            validity.verify_is_valid_at(now, Valid),
            Err(ValidityError::Expired(_))
        ));
        assert!(matches!(
            validity.verify_is_valid_at(now, AllowNotYetValid),
            Err(ValidityError::Expired(_))
        ));

        let validity = new_validity_info(1, 2);
        assert!(matches!(
            validity.verify_is_valid_at(now, Valid),
            Err(ValidityError::NotYetValid(_))
        ));
        validity.verify_is_valid_at(now, AllowNotYetValid).unwrap();
    }

    /// Verify the example disclosure from the standard.
    #[test]
    fn verify_iso_example_disclosure() {
        let device_response = DeviceResponse::example();
        println!("DeviceResponse: {:#?} ", DebugCollapseBts::from(&device_response));

        // Examine the first attribute in the device response
        let document = device_response.documents.as_ref().unwrap()[0].clone();
        assert_eq!(document.doc_type, EXAMPLE_DOC_TYPE);
        let namespaces = document.issuer_signed.name_spaces.as_ref().unwrap();
        let attrs = namespaces.get(EXAMPLE_NAMESPACE).unwrap();
        let issuer_signed_attr = attrs.0.first().unwrap().0.clone();
        assert_eq!(issuer_signed_attr.element_identifier, EXAMPLE_ATTR_NAME);
        assert_eq!(issuer_signed_attr.element_value, *EXAMPLE_ATTR_VALUE);
        println!("issuer_signed_attr: {:#?}", DebugCollapseBts::from(&issuer_signed_attr));

        // Do the verification
        let eph_reader_key = Examples::ephemeral_reader_key();
        let trust_anchors = Examples::iaca_trust_anchors();
        let disclosed_attrs = device_response
            .verify(
                Some(&eph_reader_key),
                &DeviceAuthenticationBytes::example().0 .0.session_transcript, // To be signed by device key found in MSO
                &IsoCertTimeGenerator,
                trust_anchors,
            )
            .unwrap();
        println!("DisclosedAttributes: {:#?}", DebugCollapseBts::from(&disclosed_attrs));

        // The first disclosed attribute is the same as we saw earlier in the DeviceResponse
        test::assert_disclosure_contains(
            &disclosed_attrs,
            EXAMPLE_DOC_TYPE,
            EXAMPLE_NAMESPACE,
            EXAMPLE_ATTR_NAME,
            &EXAMPLE_ATTR_VALUE,
        );
    }

    const DISCLOSURE_DOC_TYPE: &str = "example_doctype";
    const DISCLOSURE_NAME_SPACE: &str = "example_namespace";
    const DISCLOSURE_ATTRS: [(&str, bool); 2] = [("first_name", true), ("family_name", false)];
    const DISCLOSURE_USECASE: &str = "example_usecase";

    fn new_disclosure_request() -> ItemsRequests {
        vec![ItemsRequest {
            doc_type: DISCLOSURE_DOC_TYPE.to_string(),
            request_info: None,
            name_spaces: IndexMap::from([(
                DISCLOSURE_NAME_SPACE.to_string(),
                IndexMap::from_iter(
                    DISCLOSURE_ATTRS
                        .iter()
                        .map(|(name, intent_to_retain)| (name.to_string(), *intent_to_retain)),
                ),
            )]),
        }]
        .into()
    }

    #[tokio::test]
    async fn disclosure() {
        // Initialize server state
        let ca = KeyPair::generate_reader_mock_ca().unwrap();
        let trust_anchors = vec![
            DerTrustAnchor::from_der(ca.certificate().as_bytes().to_vec())
                .unwrap()
                .owned_trust_anchor,
        ];
        let rp_privkey = ca.generate_reader_mock(ReaderRegistration::new_mock().into()).unwrap();
        let keys = SingleKeyRing(rp_privkey);
        let session_store = MemorySessionStore::new();

        let verifier = Verifier::new(
            "https://example.com".parse().unwrap(),
            keys,
            session_store,
            trust_anchors,
        );

        // Start session
        let (session_token, reader_engagement) = verifier
            .new_session(
                new_disclosure_request(),
                SessionType::SameDevice,
                DISCLOSURE_USECASE.to_string(),
                false,
            )
            .await
            .unwrap();

        // Construct first device protocol message
        let (device_engagement, device_eph_key) =
            DeviceEngagement::new_device_engagement("https://example.com/".parse().unwrap()).unwrap();
        let msg = cbor_serialize(&device_engagement).unwrap();

        // send first device protocol message
        let encrypted_device_request = verifier.process_message(&msg, session_token.clone()).await.unwrap();

        // decrypt server response
        // Note that the unwraps here are safe, as we created the `ReaderEngagement`.
        let rp_key = SessionKey::new(
            &device_eph_key,
            &(reader_engagement.0.security.as_ref().unwrap()).try_into().unwrap(),
            &SessionTranscript::new(SessionType::SameDevice, &reader_engagement, &device_engagement).unwrap(),
            SessionKeyUser::Reader,
        )
        .unwrap();
        let _device_request: DeviceRequest = encrypted_device_request.decrypt_and_deserialize(&rp_key).unwrap();

        // We have no mdoc in this test to actually disclose, so we let the wallet terminate the session
        let end_session_message = cbor_serialize(&SessionData {
            data: None,
            status: Some(SessionStatus::Termination),
        })
        .unwrap();
        let ended_session_response = verifier
            .process_message(&end_session_message, session_token)
            .await
            .unwrap();

        assert_eq!(ended_session_response.status.unwrap(), SessionStatus::Termination);
    }

    fn example_items_requests() -> ItemsRequests {
        vec![ItemsRequest {
            doc_type: EXAMPLE_DOC_TYPE.to_string(),
            name_spaces: IndexMap::from_iter([(
                EXAMPLE_NAMESPACE.to_string(),
                IndexMap::from_iter([
                    ("family_name".to_string(), false),
                    ("issue_date".to_string(), false),
                    ("expiry_date".to_string(), false),
                    ("document_number".to_string(), false),
                    ("portrait".to_string(), false),
                    ("driving_privileges".to_string(), false),
                ]),
            )]),
            request_info: None,
        }]
        .into()
    }

    /// Helper to compute all attribute identifiers contained in a bunch of [`ItemsRequest`]s.
    fn attribute_identifiers(items_requests: &ItemsRequests) -> Vec<AttributeIdentifier> {
        items_requests
            .0
            .iter()
            .flat_map(AttributeIdentifierHolder::attribute_identifiers)
            .collect()
    }

    #[rstest]
    #[case(do_nothing())]
    #[case(swap_attributes())]
    #[case(remove_documents())]
    #[case(remove_document())]
    #[case(change_doctype())]
    #[case(remove_namespace())]
    #[case(change_namespace())]
    #[case(remove_attribute())]
    fn match_disclosed_attributes(
        #[case] testcase: (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>),
    ) {
        // Construct an items request that matches the example device response
        let (device_response, items_requests, expected_result) = testcase;
        assert_eq!(
            items_requests
                .match_against_response(&device_response)
                .map_err(|e| match e {
                    Error::Verification(VerificationError::MissingAttributes(e)) => e,
                    _ => panic!(),
                }),
            expected_result,
        );
    }

    // return an unmodified device response, which should verify
    fn do_nothing() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        (DeviceResponse::example(), example_items_requests(), Ok(()))
    }

    // Matching attributes is insensitive to swapped attributes, so verification succeeds
    fn swap_attributes() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        device_response.documents.as_mut().unwrap()[0]
            .issuer_signed
            .name_spaces
            .as_mut()
            .unwrap()
            .first_mut()
            .as_mut()
            .unwrap()
            .1
             .0
            .swap(0, 1);

        (device_response, example_items_requests(), Ok(()))
    }

    // remove all disclosed documents
    fn remove_documents() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        device_response.documents = None;

        let items_requests = example_items_requests();
        let missing = attribute_identifiers(&items_requests);
        (device_response, items_requests, Err(missing))
    }

    // remove a single disclosed document
    fn remove_document() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        device_response.documents.as_mut().unwrap().pop();

        let items_requests = example_items_requests();
        let missing = attribute_identifiers(&items_requests);
        (device_response, items_requests, Err(missing))
    }

    // Change the first doctype so it is not the requested one
    fn change_doctype() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        device_response
            .documents
            .as_mut()
            .unwrap()
            .first_mut()
            .unwrap()
            .doc_type = "some_not_requested_doc_type".to_string();

        let items_requests = example_items_requests();
        let missing = attribute_identifiers(&items_requests);
        (device_response, items_requests, Err(missing))
    }

    // Remove one of the disclosed namespaces
    fn remove_namespace() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        device_response.documents.as_mut().unwrap()[0]
            .issuer_signed
            .name_spaces
            .as_mut()
            .unwrap()
            .pop();

        let items_requests = example_items_requests();
        let missing = attribute_identifiers(&items_requests);
        (device_response, items_requests, Err(missing))
    }

    // Change a namespace so it is not the requested one
    fn change_namespace() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        let namespaces = device_response
            .documents
            .as_mut()
            .unwrap()
            .first_mut()
            .unwrap()
            .issuer_signed
            .name_spaces
            .as_mut()
            .unwrap();
        let (_, attributes) = namespaces.pop().unwrap();
        namespaces.insert("some_not_requested_name_space".to_string(), attributes);

        let items_requests = example_items_requests();
        let missing = attribute_identifiers(&items_requests);
        (device_response, items_requests, Err(missing))
    }

    // Remove one of the disclosed attributes
    fn remove_attribute() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        device_response.documents.as_mut().unwrap()[0]
            .issuer_signed
            .name_spaces
            .as_mut()
            .unwrap()
            .first_mut()
            .unwrap()
            .1
             .0
            .pop();

        let items_requests = example_items_requests();
        let missing = vec![attribute_identifiers(&items_requests).last().unwrap().clone()];
        (device_response, items_requests, Err(missing))
    }
}
