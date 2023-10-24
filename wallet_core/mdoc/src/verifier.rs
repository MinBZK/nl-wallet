//! RP software, for verifying mdoc disclosures, see [`DeviceResponse::verify()`].

use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use indexmap::IndexMap;
use p256::{elliptic_curve::rand_core::OsRng, SecretKey};
use serde::{Deserialize, Serialize};
use url::Url;
use webpki::TrustAnchor;

use wallet_common::{
    account::serialization::DerSecretKey,
    generator::{Generator, TimeGenerator},
};

use crate::{
    basic_sa_ext::Entry,
    iso::*,
    server_keys::{KeyRing, PrivateKey},
    server_state::{SessionState, SessionStore, SessionToken},
    utils::{
        cose::{self, ClonePayload, MdocCose},
        crypto::{cbor_digest, dh_hmac_key, SessionKey, SessionKeyUser},
        serialization::{cbor_deserialize, cbor_serialize, CborSeq, TaggedBytes},
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
    items_requests: Vec<ItemsRequest>,
    usecase_id: String,
    ephemeral_privkey: DerSecretKey,
    reader_engagement: ReaderEngagement,
}

/// State for a session that is waiting for the user's disclosure, i.e., the device has read our
/// [`ReaderEngagement`] and contacted us at the session URL.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WaitingForResponse {
    #[allow(unused)] // TODO write function that matches this field against the disclosed attributes
    items_requests: Vec<ItemsRequest>,
    their_key: SessionKey,
    ephemeral_privkey: DerSecretKey,
    session_transcript: SessionTranscript,
}

/// State for a session that has ended (for any reason).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Done {
    pub session_result: SessionResult,
}

/// The outcome of a session: the disclosed attributes if they have been sucessfully received and verified.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SessionResult {
    Done { disclosed_attributes: DisclosedAttributes },
    Failed { error: String },
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

impl SessionState<Created> {
    fn into_enum(self) -> SessionState<DisclosureData> {
        SessionState {
            session_data: DisclosureData::Created(self.session_data),
            token: self.token,
            last_active: self.last_active,
        }
    }
}

impl SessionState<WaitingForResponse> {
    fn into_enum(self) -> SessionState<DisclosureData> {
        SessionState {
            session_data: DisclosureData::WaitingForResponse(self.session_data),
            token: self.token,
            last_active: self.last_active,
        }
    }
}

impl SessionState<Done> {
    fn into_enum(self) -> SessionState<DisclosureData> {
        SessionState {
            session_data: DisclosureData::Done(self.session_data),
            token: self.token,
            last_active: self.last_active,
        }
    }
}

/// Start a new disclosure session. Returns a [`ReaderEngagement`] instance that should be put in a QR
/// or Universal Link or `mdoc://` URI.
///
/// - `base_url` is the URL at which the server is publically reachable; this is included in the [`ReaderEngagement`]
///   returned to the wallet.
/// - `items_requests` contains the attributes to be requested.
/// - `usecase_id` should point to an existing item in the `certificates` parameter.
/// - `keys` contains for each usecase a certificate and corresponding private key for use in RP authentication.
/// - `sessions` contains all currently active sessions, managed by this function and by [`process_message()`].
pub fn new_session(
    base_url: &Url,
    items_requests: Vec<ItemsRequest>,
    usecase_id: String,
    keys: &impl KeyRing,
    sessions: &impl SessionStore<Data = SessionState<DisclosureData>>,
) -> Result<(SessionToken, ReaderEngagement)> {
    if !keys.contains_key(&usecase_id) {
        return Err(VerificationError::UnknownCertificate(usecase_id.clone()).into());
    }

    if items_requests.is_empty() {
        return Err(VerificationError::NoItemsRequests.into());
    }

    let (session_token, reader_engagement, session_state) =
        Session::<Created>::new(items_requests, usecase_id, base_url)?;
    sessions.write(&session_state.state.into_enum());
    Ok((session_token, reader_engagement))
}

