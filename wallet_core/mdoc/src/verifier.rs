//! RP software, for verifying mdoc disclosures, see [`DeviceResponse::verify()`].

use std::{sync::Arc, time::Duration};

use base64::prelude::*;
use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use indexmap::IndexMap;
use nutype::nutype;
use p256::SecretKey;
use rand_core::OsRng;
use ring::hmac;
use serde::{Deserialize, Serialize};
use serde_with::{hex::Hex, serde_as};
use strfmt::strfmt;
use strum;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};
use url::Url;
use webpki::TrustAnchor;

use wallet_common::{
    account::serialization::DerSecretKey,
    config::wallet_config::BaseUrl,
    generator::{Generator, TimeGenerator},
    trust_anchor::OwnedTrustAnchor,
    utils,
};

use crate::{
    identifiers::{AttributeIdentifier, AttributeIdentifierHolder},
    iso::*,
    server_keys::{KeyPair, KeyRing},
    server_state::{
        Expirable, HasProgress, Progress, SessionState, SessionStore, SessionStoreError, SessionToken,
        CLEANUP_INTERVAL_SECONDS,
    },
    unsigned::Entry,
    utils::{
        cose::{self, ClonePayload, MdocCose},
        crypto::{cbor_digest, dh_hmac_key, SessionKey, SessionKeyUser},
        reader_auth::ReturnUrlPrefix,
        serialization::{cbor_deserialize, cbor_hex, cbor_serialize, CborSeq, TaggedBytes},
        x509::CertificateUsage,
    },
    Error, Result, SessionData,
};

/// Attributes of an mdoc that was disclosed in a [`DeviceResponse`], as computed by [`DeviceResponse::verify()`].
/// Grouped per namespace. Validity information and the attributes issuer's common_name is also included.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDisclosedAttributes {
    pub attributes: IndexMap<NameSpace, Vec<Entry>>,
    pub issuer: Vec<String>,
    pub validity_info: ValidityInfo,
}
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
    #[error("missing verifier URL params")]
    MissingVerifierUrlParameters,
    #[error("session is done")]
    SessionIsDone,
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
    #[error("disclosed attributes requested for disclosure session with status other than 'Done'")]
    SessionNotDone,
    #[error("transcript hash '{0:?}' does not match expected")]
    TranscriptHashMismatch(Vec<u8>),
    #[error("the ephemeral ID {} is invalid", hex::encode(.0))]
    InvalidEphemeralId(Vec<u8>),
    #[error("the ephemeral ID {} has expired", hex::encode(.0))]
    ExpiredEphemeralId(Vec<u8>),
    #[error("URL encoding error: {0}")]
    UrlEncoding(#[from] serde_urlencoded::ser::Error),
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
    return_url: Option<Url>,
    ephemeral_privkey: DerSecretKey,
}

/// State for a session that is waiting for the user's disclosure, i.e., the device has read our
/// [`ReaderEngagement`] and contacted us at the session URL.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WaitingForResponse {
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
    Expired,
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

impl HasProgress for DisclosureData {
    fn progress(&self) -> Progress {
        match self {
            Self::Created(_) | Self::WaitingForResponse(_) => Progress::Active,
            Self::Done(done) => Progress::Finished {
                has_succeeded: matches!(done.session_result, SessionResult::Done { .. }),
            },
        }
    }
}

impl Expirable for DisclosureData {
    fn is_expired(&self) -> bool {
        matches!(
            self,
            Self::Done(Done {
                session_result: SessionResult::Expired
            })
        )
    }

    fn expire(&mut self) {
        *self = Self::Done(Done {
            session_result: SessionResult::Expired,
        })
    }
}

impl From<SessionState<Created>> for SessionState<DisclosureData> {
    fn from(value: SessionState<Created>) -> Self {
        SessionState {
            data: DisclosureData::Created(value.data),
            token: value.token,
            last_active: value.last_active,
        }
    }
}

impl From<SessionState<WaitingForResponse>> for SessionState<DisclosureData> {
    fn from(value: SessionState<WaitingForResponse>) -> Self {
        SessionState {
            data: DisclosureData::WaitingForResponse(value.data),
            token: value.token,
            last_active: value.last_active,
        }
    }
}

impl From<SessionState<Done>> for SessionState<DisclosureData> {
    fn from(value: SessionState<Done>) -> Self {
        SessionState {
            data: DisclosureData::Done(value.data),
            token: value.token,
            last_active: value.last_active,
        }
    }
}

/// status without the underlying data
#[derive(Debug, Deserialize, Serialize, strum::Display)]
#[serde(rename_all = "UPPERCASE", tag = "status")]
pub enum StatusResponse {
    Created { engagement_url: Url },
    WaitingForResponse,
    Done,
    Failed,
    Cancelled,
    Expired,
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

const EPHEMERAL_ID_VALIDITY_SECONDS: Duration = Duration::from_secs(10);

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierUrlParameters {
    #[serde_as(as = "Hex")]
    ephemeral_id: Vec<u8>,
    // default (de)serialization of DateTime is the RFC 3339 format
    time: DateTime<Utc>,
}

#[nutype(
    derive(Debug, Clone, Serialize, Deserialize, FromStr),
    validate(predicate = ReturnUrlTemplate::is_valid_return_url_template),
)]
pub struct ReturnUrlTemplate(String);

