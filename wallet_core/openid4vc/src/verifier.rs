//! RP software, for verifying mdoc disclosures, see [`DeviceResponse::verify()`].

use std::sync::Arc;

use base64::prelude::*;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use josekit::jwk::alg::ec::{EcCurve, EcKeyPair};
use ring::hmac;
use serde::{Deserialize, Serialize};
use serde_with::{hex::Hex, serde_as};
use strum;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};
use url::Url;

use nl_wallet_mdoc::{
    holder::TrustAnchor,
    identifiers::AttributeIdentifier,
    iso::*,
    server_keys::KeyRing,
    server_state::{
        Expirable, HasProgress, Progress, SessionState, SessionStore, SessionStoreError, SessionToken,
        CLEANUP_INTERVAL_SECONDS,
    },
    verifier::{
        DisclosedAttributes, ItemsRequests, ReturnUrlTemplate, SessionType, ValidityError,
        EPHEMERAL_ID_VALIDITY_SECONDS,
    },
};
use wallet_common::{
    config::wallet_config::BaseUrl, generator::Generator, jwt::Jwt, trust_anchor::OwnedTrustAnchor,
    utils::random_string,
};

use crate::{
    jwt,
    openid4vp::{VpAuthorizationRequest, VpAuthorizationResponse, VpRequestUriObject, VpResponse},
};

// TODO check these
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
    #[error("redirect URI '{0}' does not match expected")]
    RedirectUriMismatch(Url),
    #[error("the ephemeral ID {} is invalid", hex::encode(.0))]
    InvalidEphemeralId(Vec<u8>),
    #[error("the ephemeral ID {} has expired", hex::encode(.0))]
    ExpiredEphemeralId(Vec<u8>),
    #[error("URL encoding error: {0}")]
    UrlEncoding(#[from] serde_urlencoded::ser::Error),

    #[error("TODO")]
    UnexpectedState,
}

// TODO
#[derive(thiserror::Error, Debug)]
pub enum AuthorizationRequestError {}

#[derive(thiserror::Error, Debug)]
pub enum AuthorizationResponseError {}

/// A disclosure session. `S` must implement [`DisclosureState`] and is the state that the session is in.
/// The session progresses through the possible states using a state engine that uses the typestate pattern:
/// for each state `S`, `Session<S>` has its own state transition method that consume the previous state.
#[derive(Debug)]
struct Session<S: DisclosureState> {
    state: SessionState<S>,
}

/// State for a session that has just been created.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Created {
    items_requests: ItemsRequests,
    usecase_id: String,
    client_id: String,
    return_url_template: Option<ReturnUrlTemplate>,
}

/// State for a session that is waiting for the user's disclosure, i.e., the device has contacted us at the session URL.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WaitingForResponse {
    auth_request: VpAuthorizationRequest,
    encryption_key: EncryptionPrivateKey,
    redirect_uri: Option<Url>,
}

#[derive(Debug, Clone)]
struct EncryptionPrivateKey(EcKeyPair);

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
        redirect_uri: Option<Url>,
    },
    Failed {
        error: String,
    },
    Cancelled,
    Expired,
}

impl Serialize for EncryptionPrivateKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        BASE64_URL_SAFE_NO_PAD
            .encode(self.0.to_der_private_key())
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EncryptionPrivateKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(EncryptionPrivateKey(
            EcKeyPair::from_der(
                BASE64_URL_SAFE_NO_PAD
                    .decode(String::deserialize(deserializer)?)
                    .unwrap(),
                None,
            )
            .unwrap(),
        ))
    }
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

// From/TryFrom converters for the various state structs to the `DisclosureData` enum

impl From<Session<Created>> for SessionState<DisclosureData> {
    fn from(value: Session<Created>) -> Self {
        SessionState {
            data: DisclosureData::Created(value.state.data),
            token: value.state.token,
            last_active: value.state.last_active,
        }
    }
}

impl TryFrom<SessionState<DisclosureData>> for Session<Created> {
    type Error = VerificationError;

    fn try_from(value: SessionState<DisclosureData>) -> Result<Self, Self::Error> {
        let DisclosureData::Created(session_data) = value.data else {
            return Err(VerificationError::UnexpectedState);
        };
        Ok(Session::<Created> {
            state: SessionState {
                data: session_data,
                token: value.token,
                last_active: value.last_active,
            },
        })
    }
}

impl From<Session<WaitingForResponse>> for SessionState<DisclosureData> {
    fn from(value: Session<WaitingForResponse>) -> Self {
        SessionState {
            data: DisclosureData::WaitingForResponse(value.state.data),
            token: value.state.token,
            last_active: value.state.last_active,
        }
    }
}

