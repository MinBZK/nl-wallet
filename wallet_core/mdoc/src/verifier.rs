//! RP software, for verifying mdoc disclosures, see [`DeviceResponse::verify()`].

#![allow(unused)]

use std::any::Any;

use chrono::{DateTime, Local, Utc};
use ciborium::Value;
use coset::{CoseSign1, HeaderBuilder};
use futures::future::try_join_all;
use indexmap::IndexMap;
use p256::{
    ecdh::EphemeralSecret,
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
    PublicKey,
};
use serde::{Deserialize, Serialize};
use url::Url;
use webpki::TrustAnchor;

use wallet_common::generator::{Generator, TimeGenerator};

use crate::{
    basic_sa_ext::Entry,
    iso::*,
    issuer::SessionState,
    issuer_shared::SessionToken,
    server_state::SessionStore,
    utils::{
        cose::{self, ClonePayload, MdocCose, COSE_X5CHAIN_HEADER_LABEL},
        crypto::{cbor_digest, dh_hmac_key, SessionKey, SessionKeyUser},
        serialization::{
            cbor_deserialize, cbor_serialize, CborSeq, DeviceAuthenticationString, RequiredValue, TaggedBytes,
        },
        x509::{Certificate, CertificateUsage},
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
    #[error("DeviceAuth::DeviceMac found but no ephemeral reader key specified")]
    EphemeralKeyMissing,
    #[error("validity error: {0}")]
    Validity(#[from] ValidityError),
    #[error("missing OriginInfo in engagement: {0}")]
    MissingOriginInfo(usize),
    #[error("incorrect OriginInfo in engagement")]
    IncorrectOriginInfo,
    #[error("unexpected input: session is done")]
    UnexpectedInput,
}

struct Session<S> {
    state: SessionState<DisclosureData<S>>,
}

#[derive(Debug, Clone)]
pub struct DisclosureData<S> {
    disclosure_state_enum: DisclosureStateEnum,
    disclosure_state: S,
}

/// An enum whose variants correspond 1-to-1 with all possible states for a session, i.e., all implementations
/// of the [`DisclosureState`] trait.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum DisclosureStateEnum {
    Created,
    WaitingForResponse,
    Done,
}

/// State for a session that has just been created.
struct Created {
    items_requests: Vec<ItemsRequest>,
    reader_cert: Certificate,
    reader_cert_privkey: SigningKey,
    ephemeral_privkey: EphemeralSecret,
    reader_engagement: ReaderEngagement,
}

/// State for a session that is waiting for the user's disclosure, i.e., the device has read our
/// [`ReaderEngagement`] and contacted us at the session URL.
struct WaitingForResponse {
    items_requests: Vec<ItemsRequest>,
    our_key: SessionKey,
    their_key: SessionKey,
    session_transcript: SessionTranscript,
}

/// State for a session that has finished (perhaps due to cancellation or errors).
struct Done {
    session_result: SessionResult,
}

enum SessionResult {
    Done { disclosed_attributes: DisclosedAttributes },
    Failed, // TODO add error details
    Cancelled,
}

/// Disclosure session states for use as `T` in `Session<T>`.
pub trait DisclosureState {
    fn state_enum() -> DisclosureStateEnum
    where
        Self: Sized;
}

impl DisclosureState for Created {
    fn state_enum() -> DisclosureStateEnum {
        DisclosureStateEnum::Created
    }
}
impl DisclosureState for WaitingForResponse {
    fn state_enum() -> DisclosureStateEnum {
        DisclosureStateEnum::WaitingForResponse
    }
}
impl DisclosureState for Done {
    fn state_enum() -> DisclosureStateEnum {
        DisclosureStateEnum::Done
    }
}

pub fn new_session(
    base_url: Url,
    items_requests: Vec<ItemsRequest>,
    reader_cert: &Certificate,
    reader_cert_privkey: &SigningKey,
    sessions: impl SessionStore<Data = SessionState<DisclosureData<Box<dyn DisclosureState>>>>,
) -> Result<ReaderEngagement> {
    let token = SessionToken::new();
    let url = base_url.join(&token.0).unwrap();
    let (reader_engagement, session_state) =
        Session::<Created>::new(items_requests, reader_cert, reader_cert_privkey, url)?;
    sessions.write(&session_state.state.into_boxed());
    Ok(reader_engagement)
}

pub async fn process_message(
    msg: &[u8],
    token: SessionToken,
    sessions: &impl SessionStore<Data = SessionState<DisclosureData<Box<dyn DisclosureState>>>>,
    trust_anchors: &[TrustAnchor<'_>],
) -> Result<SessionData> {
    let state = sessions.get(&token)?;

    let session = Session::<Box<dyn Any>> {
        state: SessionState {
            session_data: DisclosureData {
                disclosure_state_enum: state.session_data.disclosure_state_enum,
                disclosure_state: Box::new(state.session_data.disclosure_state),
            },
            token: state.token,
            last_active: state.last_active,
        },
    };

    let (response, next) = match state.session_data.disclosure_state_enum {
        DisclosureStateEnum::Created => {
            let session = Session::<Created> {
                state: session.state.into_unboxed(),
            };
            let (response, session) = session.process_device_engagement(cbor_deserialize(msg)?).await;
            match session {
                Ok(next) => Ok((response, next.state.into_boxed())),
                Err(next) => Ok((response, next.state.into_boxed())),
            }
        }
        DisclosureStateEnum::WaitingForResponse => {
            let session = Session::<WaitingForResponse> {
                state: session.state.into_unboxed(),
            };
            let (response, session) = session.process_response(cbor_deserialize(msg)?, trust_anchors);
            Ok((response, session.state.into_boxed()))
        }
        DisclosureStateEnum::Done => Err(Error::from(VerificationError::UnexpectedInput)),
    }?;

    sessions.write(&next);

    Ok(response)
}

// Transitioning functions and helpers valid for any state
impl<T: DisclosureState> Session<T> {
    fn fail(self) -> Session<Done> {
        self.transition(Done {
            session_result: SessionResult::Failed,
        })
    }

    fn abort(self, status: SessionStatus) -> Session<Done> {
        self.transition(Done {
            session_result: status.into(),
        })
    }

    /// Converts `self` to a fresh copy with an updated timestamp and the specified state.
    fn transition<NewT: DisclosureState>(self, new_state: NewT) -> Session<NewT> {
        Session {
            state: SessionState::<DisclosureData<NewT>> {
                session_data: DisclosureData {
                    disclosure_state: new_state,
                    disclosure_state_enum: NewT::state_enum(),
                },
                token: self.state.token,
                last_active: Local::now(),
            },
        }
    }

    fn state(&self) -> &T {
        &self.state.session_data.disclosure_state
    }
}

impl Session<Created> {
    // TODO: update this API when it has been decided on,
    // and implement RP authentication in this function instead of leaving it up to the caller.
    /// Create a new disclosure session.
    fn new(
        items_requests: Vec<ItemsRequest>,
        reader_cert: &Certificate,
        reader_cert_privkey: &SigningKey,
        url: Url,
    ) -> Result<(ReaderEngagement, Session<Created>)> {
        let (reader_engagement, ephemeral_privkey) = ReaderEngagement::new_reader_engagement(url)?;
        let session = Session::<Created> {
            state: SessionState::new(
                SessionToken::new(),
                DisclosureData {
                    disclosure_state_enum: DisclosureStateEnum::Created,
                    disclosure_state: Created {
                        items_requests,
                        reader_cert: reader_cert.clone(),
                        reader_cert_privkey: reader_cert_privkey.clone(),
                        ephemeral_privkey,
                        reader_engagement: reader_engagement.clone(),
                    },
                },
            ),
        };

        Ok((reader_engagement, session))
    }

    /// Process the device's [`DeviceEngagement`],
    /// returning a response to answer the device with and the next session state.
    async fn process_device_engagement(
        self,
        device_engagement: DeviceEngagement,
    ) -> (
        SessionData,
        std::result::Result<Session<WaitingForResponse>, Session<Done>>,
    ) {
        let (response, next) = match self.process_device_engagement_inner(&device_engagement).await {
            Ok((response, items_requests, our_key, their_key, session_transcript)) => (
                response,
                Ok(self.wait_for_response(items_requests, our_key, their_key, session_transcript)),
            ),
            Err(_) => (SessionData::new_decoding_error(), Err(self.fail())),
        };

        (response, next)
    }

    // Helper function that returns ordinary errors instead of `Session<...>`
    async fn process_device_engagement_inner(
        &self,
        device_engagement: &DeviceEngagement,
    ) -> Result<(
        SessionData,
        Vec<ItemsRequest>,
        SessionKey,
        SessionKey,
        SessionTranscript,
    )> {
        // Check that the device has sent the expected OriginInfo
        let url = self.state().reader_engagement.0.connection_methods.as_ref().unwrap()[0]
            .0
            .connection_options
            .0
            .uri
            .clone();

        if device_engagement.0.origin_infos
            != vec![
                OriginInfo {
                    cat: OriginInfoDirection::Received,
                    typ: OriginInfoType::Website(url),
                },
                OriginInfo {
                    cat: OriginInfoDirection::Delivered,
                    typ: OriginInfoType::MessageData,
                },
            ]
        {
            return Err(VerificationError::IncorrectOriginInfo.into());
        }

        // Compute the session transcript whose CBOR serialization acts as the challenge throughout the protocol
        let session_transcript = SessionTranscriptKeyed {
            device_engagement_bytes: device_engagement.clone().into(),
            handover: Handover::SchemeHandoverBytes(TaggedBytes(self.state().reader_engagement.clone())),
            ereader_key_bytes: self
                .state()
                .reader_engagement
                .0
                .security
                .as_ref()
                .unwrap()
                .0
                .e_sender_key_bytes
                .clone(),
        }
        .into();

        let state = self.state();
        let device_request = self.new_device_request(&session_transcript).await?;

        // Compute the AES keys with which we and the device encrypt responses
        // TODO remove unwrap() and return an error if the device passes no key
        let their_pubkey = device_engagement.0.security.as_ref().unwrap().try_into()?;
        let our_key = SessionKey::new(
            &self.state().ephemeral_privkey,
            &their_pubkey,
            &session_transcript,
            SessionKeyUser::Reader,
        )?;
        let their_key = SessionKey::new(
            &self.state().ephemeral_privkey,
            &their_pubkey,
            &session_transcript,
            SessionKeyUser::Device,
        )?;

        let response = SessionData::serialize_and_encrypt(&device_request, &our_key)?;

        Ok((
            response,
            self.state.session_data.disclosure_state.items_requests.clone(),
            our_key,
            their_key,
            session_transcript,
        ))
    }

    fn wait_for_response(
        self,
        items_requests: Vec<ItemsRequest>,
        our_key: SessionKey,
        their_key: SessionKey,
        session_transcript: SessionTranscript,
    ) -> Session<WaitingForResponse> {
        self.transition(WaitingForResponse {
            items_requests,
            our_key,
            their_key,
            session_transcript,
        })
    }

    async fn new_device_request(&self, session_transcript: &SessionTranscript) -> Result<DeviceRequest> {
        let doc_requests = try_join_all(self.state().items_requests.iter().map(|items_request| async {
            let reader_auth = ReaderAuthenticationKeyed {
                reader_auth_string: Default::default(),
                session_transcript: session_transcript.clone(),
                items_request_bytes: items_request.clone().into(),
            };
            let cose = MdocCose::<_, ReaderAuthenticationBytes>::sign(
                &TaggedBytes(CborSeq(reader_auth)),
                cose::new_certificate_header(&self.state().reader_cert),
                &self.state().reader_cert_privkey,
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
            return (SessionData::new_termination(), self.abort(status));
        };

        let (response, next) = match self.process_response_inner(&session_data, trust_anchors) {
            Ok((response, disclosed_attributes)) => (response, self.finish(disclosed_attributes)),
            Err(_) => (SessionData::new_decoding_error(), self.fail()),
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
            None, // TODO without this, documents using MAC can't be verified
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

    fn finish(self, disclosed_attributes: DisclosedAttributes) -> Session<Done> {
        self.transition(Done {
            session_result: SessionResult::Done { disclosed_attributes },
        })
    }
}

impl ReaderEngagement {
    fn new_reader_engagement(referrer_url: Url) -> Result<(ReaderEngagement, EphemeralSecret)> {
        let privkey = EphemeralSecret::random(&mut OsRng);

        let engagement = Engagement {
            version: EngagementVersion::V1_0,
            security: Some((&privkey.public_key()).try_into()?),
            connection_methods: Some(vec![ConnectionMethodKeyed {
                typ: ConnectionMethodType::RestApi,
                version: ConnectionMethodVersion::RestApi,
                connection_options: RestApiOptionsKeyed {
                    uri: referrer_url.clone(),
                }
                .into(),
            }
            .into()]),
            origin_infos: vec![OriginInfo {
                cat: OriginInfoDirection::Delivered,
                typ: OriginInfoType::Website(referrer_url),
            }],
        };

        Ok((engagement.into(), privkey))
    }
}

impl SessionState<DisclosureData<Box<dyn Any>>> {
    /// Unpack the boxed state. NOTE: will panic if the box does not contain the right type.
    fn into_unboxed<NewT: Sized + 'static>(self) -> SessionState<DisclosureData<NewT>> {
        SessionState {
            session_data: DisclosureData {
                disclosure_state: *self.session_data.disclosure_state.downcast().unwrap(),
                disclosure_state_enum: self.session_data.disclosure_state_enum,
            },
            token: self.token,
            last_active: Local::now(),
        }
    }
}

impl<T: DisclosureState + 'static> SessionState<DisclosureData<T>> {
    fn into_boxed(self) -> SessionState<DisclosureData<Box<(dyn DisclosureState + 'static)>>> {
        SessionState {
            session_data: DisclosureData {
                disclosure_state: Box::new(self.session_data.disclosure_state),
                disclosure_state_enum: self.session_data.disclosure_state_enum,
            },
            token: self.token,
            last_active: Local::now(),
        }
    }
}

impl From<SessionStatus> for SessionResult {
    fn from(status: SessionStatus) -> Self {
        match status {
            SessionStatus::EncryptionError => SessionResult::Failed,
            SessionStatus::DecodingError => SessionResult::Failed,
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
        eph_reader_key: Option<&EphemeralSecret>,
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
        eph_reader_key: Option<&EphemeralSecret>,
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

    use crate::{
        verifier::{
            ValidityError,
            ValidityRequirement::{AllowNotYetValid, Valid},
        },
        ValidityInfo,
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
}