impl ReturnUrlTemplate {
    pub fn into_url(self, session_token: &SessionToken) -> Url {
        strfmt!(&self.into_inner(), session_token => session_token.to_string())
            .expect("valid ReturnUrlTemplate should always format")
            .parse()
            .expect("formatted ReturnUrlTemplate should always be a valid URL")
    }

    fn is_valid_return_url_template(s: &str) -> bool {
        // it should be a valid ReturnUrlPrefix when removing the template parameter
        let s = s.replace("{session_token}", "");
        let url = s.parse::<Url>(); // this makes sure no Url-invalid characters are present
        url.is_ok_and(|mut u| {
            u.set_query(None); // query is allowed in a template but not in a prefix
            u.set_fragment(None); // fragment is allowed in a template but not in a prefix
            u = u
                .join("path_segment_that_ends_with_a_slash/")
                .expect("should always result in a valid URL"); // path not ending with a '/' is allowed in a template but not in prefix
            TryInto::<ReturnUrlPrefix>::try_into(u).is_ok()
        })
    }
}

pub struct Verifier<K, S> {
    keys: K,
    sessions: Arc<S>,
    cleanup_task: JoinHandle<()>,
    trust_anchors: Vec<OwnedTrustAnchor>,
    ephemeral_id_secret: hmac::Key,
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
    S: SessionStore<DisclosureData>,
{
    /// Create a new [`Verifier`].
    ///
    /// - `keys` contains for each usecase a certificate and corresponding private key for use in RP authentication.
    /// - `sessions` will contain all sessions.
    /// - `trust_anchors` contains self-signed X509 CA certificates acting as trust anchor for the mdoc verification:
    ///   the mdoc verification function [`Document::verify()`] returns true if the mdoc verifies against one of these CAs.
    pub fn new(keys: K, sessions: S, trust_anchors: Vec<OwnedTrustAnchor>, ephemeral_id_secret: hmac::Key) -> Self
    where
        S: Send + Sync + 'static,
    {
        let sessions = Arc::new(sessions);
        Self {
            keys,
            cleanup_task: sessions.clone().start_cleanup_task(CLEANUP_INTERVAL_SECONDS),
            sessions,
            trust_anchors,
            ephemeral_id_secret,
        }
    }

    /// Start a new disclosure session. Returns a [`SessionToken`] that can be used to retrieve the
    /// session state.
    ///
    /// - `items_requests` contains the attributes to be requested.
    /// - `usecase_id` should point to an existing item in the `certificates` parameter.
    pub async fn new_session(
        &self,
        items_requests: ItemsRequests,
        session_type: SessionType,
        usecase_id: String,
        return_url_template: Option<ReturnUrlTemplate>,
    ) -> Result<SessionToken> {
        info!("create verifier session: {usecase_id}");

        if !self.keys.contains_key(&usecase_id) {
            return Err(VerificationError::UnknownCertificate(usecase_id.clone()).into());
        }

        if items_requests.0.is_empty() {
            return Err(VerificationError::NoItemsRequests.into());
        }

        let (session_token, session_state) =
            Session::<Created>::new(items_requests, session_type, usecase_id, return_url_template)?;
        self.sessions
            .write(session_state.state.into(), true)
            .await
            .map_err(VerificationError::SessionStore)?;
        info!("Session({session_token}): session created");
        Ok(session_token)
    }

    /// Process a disclosure protocol message of the wallet.
    ///
    /// - `msg` is the received protocol message.
    /// - `token` is the session token as parsed from the URL.
    /// - `verifier_url` is a tuple of the base URL of the disclosure endpoint of the wallet server and the query parameters passed to the verifier URL.
    pub async fn process_message(
        &self,
        msg: &[u8],
        session_token: &SessionToken,
        verifier_url: Option<(BaseUrl, VerifierUrlParameters)>,
    ) -> Result<SessionData> {
        let state = self
            .sessions
            .get(session_token)
            .await
            .map_err(VerificationError::SessionStore)?
            .ok_or_else(|| VerificationError::UnknownSessionId(session_token.clone()))?;

        info!("Session({session_token}): process message");

        let (response, next) = match (state.data, verifier_url) {
            (
                DisclosureData::Created(session_data),
                Some((verifier_base_url, VerifierUrlParameters { time, ephemeral_id })),
            ) => {
                if Utc::now() - EPHEMERAL_ID_VALIDITY_SECONDS > time {
                    return Err(VerificationError::ExpiredEphemeralId(ephemeral_id).into());
                }

                hmac::verify(
                    &self.ephemeral_id_secret,
                    &Self::format_ephemeral_id_payload(session_token, &time),
                    &ephemeral_id,
                )
                .map_err(|_| VerificationError::InvalidEphemeralId(ephemeral_id.clone()))?;

                let session = Session::<Created> {
                    state: SessionState {
                        data: session_data,
                        token: state.token,
                        last_active: state.last_active,
                    },
                };

                // to be able to compute the SessionTranscript, the verifier URL needs to be reconstructed here
                let verifier_url = Self::format_verifier_url(&verifier_base_url, session_token, time, ephemeral_id)?;
                let (response, session) = session
                    .process_device_engagement(&cbor_deserialize(msg)?, verifier_url, &self.keys)
                    .await;
                match session {
                    Ok(next) => Ok((response, next.state.into())),
                    Err(next) => Ok((response, next.state.into())),
                }
            }
            (DisclosureData::WaitingForResponse(session_data), _) => {
                let session = Session::<WaitingForResponse> {
                    state: SessionState {
                        data: session_data,
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
            (DisclosureData::Created(_), None) => Err(VerificationError::MissingVerifierUrlParameters),
            (DisclosureData::Done(_), _) => Err(VerificationError::SessionIsDone),
        }?;

        self.sessions
            .write(next, false)
            .await
            .map_err(VerificationError::SessionStore)?;

        Ok(response)
    }

    fn create_reader_engagement(
        &self,
        ephemeral_privkey: &SecretKey,
        verifier_base_url: &BaseUrl,
        session_token: &SessionToken,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<ReaderEngagement> {
        let time = time.generate();
        let ephemeral_id = self.generate_ephemeral_id(session_token, &time);

        let verifier_url = Self::format_verifier_url(verifier_base_url, session_token, time, ephemeral_id)?;
        ReaderEngagement::try_new(ephemeral_privkey, verifier_url)
    }

    pub async fn status_response(
        &self,
        session_token: &SessionToken,
        engagement_base_url: &BaseUrl,
        verifier_base_url: &BaseUrl,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<StatusResponse> {
        let response = match self
            .sessions
            .get(session_token)
            .await
            .map_err(VerificationError::SessionStore)?
            .ok_or_else(|| VerificationError::UnknownSessionId(session_token.clone()))?
            .data
        {
            DisclosureData::Created(Created {
                session_type,
                return_url,
                ephemeral_privkey,
                ..
            }) => {
                let reader_engagement =
                    self.create_reader_engagement(&ephemeral_privkey.0, verifier_base_url, session_token, time)?;
                let engagement_url =
                    Self::format_engagement_url(engagement_base_url, &reader_engagement, session_type, return_url);

                StatusResponse::Created { engagement_url }
            }
            DisclosureData::WaitingForResponse(_) => StatusResponse::WaitingForResponse,
            DisclosureData::Done(Done {
                session_result: SessionResult::Done { .. },
            }) => StatusResponse::Done,
            DisclosureData::Done(Done {
                session_result: SessionResult::Failed { .. },
            }) => StatusResponse::Failed,
            DisclosureData::Done(Done {
                session_result: SessionResult::Cancelled { .. },
            }) => StatusResponse::Cancelled,
            DisclosureData::Done(Done {
                session_result: SessionResult::Expired { .. },
            }) => StatusResponse::Expired,
        };

        Ok(response)
    }

    /// Returns the disclosed attributes for a session with status `Done` and an error otherwise
    pub async fn disclosed_attributes(
        &self,
        session_token: &SessionToken,
        transcript_hash: Option<Vec<u8>>,
    ) -> Result<DisclosedAttributes> {
        match self
            .sessions
            .get(session_token)
            .await
            .map_err(VerificationError::SessionStore)?
            .ok_or_else(|| VerificationError::UnknownSessionId(session_token.clone()))?
            .data
        {
            DisclosureData::Done(Done {
                session_result:
                    SessionResult::Done {
                        transcript_hash: None,
                        disclosed_attributes,
                    },
            }) => Ok(disclosed_attributes),
            DisclosureData::Done(Done {
                session_result:
                    SessionResult::Done {
                        transcript_hash: Some(hash),
                        disclosed_attributes,
                    },
            }) if transcript_hash.as_ref().is_some_and(|h| h == &hash) => Ok(disclosed_attributes),
            DisclosureData::Done(Done {
                session_result:
                    SessionResult::Done {
                        transcript_hash: Some(hash),
                        ..
                    },
            }) => Err(VerificationError::TranscriptHashMismatch(hash).into()),
            _ => Err(VerificationError::SessionNotDone.into()),
        }
    }
}

impl<K, S> Verifier<K, S> {
    fn generate_ephemeral_id(&self, session_token: &SessionToken, time: &DateTime<Utc>) -> Vec<u8> {
        let ephemeral_id = hmac::sign(
            &self.ephemeral_id_secret,
            &Self::format_ephemeral_id_payload(session_token, time),
        )
        .as_ref()
        .to_vec();
        ephemeral_id
    }

    fn format_verifier_url(
        verifier_base_url: &BaseUrl,
        session_token: &SessionToken,
        time: DateTime<Utc>,
        ephemeral_id: Vec<u8>,
    ) -> Result<Url, VerificationError> {
        let mut verifier_url = verifier_base_url.join(session_token.as_ref());
        verifier_url.set_query(Some(
            serde_urlencoded::to_string(VerifierUrlParameters { time, ephemeral_id })?.as_str(),
        ));
        Ok(verifier_url)
    }

    fn format_engagement_url(
        base_url: &BaseUrl,
        reader_engagement: &ReaderEngagement,
        session_type: SessionType,
        return_url: Option<Url>,
    ) -> Url {
        let mut engagement_url = base_url.join(
            &BASE64_URL_SAFE_NO_PAD
                .encode(cbor_serialize(reader_engagement).expect("serializing an engagement should never fail")),
        );

        engagement_url
            .query_pairs_mut()
            .append_pair("session_type", &session_type.to_string());
        if let Some(return_url) = return_url {
            engagement_url
                .query_pairs_mut()
                .append_pair("return_url", return_url.as_ref());
        }
        engagement_url
    }

    // formats the payload to hash to the ephemeral ID in a consistent way
    fn format_ephemeral_id_payload(session_token: &SessionToken, time: &DateTime<Utc>) -> Vec<u8> {
        // default (de)serialization of DateTime is the RFC 3339 format
        format!("{}|{}", session_token, time.to_rfc3339()).into()
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
            state: SessionState::new(self.state.token, new_state),
        }
    }

    fn state(&self) -> &T {
        &self.state.data
    }
}

impl Session<Created> {
    /// Create a new disclosure session.
    fn new(
        items_requests: ItemsRequests,
        session_type: SessionType,
        usecase_id: String,
        return_url_template: Option<ReturnUrlTemplate>,
    ) -> Result<(SessionToken, Session<Created>)> {
        let session_token = SessionToken::new_random();
        let ephemeral_privkey = SecretKey::random(&mut OsRng);
        let session = Session::<Created> {
            state: SessionState::new(
                session_token.clone(),
                Created {
                    items_requests,
                    session_type,
                    usecase_id,
                    return_url: return_url_template.map(|u| u.into_url(&session_token)),
                    ephemeral_privkey: ephemeral_privkey.into(),
                },
            ),
        };

        Ok((session_token, session))
    }

    /// Process the device's [`DeviceEngagement`],
    /// returning a response to answer the device with and the next session state.
    async fn process_device_engagement(
        self,
        device_engagement: &DeviceEngagement,
        verifier_url: Url,
        keys: &impl KeyRing,
    ) -> (
        SessionData,
        std::result::Result<Session<WaitingForResponse>, Session<Done>>,
    ) {
        info!("Session({}): process device engagement", self.state.token);
        let (response, next) = match self
            .process_device_engagement_inner(device_engagement, verifier_url, keys)
            .await
        {
            Ok((response, items_requests, their_key, ephemeral_privkey, session_transcript)) => (
                response,
                Ok(self.transition_wait_for_response(items_requests, their_key, ephemeral_privkey, session_transcript)),
            ),
            Err(e) => {
                warn!(
                    "Session({}): process device engagement failed, returning decoding error",
                    self.state.token
                );
                (SessionData::new_decoding_error(), Err(self.transition_fail(e)))
            }
        };

        (response, next)
    }

    // Helper function that returns ordinary errors instead of `Session<...>`
    async fn process_device_engagement_inner(
        &self,
        device_engagement: &DeviceEngagement,
        verifier_url: Url,
        keys: &impl KeyRing,
    ) -> Result<(SessionData, ItemsRequests, SessionKey, SecretKey, SessionTranscript)> {
        Self::verify_origin_infos(&device_engagement.0.origin_infos)?;

        let reader_engagement = ReaderEngagement::try_new(&self.state().ephemeral_privkey.0, verifier_url)?;

        // Compute the session transcript whose CBOR serialization acts as the challenge throughout the protocol
        let session_transcript =
            SessionTranscript::new(self.state().session_type, &reader_engagement, device_engagement).unwrap();

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
        let return_url_used = self.state.data.return_url.is_some();
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
            let items_request = items_request.clone().into();
            let reader_auth = ReaderAuthenticationKeyed::new(session_transcript, &items_request);
            let cose = MdocCose::<_, ReaderAuthenticationBytes>::sign(
                &TaggedBytes(CborSeq(reader_auth)),
                cose::new_certificate_header(private_key.certificate()),
                private_key,
                false,
            )
            .await?;
            let cose = MdocCose::from(cose.0);
            let doc_request = DocRequest {
                items_request,
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
        info!("Session({}): process response", self.state.token);

        // Abort if user wants to abort
        if let Some(status) = session_data.status {
            return (SessionData::new_termination(), self.transition_abort(status));
        };

        let (response, next) = match self.process_response_inner(&session_data, trust_anchors) {
            Ok((response, disclosed_attributes, transcript_hash)) => {
                (response, self.transition_finish(disclosed_attributes, transcript_hash))
            }
            Err(e) => {
                warn!(
                    "Session({}): process response failed, returning decoding error",
                    self.state.token
                );
                (SessionData::new_decoding_error(), self.transition_fail(e))
            }
        };

        (response, next)
    }

    // Helper function that returns ordinary errors instead of `Session<Done>`
    fn process_response_inner(
        &self,
        session_data: &SessionData,
        trust_anchors: &[TrustAnchor],
    ) -> Result<(SessionData, DisclosedAttributes, Option<Vec<u8>>)> {
        debug!(
            "Session({}): decrypting and deserializing device response",
            self.state.token
        );

        let device_response: DeviceResponse = session_data.decrypt_and_deserialize(&self.state().their_key)?;

        debug!("Session({}): verify device response", self.state.token);

        let disclosed_attributes = device_response.verify(
            Some(&self.state().ephemeral_privkey.0),
            &self.state().session_transcript,
            &TimeGenerator,
            trust_anchors,
        )?;

        debug!(
            "Session({}): check whether all requested attributes are received",
            self.state.token
        );

        self.state().items_requests.match_against_response(&device_response)?;

        let response = SessionData {
            data: None,
            status: Some(SessionStatus::Termination),
        };

        let transcript_hash = self
            .state
            .data
            .return_url_used
            .then(|| cbor_serialize(&TaggedBytes(&self.state.data.session_transcript)).map(|b| utils::sha256(&b)))
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
    pub fn try_new(privkey: &SecretKey, verifier_url: Url) -> Result<Self> {
        let engagement = Engagement {
            version: EngagementVersion::V1_0,
            security: Some((&privkey.public_key()).try_into()?),
            connection_methods: Some(vec![ConnectionMethodKeyed {
                typ: ConnectionMethodType::RestApi,
                version: ConnectionMethodVersion::RestApi,
                connection_options: RestApiOptionsKeyed { uri: verifier_url }.into(),
            }
            .into()]),
            origin_infos: vec![],
        };

        Ok(engagement.into())
    }

    #[cfg(test)]
    pub fn new_random(verifier_url: Url) -> Result<(Self, SecretKey)> {
        let privkey = SecretKey::random(&mut OsRng);
        Ok((Self::try_new(&privkey, verifier_url)?, privkey))
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
            .flat_map(|items_request| {
                device_response
                    .documents
                    .as_ref()
                    .and_then(|docs| docs.iter().find(|doc| doc.doc_type == items_request.doc_type))
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
            return Err(VerificationError::UnexpectedStatus(self.status).into());
        }

        if self.documents.is_none() {
            return Err(VerificationError::NoDocuments.into());
        }

        let mut attrs = IndexMap::new();
        for doc in self.documents.as_ref().unwrap() {
            debug!("verifying document with doc_type: {}", doc.doc_type);
            let (doc_type, doc_attrs) = doc
                .verify(eph_reader_key, session_transcript, time, trust_anchors)
                .map_err(|e| {
                    warn!("document verification failed: {e}");
                    e
                })?;
            attrs.insert(doc_type, doc_attrs);
            debug!("document OK");
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
            .map(|name_spaces| {
                name_spaces
                    .as_ref()
                    .iter()
                    .map(|(namespace, items)| Ok((namespace.clone(), mso.verify_attrs_in_namespace(items, namespace)?)))
                    .collect::<Result<_>>()
            })
            .transpose()?
            .unwrap_or_default();

        Ok((
            DocumentDisclosedAttributes {
                attributes: attrs,
                issuer: self.issuer_auth.signing_cert()?.iter_common_name()?,
                validity_info: mso.validity_info.clone(),
            },
            mso,
        ))
    }
}

impl MobileSecurityObject {
    fn verify_attrs_in_namespace(&self, attrs: &Attributes, namespace: &NameSpace) -> Result<Vec<Entry>> {
        attrs
            .as_ref()
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
        debug!("verifying document with doc_type: {:?}", &self.doc_type);
        debug!("verify issuer_signed");
        let (attrs, mso) = self
            .issuer_signed
            .verify(ValidityRequirement::Valid, time, trust_anchors)?;

        debug!("verifying mso.doc_type matches document doc_type");
        if self.doc_type != mso.doc_type {
            return Err(VerificationError::WrongDocType {
                document: self.doc_type.clone(),
                mso: mso.doc_type,
            }
            .into());
        }

        debug!("serializing session transcript");
        let session_transcript_bts = cbor_serialize(&TaggedBytes(session_transcript))?;
        let device_authentication = DeviceAuthenticationKeyed::new(&self.doc_type, session_transcript);
        debug!("serializing device_authentication");
        let device_authentication_bts = cbor_serialize(&TaggedBytes(CborSeq(device_authentication)))?;

        debug!("extracting device_key");
        let device_key = (&mso.device_key_info.device_key).try_into()?;
        match &self.device_signed.device_auth {
            DeviceAuth::DeviceSignature(sig) => {
                debug!("verifying DeviceSignature");
                sig.clone_with_payload(device_authentication_bts.to_vec())
                    .verify(&device_key)?;
            }
            DeviceAuth::DeviceMac(mac) => {
                debug!("verifying DeviceMac");
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
        debug!("signature valid");

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

    use base64::prelude::*;
    use chrono::{DateTime, Duration, Utc};
    use indexmap::IndexMap;
    use p256::SecretKey;
    use ring::{hmac, rand};
    use rstest::rstest;
    use url::Url;

    use wallet_common::{
        config::wallet_config::BaseUrl,
        generator::{Generator, TimeGenerator},
        trust_anchor::DerTrustAnchor,
    };

    use crate::{
        examples::{
            Example, Examples, IsoCertTimeGenerator, EXAMPLE_ATTR_NAME, EXAMPLE_ATTR_VALUE, EXAMPLE_DOC_TYPE,
            EXAMPLE_NAMESPACE,
        },
        identifiers::AttributeIdentifierHolder,
        server_keys::{KeyPair, SingleKeyRing},
        server_state::{MemorySessionStore, SessionToken},
        test::{self, DebugCollapseBts},
        utils::{
            crypto::{SessionKey, SessionKeyUser},
            reader_auth::ReaderRegistration,
            serialization::{cbor_deserialize, cbor_serialize},
        },
        DeviceAuthenticationBytes, DeviceEngagement, DeviceRequest, DeviceResponse, Document, Error, ItemsRequest,
        ReaderEngagement, SessionData, SessionStatus, SessionTranscript, ValidityInfo,
    };

    use super::*;

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
        validity.verify_is_valid_at(now, ValidityRequirement::Valid).unwrap();
        validity
            .verify_is_valid_at(now, ValidityRequirement::AllowNotYetValid)
            .unwrap();

        let validity = new_validity_info(-2, -1);
        assert!(matches!(
            validity.verify_is_valid_at(now, ValidityRequirement::Valid),
            Err(ValidityError::Expired(_))
        ));
        assert!(matches!(
            validity.verify_is_valid_at(now, ValidityRequirement::AllowNotYetValid),
            Err(ValidityError::Expired(_))
        ));

        let validity = new_validity_info(1, 2);
        assert!(matches!(
            validity.verify_is_valid_at(now, ValidityRequirement::Valid),
            Err(ValidityError::NotYetValid(_))
        ));
        validity
            .verify_is_valid_at(now, ValidityRequirement::AllowNotYetValid)
            .unwrap();
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
        let attrs = namespaces.as_ref().get(EXAMPLE_NAMESPACE).unwrap();
        let issuer_signed_attr = attrs.as_ref().first().unwrap().0.clone();
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

    async fn init_and_start_disclosure(
        time: &impl Generator<DateTime<Utc>>,
    ) -> (
        Verifier<SingleKeyRing, MemorySessionStore<DisclosureData>>,
        ReaderEngagement,
        DeviceEngagement,
        SecretKey,
        SessionToken,
        BaseUrl,
        VerifierUrlParameters,
    ) {
        // Initialize server state
        let ca = KeyPair::generate_reader_mock_ca().unwrap();
        let trust_anchors = vec![
            DerTrustAnchor::from_der(ca.certificate().as_bytes().to_vec())
                .unwrap()
                .owned_trust_anchor,
        ];
        let rp_privkey = ca.generate_reader_mock(ReaderRegistration::new_mock().into()).unwrap();
        let keys = SingleKeyRing(rp_privkey);
        let session_store = MemorySessionStore::default();

        let verifier = Verifier::new(
            keys,
            session_store,
            trust_anchors,
            hmac::Key::generate(hmac::HMAC_SHA256, &rand::SystemRandom::new()).unwrap(),
        );

        // Start session
        let session_token = verifier
            .new_session(
                new_disclosure_request(),
                SessionType::SameDevice,
                DISCLOSURE_USECASE.to_string(),
                None,
            )
            .await
            .unwrap();

        let verifier_url = format!("https://example.com/disclosure/{session_token}")
            .parse()
            .unwrap();
        let response = verifier
            .status_response(
                &session_token,
                &"https://app.example.com/app".parse().unwrap(),
                &verifier_url,
                time,
            )
            .await
            .expect("should result in status response for session");

        let engagement_url = match response {
            StatusResponse::Created { engagement_url } => engagement_url,
            _ => panic!("should match DisclosureData::Created"),
        };

        let reader_engagement: ReaderEngagement = cbor_deserialize(
            BASE64_URL_SAFE_NO_PAD
                .decode(engagement_url.path_segments().unwrap().last().unwrap())
                .expect("serializing an engagement should never fail")
                .as_slice(),
        )
        .expect("should always deserialize");

        let verifier_url_params: VerifierUrlParameters =
            serde_urlencoded::from_str(reader_engagement.verifier_url().unwrap().query().unwrap()).unwrap();

        // Construct first device protocol message
        let (device_engagement, device_eph_key) =
            DeviceEngagement::new_device_engagement("https://example.com/".parse().unwrap()).unwrap();

        (
            verifier,
            reader_engagement,
            device_engagement,
            device_eph_key,
            session_token,
            verifier_url,
            verifier_url_params,
        )
    }

    #[tokio::test]
    async fn disclosure() {
        let (
            verifier,
            reader_engagement,
            device_engagement,
            device_eph_key,
            session_token,
            verifier_url,
            verifier_url_params,
        ) = init_and_start_disclosure(&TimeGenerator).await;

        let msg = cbor_serialize(&device_engagement).unwrap();

        // send first device protocol message
        let encrypted_device_request = verifier
            .process_message(&msg, &session_token.clone(), Some((verifier_url, verifier_url_params)))
            .await
            .unwrap();

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
            .process_message(&end_session_message, &session_token, None) // VerifierUrlParameters are only verified for DisclosureData::Created
            .await
            .unwrap();

        assert_eq!(ended_session_response.status.unwrap(), SessionStatus::Termination);
    }

    struct ExpiredEphemeralIdGenerator;

    impl Generator<DateTime<Utc>> for ExpiredEphemeralIdGenerator {
        fn generate(&self) -> DateTime<Utc> {
            Utc::now() - EPHEMERAL_ID_VALIDITY_SECONDS - Duration::seconds(1)
        }
    }

    #[tokio::test]
    async fn disclosure_expired_id() {
        let (verifier, _, device_engagement, _, session_token, verifier_url, verifier_url_params) =
            init_and_start_disclosure(&ExpiredEphemeralIdGenerator).await;

        let msg = cbor_serialize(&device_engagement).unwrap();

        let session_response = verifier
            .process_message(
                &msg,
                &session_token.clone(),
                Some((verifier_url, verifier_url_params.clone())),
            )
            .await
            .expect_err("should result in VerificationError::ExpiredEphemeralId");

        assert!(matches!(
            session_response,
            Error::Verification(VerificationError::ExpiredEphemeralId(ephemeral_id)) if ephemeral_id == verifier_url_params.ephemeral_id
        ));
    }

    #[tokio::test]
    async fn disclosure_invalid_id() {
        let (verifier, _, device_engagement, _, session_token, verifier_url, mut verifier_url_params) =
            init_and_start_disclosure(&TimeGenerator).await;

        let msg = cbor_serialize(&device_engagement).unwrap();

        let invalid_ephemeral_id = b"\xde\xad\xbe\xef".to_vec();

        // set an invalid ephemeral id
        verifier_url_params.ephemeral_id.clone_from(&invalid_ephemeral_id);

        let session_response = verifier
            .process_message(&msg, &session_token.clone(), Some((verifier_url, verifier_url_params)))
            .await
            .expect_err("should result in VerificationError::InvalidEphemeralId(...)");

        assert_eq!(
            session_response.to_string(),
            "verification error: the ephemeral ID deadbeef is invalid"
        );
        assert!(matches!(
            session_response,
            Error::Verification(VerificationError::InvalidEphemeralId(ephemeral_id)) if ephemeral_id == invalid_ephemeral_id
        ));
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
    #[case(change_namespace())]
    #[case(remove_attribute())]
    #[case(multiple_doc_types_swapped())]
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
        let first_document = device_response.documents.as_mut().unwrap().first_mut().unwrap();
        let name_spaces = first_document.issuer_signed.name_spaces.as_mut().unwrap();

        name_spaces.modify_first_attributes(|attributes| {
            attributes.swap(0, 1);
        });

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

    // Change a namespace so it is not the requested one
    fn change_namespace() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        let first_document = device_response.documents.as_mut().unwrap().first_mut().unwrap();
        let name_spaces = first_document.issuer_signed.name_spaces.as_mut().unwrap();

        name_spaces.modify_namespaces(|name_spaces| {
            let (_, attributes) = name_spaces.pop().unwrap();
            name_spaces.insert("some_not_requested_name_space".to_string(), attributes);
        });

        let items_requests = example_items_requests();
        let missing = attribute_identifiers(&items_requests);
        (device_response, items_requests, Err(missing))
    }

    // Remove one of the disclosed attributes
    fn remove_attribute() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        let first_document = device_response.documents.as_mut().unwrap().first_mut().unwrap();
        let name_spaces = first_document.issuer_signed.name_spaces.as_mut().unwrap();

        name_spaces.modify_first_attributes(|attributes| {
            attributes.pop();
        });

        let items_requests = example_items_requests();
        let missing = vec![attribute_identifiers(&items_requests).last().unwrap().clone()];
        (device_response, items_requests, Err(missing))
    }

    // Add one extra document with doc_type "a", and swap the order in the items_requests
    fn multiple_doc_types_swapped() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        let mut cloned_doc: Document = device_response.documents.as_ref().unwrap()[0].clone();
        cloned_doc.doc_type = "a".to_string();
        device_response.documents.as_mut().unwrap().push(cloned_doc);

        let mut items_requests = example_items_requests();
        let mut cloned_items_request = items_requests.0[0].clone();
        cloned_items_request.doc_type = "a".to_string();
        items_requests.0.push(cloned_items_request);

        // swap the document order in items_requests
        items_requests.0.reverse();

        (device_response, items_requests, Ok(()))
    }

    #[rstest]
    #[case("https://example.com/{session_token}", true)]
    #[case("https://example.com/return/{session_token}", true)]
    #[case("https://example.com/return/{session_token}/url", true)]
    #[case("https://example.com/{session_token}/", true)]
    #[case("https://example.com/return/{session_token}/", true)]
    #[case("https://example.com/return/{session_token}/url/", true)]
    #[case("https://example.com/return/{session_token}?hello=world&bye=mars#hashtag", true)]
    #[case("https://example.com/{session_token}/{session_token}", true)]
    #[case("https://example.com/", true)]
    #[case("https://example.com/return", true)]
    #[case("https://example.com/return/url", true)]
    #[case("https://example.com/return/", true)]
    #[case("https://example.com/return/url/", true)]
    #[case("https://example.com/return/?hello=world&bye=mars#hashtag", true)]
    #[case("https://example.com/{session_token}/{not_session_token}", true)]
    #[case("file://etc/passwd", false)]
    #[case("file://etc/{session_token}", false)]
    #[case("https://{session_token}", false)]
    fn test_return_url_template(#[case] return_url_string: String, #[case] should_parse: bool) {
        assert_eq!(return_url_string.parse::<ReturnUrlTemplate>().is_ok(), should_parse);
        assert_eq!(
            ReturnUrlTemplate::is_valid_return_url_template(&return_url_string),
            should_parse
        )
    }

    #[rstest]
    #[case(
        "https://example.com",
        SessionType::CrossDevice,
        None,
        "https://example.com/owBjMS4wAYIB2BhYS6QBAiABIVggQSzRKlJog6Smkn-OOtSkZi7umI1jmPiIn9-SJ43QJREiWCBl10JVf2y_W7g_R5UgqwCWjON7sQcpHXOFrmixq77auAKBgwQBgXRodHRwczovL2V4YW1wbGUuY29tLw?session_type=cross_device"
    )]
    #[case(
        "https://example.com",
        SessionType::SameDevice,
        Some("https://example.com/return/deadbeef".parse().unwrap()),
        "https://example.com/owBjMS4wAYIB2BhYS6QBAiABIVggQSzRKlJog6Smkn-OOtSkZi7umI1jmPiIn9-SJ43QJREiWCBl10JVf2y_W7g_R5UgqwCWjON7sQcpHXOFrmixq77auAKBgwQBgXRodHRwczovL2V4YW1wbGUuY29tLw?session_type=same_device&return_url=https%3A%2F%2Fexample.com%2Freturn%2Fdeadbeef"
    )]
    #[case(
        "https://example.com",
        SessionType::CrossDevice,
        Some("https://example.com/return/deadbeef".parse().unwrap()),
        "https://example.com/owBjMS4wAYIB2BhYS6QBAiABIVggQSzRKlJog6Smkn-OOtSkZi7umI1jmPiIn9-SJ43QJREiWCBl10JVf2y_W7g_R5UgqwCWjON7sQcpHXOFrmixq77auAKBgwQBgXRodHRwczovL2V4YW1wbGUuY29tLw?session_type=cross_device&return_url=https%3A%2F%2Fexample.com%2Freturn%2Fdeadbeef"
    )]
    #[case(
        "https://example.com",
        SessionType::SameDevice,
        None,
        "https://example.com/owBjMS4wAYIB2BhYS6QBAiABIVggQSzRKlJog6Smkn-OOtSkZi7umI1jmPiIn9-SJ43QJREiWCBl10JVf2y_W7g_R5UgqwCWjON7sQcpHXOFrmixq77auAKBgwQBgXRodHRwczovL2V4YW1wbGUuY29tLw?session_type=same_device"
    )]
    #[case(
        "https://example.com",
        SessionType::SameDevice,
        Some("https://example.com/return/deadbeef?hello=world#hashtag".parse().unwrap()),
        "https://example.com/owBjMS4wAYIB2BhYS6QBAiABIVggQSzRKlJog6Smkn-OOtSkZi7umI1jmPiIn9-SJ43QJREiWCBl10JVf2y_W7g_R5UgqwCWjON7sQcpHXOFrmixq77auAKBgwQBgXRodHRwczovL2V4YW1wbGUuY29tLw?session_type=same_device&return_url=https%3A%2F%2Fexample.com%2Freturn%2Fdeadbeef%3Fhello%3Dworld%23hashtag"
    )]
    #[case(
        "https://example.com",
        SessionType::SameDevice,
        Some("https://example.com/deadbeef?id=deadbeef#deadbeef".parse().unwrap()),
        "https://example.com/owBjMS4wAYIB2BhYS6QBAiABIVggQSzRKlJog6Smkn-OOtSkZi7umI1jmPiIn9-SJ43QJREiWCBl10JVf2y_W7g_R5UgqwCWjON7sQcpHXOFrmixq77auAKBgwQBgXRodHRwczovL2V4YW1wbGUuY29tLw?session_type=same_device&return_url=https%3A%2F%2Fexample.com%2Fdeadbeef%3Fid%3Ddeadbeef%23deadbeef"
    )]
    fn test_format_engagement_url(
        #[case] base_url: BaseUrl,
        #[case] session_type: SessionType,
        #[case] return_url: Option<Url>,
        #[case] expected: Url,
    ) {
        // generated with: `BASE64_URL_SAFE_NO_PAD.encode(cbor_serialize(&ReaderEngagement::new_reader_engagement("https://example.com".parse().unwrap()).unwrap().0).unwrap()`
        let reader_engagement: ReaderEngagement =
            cbor_deserialize(BASE64_URL_SAFE_NO_PAD.decode("owBjMS4wAYIB2BhYS6QBAiABIVggQSzRKlJog6Smkn-OOtSkZi7umI1jmPiIn9-SJ43QJREiWCBl10JVf2y_W7g_R5UgqwCWjON7sQcpHXOFrmixq77auAKBgwQBgXRodHRwczovL2V4YW1wbGUuY29tLw").unwrap().as_slice()).unwrap();
        let result = Verifier::<(), ()>::format_engagement_url(&base_url, &reader_engagement, session_type, return_url);
        assert_eq!(result, expected);
    }
}