impl TryFrom<SessionState<DisclosureData>> for Session<WaitingForResponse> {
    type Error = VerificationError;

    fn try_from(value: SessionState<DisclosureData>) -> Result<Self, Self::Error> {
        let DisclosureData::WaitingForResponse(session_data) = value.data else {
            return Err(VerificationError::UnexpectedState);
        };
        Ok(Session::<WaitingForResponse> {
            state: SessionState {
                data: session_data,
                token: value.token,
                last_active: value.last_active,
            },
        })
    }
}

impl From<Session<Done>> for SessionState<DisclosureData> {
    fn from(value: Session<Done>) -> Self {
        SessionState {
            data: DisclosureData::Done(value.state.data),
            token: value.state.token,
            last_active: value.state.last_active,
        }
    }
}

/// status without the underlying data
#[derive(Debug, Deserialize, Serialize, strum::Display)]
#[serde(rename_all = "UPPERCASE", tag = "status")]
pub enum StatusResponse {
    Created { ul: BaseUrl },
    WaitingForResponse,
    Done,
    Failed,
    Cancelled,
    Expired,
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
    /// - `ephemeral_id_secret` is used as a HMAC secret to create ephemeral session IDs.
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
    /// - `return_url_template` is the return URL the user should be returned to, if present.
    pub async fn new_session(
        &self,
        items_requests: ItemsRequests,
        usecase_id: String,
        return_url_template: Option<ReturnUrlTemplate>,
    ) -> Result<SessionToken, VerificationError> {
        info!("create verifier session: {usecase_id}");

        if items_requests.0.is_empty() {
            return Err(VerificationError::NoItemsRequests);
        }

        let Some(private_key) = self.keys.private_key(&usecase_id) else {
            return Err(VerificationError::UnknownCertificate(usecase_id.clone()));
        };
        let client_id = private_key.certificate().san_dns_name().unwrap().unwrap();

        let (session_token, session_state) =
            Session::<Created>::new(items_requests, usecase_id, client_id, return_url_template)?;

        self.sessions
            .write(session_state.into(), true)
            .await
            .map_err(VerificationError::SessionStore)?;

        info!("Session({session_token}): session created");
        Ok(session_token)
    }

    fn verify_ephemeral_id(
        &self,
        session_token: &SessionToken,
        url_params: &VerifierUrlParameters,
    ) -> Result<(), VerificationError> {
        if Utc::now() - EPHEMERAL_ID_VALIDITY_SECONDS > url_params.time {
            return Err(VerificationError::ExpiredEphemeralId(url_params.ephemeral_id.clone()));
        }
        hmac::verify(
            &self.ephemeral_id_secret,
            &Self::format_ephemeral_id_payload(session_token, &url_params.time),
            &url_params.ephemeral_id,
        )
        .map_err(|_| VerificationError::InvalidEphemeralId(url_params.ephemeral_id.clone()))?;

        Ok(())
    }

    pub async fn process_get_request(
        &self,
        session_token: &SessionToken,
        verifier_base_url: &BaseUrl,
        url_params: VerifierUrlParameters,
    ) -> Result<Jwt<VpAuthorizationRequest>, VerificationError> {
        let session: Session<Created> = self
            .sessions
            .get(session_token)
            .await
            .map_err(VerificationError::SessionStore)?
            .ok_or_else(|| VerificationError::UnknownSessionId(session_token.clone()))?
            .try_into()
            .unwrap();

        info!("Session({session_token}): get request");

        self.verify_ephemeral_id(session_token, &url_params)?;

        let (result, next) = match session
            .process_get_request(verifier_base_url, session_token, url_params.session_type, &self.keys)
            .await
        {
            Ok((jws, next)) => (Ok(jws), next.into()),
            Err((err, next)) => (Err(err), next.into()),
        };

        self.sessions
            .write(next, false)
            .await
            .map_err(VerificationError::SessionStore)
            .unwrap();

        result
    }