/// Process a disclosure protocol message of the wallet.
///
/// - `msg` is the received protocol message.
/// - `token` is the session token as parsed from the URL.
/// - `keys` and `sessions` are as in [`new_session()`].
/// - `trust_anchors` contains self-signed X509 CA certificates acting as trust anchor for the mdoc verification:
///   the mdoc verification function [`Document::verify()`] returns true if the mdoc verifies against one of these CAs.
pub async fn process_message(
    msg: &[u8],
    token: SessionToken,
    keys: &impl KeyRing,
    sessions: &impl SessionStore<Data = SessionState<DisclosureData>>,
    trust_anchors: &[TrustAnchor<'_>],
) -> Result<SessionData> {
    let state = sessions
        .get(&token)
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
            let (response, session) = session.process_device_engagement(cbor_deserialize(msg)?, keys).await;
            match session {
                Ok(next) => Ok((response, next.state.into_enum())),
                Err(next) => Ok((response, next.state.into_enum())),
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
            let (response, session) = session.process_response(cbor_deserialize(msg)?, trust_anchors);
            Ok((response, session.state.into_enum()))
        }
        DisclosureData::Done(_) => Err(Error::from(VerificationError::UnexpectedInput)),
    }?;

    sessions.write(&next);

    Ok(response)
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
        items_requests: Vec<ItemsRequest>,
        usecase_id: String,
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
                    usecase_id,
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
    ) -> Result<(SessionData, Vec<ItemsRequest>, SessionKey, SecretKey, SessionTranscript)> {
        Self::verify_origin_infos(&device_engagement.0.origin_infos)?;

        // Compute the session transcript whose CBOR serialization acts as the challenge throughout the protocol
        let session_transcript = SessionTranscript::new(&self.state().reader_engagement, device_engagement);

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
        items_requests: Vec<ItemsRequest>,
        their_key: SessionKey,
        ephemeral_privkey: SecretKey,
        session_transcript: SessionTranscript,
    ) -> Session<WaitingForResponse> {
        self.transition(WaitingForResponse {
            items_requests,
            their_key,
            ephemeral_privkey: ephemeral_privkey.into(),
            session_transcript,
        })
    }

    async fn new_device_request(
        &self,
        session_transcript: &SessionTranscript,
        private_key: &PrivateKey,
    ) -> Result<DeviceRequest> {
        let doc_requests = try_join_all(self.state().items_requests.iter().map(|items_request| async {
            let reader_auth = ReaderAuthenticationKeyed {
                reader_auth_string: Default::default(),
                session_transcript: session_transcript.clone(),
                items_request_bytes: items_request.clone().into(),
            };
            let cose = MdocCose::<_, ReaderAuthenticationBytes>::sign(
                &TaggedBytes(CborSeq(reader_auth)),
                cose::new_certificate_header(&private_key.cert_bts),
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
            Ok((response, disclosed_attributes)) => (response, self.transition_finish(disclosed_attributes)),
            Err(e) => (SessionData::new_decoding_error(), self.transition_fail(e)),
        };

        (response, next)
    }

    // Helper function that returns ordinary errors instead of `Session<Done>`
    fn process_response_inner(
        &self,
        session_data: &SessionData,
        trust_anchors: &[TrustAnchor],
    ) -> Result<(SessionData, DisclosedAttributes)> {
        let device_response: DeviceResponse = session_data.decrypt_and_deserialize(&self.state().their_key)?;

        let disclosed_attributes = device_response.verify(
            Some(&self.state().ephemeral_privkey.0),
            &self.state().session_transcript,
            &TimeGenerator,
            trust_anchors,
        )?;
        let response = SessionData {
            data: None,
            status: Some(SessionStatus::Termination),
        };

        Ok((response, disclosed_attributes))
    }

    fn transition_finish(self, disclosed_attributes: DisclosedAttributes) -> Session<Done> {
        self.transition(Done {
            session_result: SessionResult::Done { disclosed_attributes },
        })
    }
}

impl ReaderEngagement {
    fn new_reader_engagement(session_url: Url) -> Result<(ReaderEngagement, SecretKey)> {
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

pub type X509Subject = IndexMap<String, String>;

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
        let device_authentication = DeviceAuthenticationKeyed {
            device_authentication: Default::default(),
            session_transcript: session_transcript.clone(),
            doc_type: self.doc_type.clone(),
            device_name_spaces_bytes: self.device_signed.name_spaces.clone(),
        };
        let device_authentication_bts = cbor_serialize(&TaggedBytes(CborSeq(device_authentication)))?;

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

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use chrono::{Duration, Utc};
    use indexmap::IndexMap;

    use crate::{
        server_keys::{PrivateKey, SingleKeyRing},
        server_state::MemorySessionStore,
        utils::{
            crypto::{SessionKey, SessionKeyUser},
            serialization::cbor_serialize,
            x509::{Certificate, CertificateType},
        },
        verifier::{
            new_session, process_message, ValidityError,
            ValidityRequirement::{AllowNotYetValid, Valid},
        },
        DeviceEngagement, DeviceRequest, ItemsRequest, SessionData, SessionStatus, SessionTranscript, ValidityInfo,
    };

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

    const RP_CA_CN: &str = "ca.rp.example.com";
    const RP_CERT_CN: &str = "cert.rp.example.com";
    const DISCLOSURE_DOC_TYPE: &str = "example_doctype";
    const DISCLOSURE_NAME_SPACE: &str = "example_namespace";
    const DISCLOSURE_ATTRS: [(&str, bool); 2] = [("first_name", true), ("family_name", false)];
    const DISCLOSURE_USECASE: &str = "example_usecase";

    fn new_disclosure_request() -> Vec<ItemsRequest> {
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
    }

    #[tokio::test]
    async fn disclosure() {
        // Initialize server state
        let (ca, ca_privkey) = Certificate::new_ca(RP_CA_CN).unwrap();
        let trust_anchors = &[(&ca).try_into().unwrap()];
        let (rp_cert, rp_privkey) = Certificate::new(
            &ca,
            &ca_privkey,
            RP_CERT_CN,
            CertificateType::ReaderAuth(Default::default()),
        )
        .unwrap();
        let keys = SingleKeyRing(PrivateKey::new(rp_privkey, rp_cert));
        let session_store = MemorySessionStore::new();

        // Start session
        let url = "https://example.com".parse().unwrap();
        let (session_token, reader_engagement) = new_session(
            &url,
            new_disclosure_request(),
            DISCLOSURE_USECASE.to_string(),
            &keys,
            &session_store,
        )
        .unwrap();

        // Construct first device protocol message
        let (device_engagement, device_eph_key) =
            DeviceEngagement::new_device_engagement("https://example.com/".parse().unwrap()).unwrap();
        let msg = cbor_serialize(&device_engagement).unwrap();

        // send first device protocol message
        let encrypted_device_request =
            process_message(&msg, session_token.clone(), &keys, &session_store, trust_anchors)
                .await
                .unwrap();

        // decrypt server response
        let rp_key = SessionKey::new(
            &device_eph_key,
            &(reader_engagement.0.security.as_ref().unwrap()).try_into().unwrap(),
            &SessionTranscript::new(&reader_engagement, &device_engagement),
            SessionKeyUser::Reader,
        )
        .unwrap();
        let device_request: DeviceRequest = encrypted_device_request.decrypt_and_deserialize(&rp_key).unwrap();
        dbg!(device_request);

        // We have no mdoc in this test to actually disclose, so we let the wallet terminate the session
        let end_session_message = cbor_serialize(&SessionData {
            data: None,
            status: Some(SessionStatus::Termination),
        })
        .unwrap();
        let ended_session_response = process_message(
            &end_session_message,
            session_token,
            &keys,
            &session_store,
            trust_anchors,
        )
        .await
        .unwrap();

        assert_eq!(ended_session_response.status.unwrap(), SessionStatus::Termination);
    }
}