    pub async fn process_authorization_response(
        &self,
        session_token: &SessionToken,
        authorization_response_jwe: String,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<VpResponse, VerificationError> {
        let session: Session<WaitingForResponse> = self
            .sessions
            .get(session_token)
            .await
            .map_err(VerificationError::SessionStore)?
            .ok_or_else(|| VerificationError::UnknownSessionId(session_token.clone()))?
            .try_into()
            .unwrap();

        let (result, next) = session.process_authorization_response(
            authorization_response_jwe,
            time,
            self.trust_anchors
                .iter()
                .map(Into::<TrustAnchor<'_>>::into)
                .collect_vec()
                .as_slice(),
        );

        self.sessions
            .write(next.into(), false)
            .await
            .map_err(VerificationError::SessionStore)
            .unwrap();

        result
    }

    pub async fn status_response(
        &self,
        session_token: &SessionToken,
        ul_base: &BaseUrl,
        verifier_base_url: &BaseUrl,
        session_type: SessionType,
    ) -> Result<StatusResponse, VerificationError> {
        let response = match self
            .sessions
            .get(session_token)
            .await
            .map_err(VerificationError::SessionStore)?
            .ok_or_else(|| VerificationError::UnknownSessionId(session_token.clone()))?
            .data
        {
            DisclosureData::Created(Created { client_id, .. }) => {
                let time = Utc::now();
                let ul = Self::format_ul(
                    ul_base,
                    verifier_base_url,
                    session_token,
                    time,
                    self.generate_ephemeral_id(session_token, &time),
                    session_type,
                    client_id,
                )?;
                StatusResponse::Created { ul }
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
        redirect_uri: Option<Url>,
    ) -> Result<DisclosedAttributes, VerificationError> {
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
                        redirect_uri: None,
                        disclosed_attributes,
                    },
            }) => Ok(disclosed_attributes),
            DisclosureData::Done(Done {
                session_result:
                    SessionResult::Done {
                        redirect_uri: Some(expected_redirect_uri),
                        disclosed_attributes,
                    },
            }) if redirect_uri.as_ref().is_some_and(|h| h == &expected_redirect_uri) => Ok(disclosed_attributes),
            DisclosureData::Done(Done {
                session_result:
                    SessionResult::Done {
                        redirect_uri: Some(expected_redirect_uri),
                        ..
                    },
            }) => Err(VerificationError::RedirectUriMismatch(expected_redirect_uri)),
            _ => Err(VerificationError::SessionNotDone),
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

    fn format_ul(
        base_ul: &BaseUrl,
        verifier_base_url: &BaseUrl,
        session_token: &SessionToken,
        time: DateTime<Utc>,
        ephemeral_id: Vec<u8>,
        session_type: SessionType,
        client_id: String,
    ) -> Result<BaseUrl, VerificationError> {
        let mut request_uri = verifier_base_url
            .join_base_url("request_uri")
            .join_base_url(session_token.as_ref())
            .into_inner();
        request_uri.set_query(Some(&serde_urlencoded::to_string(VerifierUrlParameters {
            time,
            ephemeral_id,
            session_type,
        })?));

        let mut ul = base_ul.clone().into_inner();
        ul.set_query(Some(&serde_urlencoded::to_string(VpRequestUriObject {
            request_uri: request_uri.try_into().unwrap(),
            client_id,
        })?));

        Ok(ul.try_into().unwrap())
    }

    // formats the payload to hash to the ephemeral ID in a consistent way
    fn format_ephemeral_id_payload(session_token: &SessionToken, time: &DateTime<Utc>) -> Vec<u8> {
        // default (de)serialization of DateTime is the RFC 3339 format
        format!("{}|{}", session_token, time.to_rfc3339()).into()
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierUrlParameters {
    pub session_type: SessionType,
    #[serde_as(as = "Hex")]
    pub ephemeral_id: Vec<u8>,
    // default (de)serialization of DateTime is the RFC 3339 format
    pub time: DateTime<Utc>,
}

// Implementation of the typestate state engine follows.

// Transitioning functions and helpers valid for any state
impl<T: DisclosureState> Session<T> {
    // TODO check if this suffices for VP error handling
    fn transition_fail(self, error: &impl ToString) -> Session<Done> {
        self.transition(Done {
            session_result: SessionResult::Failed {
                error: error.to_string(),
            },
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
        usecase_id: String,
        client_id: String,
        return_url_template: Option<ReturnUrlTemplate>,
    ) -> Result<(SessionToken, Session<Created>), VerificationError> {
        let session_token = SessionToken::new_random();
        let session = Session::<Created> {
            state: SessionState::new(
                session_token.clone(),
                Created {
                    items_requests,
                    usecase_id,
                    client_id,
                    return_url_template,
                },
            ),
        };

        Ok((session_token, session))
    }

    fn new_redirect_uri(&self, session_token: &SessionToken, session_type: &SessionType) -> Option<Url> {
        self.state.data.return_url_template.as_ref().map(|u| {
            let mut url = u.clone().into_url(session_token);
            if matches!(session_type, SessionType::SameDevice) {
                url.query_pairs_mut().append_pair("return_nonce", &random_string(32));
            }
            url
        })
    }

    /// Process the device's request for the Authorization Request,
    /// returning a response to answer the device with and the next session state.
    async fn process_get_request(
        self,
        verifier_base_url: &BaseUrl,
        session_token: &SessionToken,
        session_type: SessionType,
        keys: &impl KeyRing,
    ) -> Result<(Jwt<VpAuthorizationRequest>, Session<WaitingForResponse>), (VerificationError, Session<Done>)> {
        // TODO return error object
        info!("Session({}): process get request", self.state.token);

        let (response, next) = match self.process_get_request_inner(verifier_base_url, keys).await {
            Ok((jws, auth_request, enc_keypair)) => {
                let next = WaitingForResponse {
                    auth_request,
                    redirect_uri: self.new_redirect_uri(session_token, &session_type),
                    encryption_key: EncryptionPrivateKey(enc_keypair),
                };
                let next = self.transition(next);
                Ok((jws, next))
            }
            Err(err) => {
                warn!(
                    "Session({}): process get request failed, returning error",
                    self.state.token
                );
                let next = self.transition_fail(&err);
                Err((err, next))
            }
        }?;

        Ok((response, next))
    }

    // Helper function that returns ordinary errors instead of `Session<...>`
    async fn process_get_request_inner(
        &self,
        verifier_base_url: &BaseUrl,
        keys: &impl KeyRing,
    ) -> Result<(Jwt<VpAuthorizationRequest>, VpAuthorizationRequest, EcKeyPair), VerificationError> {
        let cert_pair = keys
            .private_key(&self.state().usecase_id)
            .ok_or_else(|| VerificationError::UnknownCertificate(self.state().usecase_id.clone()))?;

        let nonce = random_string(32);
        let response_uri = verifier_base_url
            .join_base_url("response_uri")
            .join_base_url(self.state.token.as_ref());
        let encryption_keypair = EcKeyPair::generate(EcCurve::P256).unwrap();

        let auth_request = VpAuthorizationRequest::new(
            &self.state.data.items_requests,
            cert_pair.certificate(),
            nonce.clone(),
            encryption_keypair.to_jwk_public_key().try_into().unwrap(),
            response_uri,
        )
        .unwrap();

        let jws = jwt::sign_with_certificate(&auth_request, cert_pair).await.unwrap();

        Ok((jws, auth_request, encryption_keypair))
    }
}

impl Session<WaitingForResponse> {
    /// Process the user's encrypted `VpAuthorizationResponse`, i.e. its disclosure,
    /// returning a response to answer the device with and the next session state.
    fn process_authorization_response(
        self,
        authorization_response_jwe: String,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> (Result<VpResponse, VerificationError>, Session<Done>) {
        info!("Session({}): process response", self.state.token);

        let (response, next) =
            match self.process_authorization_response_inner(authorization_response_jwe, time, trust_anchors) {
                Ok((response, disclosed_attributes)) => {
                    let redirect_uri = self.state.data.redirect_uri.clone();
                    (Ok(response), self.transition_finish(disclosed_attributes, redirect_uri))
                }
                Err(err) => {
                    warn!(
                        "Session({}): process response failed, returning decoding error",
                        self.state.token
                    );
                    let next = self.transition_fail(&err);
                    (Err(err), next)
                }
            };

        (response, next)
    }

    // Helper function that returns ordinary errors instead of `Session<Done>`
    fn process_authorization_response_inner(
        &self,
        jwe: String,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<(VpResponse, DisclosedAttributes), VerificationError> {
        debug!(
            "Session({}): decrypting and deserializing Authorization Response JWE",
            self.state.token
        );

        let data = &self.state.data;
        let nonce = data.auth_request.oauth_request.nonce.as_ref().unwrap().clone();

        // Decrypt and verify the Authorization Response
        let (auth_response, mdoc_nonce) = VpAuthorizationResponse::decrypt(jwe, &data.encryption_key.0, nonce).unwrap();
        let disclosed = auth_response
            .verify(&data.auth_request, mdoc_nonce, time, trust_anchors)
            .unwrap();

        let response = VpResponse {
            redirect_uri: data.redirect_uri.clone().map(|u| u.try_into().unwrap()),
        };

        Ok((response, disclosed))
    }

    fn transition_finish(self, disclosed_attributes: DisclosedAttributes, redirect_uri: Option<Url>) -> Session<Done> {
        self.transition(Done {
            session_result: SessionResult::Done {
                disclosed_attributes,
                redirect_uri,
            },
        })
    }
}
