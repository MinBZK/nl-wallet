//! RP software, for verifying mdoc disclosures, see [`DeviceResponse::verify()`].

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use chrono::DateTime;
use chrono::SecondsFormat;
use chrono::Utc;
use derive_more::AsRef;
use derive_more::From;
use josekit::jwk::alg::ec::EcCurve;
use josekit::jwk::alg::ec::EcKeyPair;
use josekit::jwk::Jwk;
use josekit::JoseError;
use ring::hmac;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use serde_with::hex::Hex;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use tokio::task::JoinHandle;
use tracing::debug;
use tracing::info;
use tracing::warn;

use nl_wallet_mdoc::server_keys::KeyPair;
use nl_wallet_mdoc::utils::x509::CertificateError;
use nl_wallet_mdoc::verifier::DisclosedAttributes;
use nl_wallet_mdoc::verifier::ItemsRequests;
use wallet_common::generator::Generator;
use wallet_common::jwt::Jwt;
use wallet_common::jwt::JwtError;
use wallet_common::urls::BaseUrl;
use wallet_common::utils::random_string;

use crate::jwt;
use crate::openid4vp::AuthRequestError;
use crate::openid4vp::AuthResponseError;
use crate::openid4vp::IsoVpAuthorizationRequest;
use crate::openid4vp::RequestUriMethod;
use crate::openid4vp::VpAuthorizationRequest;
use crate::openid4vp::VpAuthorizationResponse;
use crate::openid4vp::VpRequestUriObject;
use crate::openid4vp::VpResponse;
use crate::return_url::ReturnUrlTemplate;
use crate::server_state::Expirable;
use crate::server_state::HasProgress;
use crate::server_state::Progress;
use crate::server_state::SessionState;
use crate::server_state::SessionStore;
use crate::server_state::SessionStoreError;
use crate::server_state::SessionToken;
use crate::server_state::CLEANUP_INTERVAL_SECONDS;
use crate::AuthorizationErrorCode;
use crate::ErrorResponse;
use crate::VpAuthorizationErrorCode;

pub const EPHEMERAL_ID_VALIDITY_SECONDS: Duration = Duration::from_secs(10);

/// Errors that can occur during processing of any of the endpoints.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("error with sessionstore: {0}")]
    SessionStore(#[from] SessionStoreError),
    #[error("unknown session: {0}")]
    UnknownSession(SessionToken),
    #[error("session not in expected state, found: {0}")]
    UnexpectedState(SessionStatus),
}

/// Errors returned by the new session endpoint, used by the RP.
#[derive(Debug, thiserror::Error)]
pub enum NewSessionError {
    #[error("session error: {0}")]
    Session(#[from] SessionError),
    #[error("no ItemsRequest: can't request a disclosure of 0 attributes")]
    NoItemsRequests,
    #[error("unknown use case: {0}")]
    UnknownUseCase(String),
    #[error("presence or absence of return url template does not match configuration for the required use case")]
    ReturnUrlConfigurationMismatch,
}

/// Errors returned by the session status endpoint, used by the web front-end.
#[derive(Debug, thiserror::Error)]
pub enum SessionStatusError {
    #[error("session error: {0}")]
    Session(#[from] SessionError),
    #[error("URL encoding error: {0}")]
    UrlEncoding(#[from] serde_urlencoded::ser::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum CancelSessionError {
    #[error("session error: {0}")]
    Session(#[from] SessionError),
}

/// Errors returned by the disclosed attributes endpoint, used by the RP.
#[derive(Debug, thiserror::Error)]
pub enum DisclosedAttributesError {
    #[error("session error: {0}")]
    Session(#[from] SessionError),
    #[error("missing nonce in redirect URI")]
    RedirectUriNonceMissing,
    #[error("redirect URI nonce '{0}' does not equal the expected nonce")]
    RedirectUriNonceMismatch(String),
}

/// Errors returned by the endpoint that returns the Authorization Request.
#[derive(thiserror::Error, Debug)]
pub enum GetAuthRequestError {
    #[error("session error: {0}")]
    Session(#[from] SessionError),
    #[error("the ephemeral ID {} is invalid", hex::encode(.0))]
    InvalidEphemeralId(Vec<u8>),
    #[error("the ephemeral ID {} has expired", hex::encode(.0))]
    ExpiredEphemeralId(Vec<u8>),
    #[error("error creating ephemeral encryption keypair: {0}")]
    EncryptionKey(#[from] JoseError),
    #[error("error creating Authorization Request: {0}")]
    AuthRequest(#[from] AuthRequestError),
    #[error("error signing Authorization Request JWE: {0}")]
    Jwt(#[from] JwtError),
    #[error("presence or absence of return url template does not match configuration for the required use case")]
    ReturnUrlConfigurationMismatch,
    #[error("unknown use case: {0}")]
    UnknownUseCase(String),
    #[error("missing query parameters")]
    QueryParametersMissing,
    #[error("failed to deserialize query parameters: {0}")]
    QueryParametersDeserialization(#[from] serde_urlencoded::de::Error),
}

/// Errors returned by the endpoint to which the user posts the Authorization Response.
#[derive(thiserror::Error, Debug)]
pub enum PostAuthResponseError {
    #[error("session error: {0}")]
    Session(#[from] SessionError),
    #[error("error decrypting or verifying Authorization Response JWE: {0}")]
    AuthResponse(#[from] AuthResponseError),
}

/// Errors that can occur when creating a [`UseCase`] instance.
#[derive(Debug, thiserror::Error)]
pub enum UseCaseCertificateError {
    #[error("missing DNS SAN from RP certificate")]
    MissingSAN,
    #[error("RP certificate error: {0}")]
    Certificate(#[from] CertificateError),
}

#[derive(thiserror::Error, Debug)]
#[error("user aborted with error: {0:?}")]
pub struct UserError(ErrorResponse<VpAuthorizationErrorCode>);

#[derive(thiserror::Error, Debug)]
pub struct WithRedirectUri<T: std::error::Error> {
    #[source]
    pub error: T,
    pub redirect_uri: Option<BaseUrl>,
}

impl<T: std::error::Error> Display for WithRedirectUri<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}, redirect_uri: {:?}", self.error, self.redirect_uri)
    }
}

impl<T: std::error::Error> From<T> for WithRedirectUri<T> {
    fn from(error: T) -> Self {
        Self::new(error, None)
    }
}

impl<T: std::error::Error> WithRedirectUri<T> {
    fn new(error: T, redirect_uri: Option<BaseUrl>) -> Self {
        Self { error, redirect_uri }
    }
}

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
    redirect_uri_template: Option<ReturnUrlTemplate>,
}

/// State for a session that is waiting for the user's disclosure, i.e., the device has contacted us at the session URL.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WaitingForResponse {
    auth_request: IsoVpAuthorizationRequest,
    encryption_key: EncryptionPrivateKey,
    redirect_uri: Option<RedirectUri>,
}

/// State for a session that has ended (for any reason).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Done {
    session_result: SessionResult,
}

/// The outcome of a session: the disclosed attributes if they have been successfully received and verified.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE", tag = "status")]
enum SessionResult {
    Done {
        disclosed_attributes: DisclosedAttributes,
        redirect_uri_nonce: Option<String>,
    },
    Failed {
        error: String,
    },
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectUri {
    uri: BaseUrl,
    nonce: String,
}

/// Wrapper for [`EcKeyPair`] that can be serialized.
#[derive(Debug, Clone, AsRef, From)]
struct EncryptionPrivateKey(EcKeyPair);

// Ordinarily we might use DER encoding here instead of PEM, but `EcKeyPair::to_der_private_key()` does not encode
// to PKCS8 which is expected by `EcKeyPair::from_der()`. A workaround would be to explicitly pass the EC curve
// (P256 currently in our case) as a parameter to `EcKeyPair::from_der()`, but that would hinder a potential future
// implementation of other curves or signature schemes. So we use the JWK functions instead, which don't have
// this issue.
impl Serialize for EncryptionPrivateKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_ref().to_jwk_private_key().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for EncryptionPrivateKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(EncryptionPrivateKey::from(
            EcKeyPair::from_jwk(&Jwk::deserialize(deserializer)?).map_err(serde::de::Error::custom)?,
        ))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VpToken {
    pub vp_token: String,
}

/// Sent by the wallet to the `response_uri`: either an Authorization Response JWE or an error, which either indicates
/// that they refuse disclosure, or is an actual error that the wallet encountered during the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WalletAuthResponse {
    Response(VpToken),
    Error(ErrorResponse<VpAuthorizationErrorCode>),
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
    WaitingForResponse(Box<WaitingForResponse>),
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
        });
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
    type Error = SessionError;

    fn try_from(value: SessionState<DisclosureData>) -> Result<Self, Self::Error> {
        let session_data = match value.data {
            DisclosureData::Created(session_data) => Ok(session_data),
            data => Err(SessionError::UnexpectedState(data.into())),
        }?;

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
            data: DisclosureData::WaitingForResponse(Box::new(value.state.data)),
            token: value.state.token,
            last_active: value.state.last_active,
        }
    }
}

impl TryFrom<SessionState<DisclosureData>> for Session<WaitingForResponse> {
    type Error = SessionError;

    fn try_from(value: SessionState<DisclosureData>) -> Result<Self, Self::Error> {
        let session_data = match value.data {
            DisclosureData::WaitingForResponse(session_data) => Ok(session_data),
            data => Err(SessionError::UnexpectedState(data.into())),
        }?;

        Ok(Session::<WaitingForResponse> {
            state: SessionState {
                data: *session_data,
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

/// Session status as returned by the `status_response()` method and eventually the status endpoint in `wallet_server`.
/// As this endpoint is meant to be public, it contains no other data than the (flattened) state, plus a potential
/// universal link that the wallet app can use to start disclosure.
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "status")]
pub enum StatusResponse {
    Created { ul: Option<BaseUrl> },
    WaitingForResponse,
    Done,
    Failed,
    Cancelled,
    Expired,
}

/// Session status as contained in `SessionError::UnexpectedState`. This has the same flattened structure as
/// [`StatusResponse`], but is meant for internal use only to indicate the current state of the session.
/// Note that the error reason included in the `Failed` state is only meant to be included in an error response
/// from the `disclosed_attributes` endpoint in `wallet_server`, not any other endpoint.
#[derive(Debug, Clone, strum::Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionStatus {
    Created,
    WaitingForResponse,
    Done,
    Failed { error: String },
    Cancelled,
    Expired,
}

impl From<DisclosureData> for SessionStatus {
    fn from(value: DisclosureData) -> Self {
        match value {
            DisclosureData::Created(_) => Self::Created,
            DisclosureData::WaitingForResponse(_) => Self::WaitingForResponse,
            DisclosureData::Done(Done { session_result }) => match session_result {
                SessionResult::Done { .. } => Self::Done,
                SessionResult::Failed { error } => Self::Failed { error },
                SessionResult::Cancelled => Self::Cancelled,
                SessionResult::Expired => Self::Expired,
            },
        }
    }
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

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionTypeReturnUrl {
    Neither,
    #[default]
    SameDevice,
    Both,
}

#[derive(Debug, From, AsRef)]
pub struct UseCases(HashMap<String, UseCase>);

#[derive(Debug)]
pub struct UseCase {
    pub key_pair: KeyPair,
    pub client_id: String,
    pub session_type_return_url: SessionTypeReturnUrl,
}

impl UseCase {
    pub fn try_new(
        key_pair: KeyPair,
        session_type_return_url: SessionTypeReturnUrl,
    ) -> Result<Self, UseCaseCertificateError> {
        let client_id = String::from(
            key_pair
                .certificate()
                .san_dns_name()?
                .ok_or(UseCaseCertificateError::MissingSAN)?,
        );
        let use_case = Self {
            key_pair,
            client_id,
            session_type_return_url,
        };

        Ok(use_case)
    }
}

#[derive(Debug)]
pub struct Verifier<S> {
    use_cases: UseCases,
    sessions: Arc<S>,
    cleanup_task: JoinHandle<()>,
    trust_anchors: Vec<TrustAnchor<'static>>,
    ephemeral_id_secret: hmac::Key,
}

impl<S> Drop for Verifier<S> {
    fn drop(&mut self) {
        // Stop the task at the next .await
        self.cleanup_task.abort();
    }
}

impl<S> Verifier<S>
where
    S: SessionStore<DisclosureData>,
{
    /// Create a new [`Verifier`].
    ///
    /// - `use_cases` contains configuration per use case, including a certificate and corresponding private key for use
    ///   in RP authentication.
    /// - `sessions` will contain all sessions.
    /// - `trust_anchors` contains self-signed X509 CA certificates acting as trust anchor for the mdoc verification:
    ///   the mdoc verification function [`Document::verify()`] returns true if the mdoc verifies against one of these
    ///   CAs.
    /// - `ephemeral_id_secret` is used as a HMAC secret to create ephemeral session IDs.
    pub fn new(
        use_cases: UseCases,
        sessions: S,
        trust_anchors: Vec<TrustAnchor<'static>>,
        ephemeral_id_secret: hmac::Key,
    ) -> Self
    where
        S: Send + Sync + 'static,
    {
        let sessions = Arc::new(sessions);
        Self {
            use_cases,
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
    ) -> Result<SessionToken, NewSessionError> {
        info!("create verifier session: {usecase_id}");

        if items_requests.0.is_empty() {
            return Err(NewSessionError::NoItemsRequests);
        }

        let use_case = match self.use_cases.as_ref().get(&usecase_id) {
            Some(use_case) => use_case,
            None => return Err(NewSessionError::UnknownUseCase(usecase_id)),
        };

        // Check if we should or should not have received a return URL
        // template, based on the configuration for the use case.
        if match use_case.session_type_return_url {
            SessionTypeReturnUrl::Neither => return_url_template.is_some(),
            SessionTypeReturnUrl::SameDevice | SessionTypeReturnUrl::Both => return_url_template.is_none(),
        } {
            return Err(NewSessionError::ReturnUrlConfigurationMismatch);
        }

        let session_state = Session::<Created>::new(
            items_requests,
            usecase_id,
            use_case.client_id.clone(),
            return_url_template,
        );
        let session_token = session_state.state.token.clone();

        self.sessions
            .write(session_state.into(), true)
            .await
            .map_err(SessionError::SessionStore)?;

        info!("Session({session_token}): session created");
        Ok(session_token)
    }

    fn verify_ephemeral_id(
        &self,
        session_token: &SessionToken,
        url_params: &VerifierUrlParameters,
    ) -> Result<(), GetAuthRequestError> {
        if Utc::now() - EPHEMERAL_ID_VALIDITY_SECONDS > url_params.time {
            return Err(GetAuthRequestError::ExpiredEphemeralId(url_params.ephemeral_id.clone()));
        }
        hmac::verify(
            &self.ephemeral_id_secret,
            &Self::format_ephemeral_id_payload(session_token, &url_params.time),
            &url_params.ephemeral_id,
        )
        .map_err(|_| GetAuthRequestError::InvalidEphemeralId(url_params.ephemeral_id.clone()))?;

        Ok(())
    }

    async fn get_session_state(
        &self,
        session_token: &SessionToken,
    ) -> Result<SessionState<DisclosureData>, SessionError> {
        self.sessions
            .get(session_token)
            .await?
            .ok_or_else(|| SessionError::UnknownSession(session_token.clone()))
    }

    async fn get_session<T, E>(&self, session_token: &SessionToken) -> Result<Session<T>, WithRedirectUri<E>>
    where
        T: DisclosureState,
        E: std::error::Error + From<SessionError>,
        Session<T>: TryFrom<SessionState<DisclosureData>, Error = SessionError>,
    {
        let session = self
            .get_session_state(session_token)
            .await
            .map_err(E::from)?
            .try_into()
            .map_err(E::from)?;

        Ok(session)
    }

    pub async fn process_get_request(
        &self,
        session_token: &SessionToken,
        response_uri: BaseUrl,
        query: Option<&str>,
        wallet_nonce: Option<String>,
    ) -> Result<Jwt<VpAuthorizationRequest>, WithRedirectUri<GetAuthRequestError>> {
        let session: Session<Created> = self.get_session(session_token).await?;

        info!("Session({session_token}): get request");

        let url_params: VerifierUrlParameters =
            serde_urlencoded::from_str(query.ok_or(GetAuthRequestError::QueryParametersMissing)?)
                .map_err(GetAuthRequestError::QueryParametersDeserialization)?;

        // Verify the ephemeral ID here as opposed to inside `session.process_get_request()`, so that if the
        // ephemeral ID is too old e.g. because the user's internet connection was very slow, then we don't fail the
        // session. This means that the QR code/UL stays on the website so that the user can try again.
        self.verify_ephemeral_id(session_token, &url_params)?;

        let (result, redirect_uri, next) = match session
            .process_get_request(
                session_token,
                response_uri,
                url_params.session_type,
                wallet_nonce,
                &self.use_cases,
            )
            .await
        {
            Ok((jws, next)) => (
                Ok(jws),
                next.state().redirect_uri.as_ref().map(|u| u.uri.clone()),
                next.into(),
            ),
            Err((err, next)) => {
                let redirect_uri = err.redirect_uri.clone();
                (Err(err), redirect_uri, next.into())
            }
        };

        self.sessions
            .write(next, false)
            .await
            .map_err(|err| WithRedirectUri::new(SessionError::SessionStore(err).into(), redirect_uri))?;

        result
    }

    pub async fn process_authorization_response(
        &self,
        session_token: &SessionToken,
        wallet_response: WalletAuthResponse,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<VpResponse, WithRedirectUri<PostAuthResponseError>> {
        let session: Session<WaitingForResponse> = self.get_session(session_token).await?;

        let (result, next) = session.process_authorization_response(wallet_response, time, &self.trust_anchors);

        self.sessions.write(next.into(), false).await.map_err(|err| {
            WithRedirectUri::new(
                SessionError::SessionStore(err).into(),
                match &result {
                    Ok(response) => response.redirect_uri.clone(),
                    Err(err) => err.redirect_uri.clone(),
                },
            )
        })?;

        result
    }

    pub async fn status_response(
        &self,
        session_token: &SessionToken,
        session_type: Option<SessionType>,
        ul_base: &BaseUrl,
        request_uri: BaseUrl,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<StatusResponse, SessionStatusError> {
        let response = match self.get_session_state(session_token).await?.data {
            DisclosureData::Created(Created { client_id, .. }) => {
                let ul = session_type
                    .map(|session_type| {
                        let time = time.generate();
                        Self::format_ul(
                            ul_base,
                            request_uri,
                            time,
                            Self::generate_ephemeral_id(&self.ephemeral_id_secret, session_token, &time),
                            session_type,
                            client_id,
                        )
                    })
                    .transpose()?;

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

    pub async fn cancel(&self, session_token: &SessionToken) -> Result<(), CancelSessionError> {
        let SessionState { data, token, .. } = self.get_session_state(session_token).await?;

        // Create a new `SessionState<DisclosureData>` if the session
        // is in the `CREATED` or `WAITING_FOR_RESPONSE` state.
        let cancelled_session_state = match data {
            DisclosureData::Created(_) | DisclosureData::WaitingForResponse(_) => SessionState::new(
                token,
                DisclosureData::Done(Done {
                    session_result: SessionResult::Cancelled,
                }),
            ),
            DisclosureData::Done(_) => return Err(SessionError::UnexpectedState(data.into()).into()),
        };

        self.sessions
            .write(cancelled_session_state, false)
            .await
            .map_err(SessionError::SessionStore)?;

        Ok(())
    }

    /// Returns the disclosed attributes for a session with status `Done` and an error otherwise
    pub async fn disclosed_attributes(
        &self,
        session_token: &SessionToken,
        redirect_uri_nonce: Option<String>,
    ) -> Result<DisclosedAttributes, DisclosedAttributesError> {
        let disclosure_data = self.get_session_state(session_token).await?.data;

        match disclosure_data {
            DisclosureData::Done(Done {
                session_result:
                    SessionResult::Done {
                        redirect_uri_nonce: expected_nonce,
                        disclosed_attributes,
                    },
            }) => match (redirect_uri_nonce, expected_nonce) {
                (_, None) => Ok(disclosed_attributes),
                (None, Some(_)) => Err(DisclosedAttributesError::RedirectUriNonceMissing),
                (Some(received), Some(expected)) if received == expected => Ok(disclosed_attributes),
                (Some(received), Some(_)) => Err(DisclosedAttributesError::RedirectUriNonceMismatch(received)),
            },
            data => Err(SessionError::UnexpectedState(data.into()))?,
        }
    }
}

impl<S> Verifier<S> {
    fn generate_ephemeral_id(
        ephemeral_id_secret: &hmac::Key,
        session_token: &SessionToken,
        time: &DateTime<Utc>,
    ) -> Vec<u8> {
        let ephemeral_id = hmac::sign(
            ephemeral_id_secret,
            &Self::format_ephemeral_id_payload(session_token, time),
        )
        .as_ref()
        .to_vec();
        ephemeral_id
    }

    fn format_ul(
        base_ul: &BaseUrl,
        request_uri: BaseUrl,
        time: DateTime<Utc>,
        ephemeral_id: Vec<u8>,
        session_type: SessionType,
        client_id: String,
    ) -> Result<BaseUrl, serde_urlencoded::ser::Error> {
        let mut request_uri = request_uri.into_inner();
        request_uri.set_query(Some(&serde_urlencoded::to_string(VerifierUrlParameters {
            time,
            ephemeral_id,
            session_type,
        })?));

        let mut ul = base_ul.clone().into_inner();
        ul.set_query(Some(&serde_urlencoded::to_string(VpRequestUriObject {
            request_uri: request_uri.try_into().unwrap(), // safe because we constructed request_uri from a BaseUrl
            client_id,
            request_uri_method: Some(RequestUriMethod::POST),
        })?));

        Ok(ul.try_into().unwrap()) // safe because we constructed request_uri from a BaseUrl
    }

    // formats the payload to hash to the ephemeral ID in a consistent way
    fn format_ephemeral_id_payload(session_token: &SessionToken, time: &DateTime<Utc>) -> Vec<u8> {
        // default (de)serialization of DateTime is the RFC 3339 format
        format!(
            "{}|{}",
            session_token,
            time.to_rfc3339_opts(SecondsFormat::AutoSi, true)
        )
        .into()
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
    ) -> Session<Created> {
        Session::<Created> {
            state: SessionState::new(
                SessionToken::new_random(),
                Created {
                    items_requests,
                    usecase_id,
                    client_id,
                    redirect_uri_template: return_url_template,
                },
            ),
        }
    }

    /// Process the device's request for the Authorization Request,
    /// returning a response to answer the device with and the next session state.
    async fn process_get_request(
        self,
        session_token: &SessionToken,
        response_uri: BaseUrl,
        session_type: SessionType,
        wallet_nonce: Option<String>,
        use_cases: &UseCases,
    ) -> Result<
        (Jwt<VpAuthorizationRequest>, Session<WaitingForResponse>),
        (WithRedirectUri<GetAuthRequestError>, Session<Done>),
    > {
        info!("Session({}): process get request", self.state.token);

        let (response, next) = match self
            .process_get_request_inner(session_token, response_uri, session_type, wallet_nonce, use_cases)
            .await
        {
            Ok((jws, auth_request, redirect_uri, enc_keypair)) => {
                let next = WaitingForResponse {
                    auth_request,
                    encryption_key: EncryptionPrivateKey::from(enc_keypair),
                    redirect_uri,
                };
                let next = self.transition(next);
                Ok((jws, next))
            }
            Err(err) => {
                warn!(
                    "Session({}): process get request failed, returning error",
                    self.state.token
                );
                let next = self.transition_fail(&err.error);
                Err((err, next))
            }
        }?;

        Ok((response, next))
    }

    // Helper function that returns ordinary errors instead of `Session<...>`
    async fn process_get_request_inner(
        &self,
        session_token: &SessionToken,
        response_uri: BaseUrl,
        session_type: SessionType,
        wallet_nonce: Option<String>,
        use_cases: &UseCases,
    ) -> Result<
        (
            Jwt<VpAuthorizationRequest>,
            IsoVpAuthorizationRequest,
            Option<RedirectUri>,
            EcKeyPair,
        ),
        WithRedirectUri<GetAuthRequestError>,
    > {
        let usecase_id = &self.state().usecase_id;
        let usecase = use_cases.as_ref().get(usecase_id);
        let Some(usecase) = usecase else {
            // This should not happen except when the configuration has changed during this session.
            warn!("configuration inconsistency: existing session referenced nonexisting usecase '{usecase_id}'");
            return Err(GetAuthRequestError::UnknownUseCase(usecase_id.to_string()).into());
        };

        // Determine if we should include a redirect URI, based on the use case configuration and session type.
        let redirect_uri = Self::redirect_uri_and_nonce(
            session_token,
            usecase.session_type_return_url,
            session_type,
            self.state().redirect_uri_template.clone(),
        )?;

        // Construct the Authorization Request.
        let nonce = random_string(32);
        let encryption_keypair = EcKeyPair::generate(EcCurve::P256)
            .map_err(|err| WithRedirectUri::new(err.into(), redirect_uri.as_ref().map(|u| u.uri.clone())))?;
        let auth_request = IsoVpAuthorizationRequest::new(
            &self.state.data.items_requests,
            usecase.key_pair.certificate(),
            nonce.clone(),
            encryption_keypair.to_jwk_public_key().try_into().unwrap(), // safe because we just constructed this key
            response_uri,
            wallet_nonce,
        )
        .map_err(|err| WithRedirectUri::new(err.into(), redirect_uri.as_ref().map(|u| u.uri.clone())))?;

        let vp_auth_request = VpAuthorizationRequest::from(auth_request.clone());
        let jws = jwt::sign_with_certificate(&vp_auth_request, &usecase.key_pair)
            .await
            .map_err(|err| WithRedirectUri::new(err.into(), redirect_uri.as_ref().map(|u| u.uri.clone())))?;

        Ok((jws, auth_request, redirect_uri, encryption_keypair))
    }

    fn redirect_uri_and_nonce(
        session_token: &SessionToken,
        session_type_return_url: SessionTypeReturnUrl,
        session_type: SessionType,
        template: Option<ReturnUrlTemplate>,
    ) -> Result<Option<RedirectUri>, GetAuthRequestError> {
        match (session_type_return_url, session_type, template) {
            (SessionTypeReturnUrl::Both, _, Some(uri_template))
            | (SessionTypeReturnUrl::SameDevice, SessionType::SameDevice, Some(uri_template)) => {
                let nonce = random_string(32);
                let mut redirect_uri = uri_template.into_url(session_token);
                redirect_uri.query_pairs_mut().append_pair("nonce", &nonce);
                Ok(Some(RedirectUri {
                    uri: redirect_uri.try_into().unwrap(),
                    nonce,
                }))
            }
            (SessionTypeReturnUrl::Neither, _, _) | (SessionTypeReturnUrl::SameDevice, SessionType::CrossDevice, _) => {
                Ok(None)
            }
            (_, _, template) => {
                // We checked for this case when the session was created, so this should not happen
                // except when the configuration has changed during this session.
                warn!(
                    "configuration inconsistency: return URL configuration mismatch type {0:?}, session type {1:?}, \
                     redirect URI template {2:?}",
                    session_type_return_url, session_type, template
                );
                Err(GetAuthRequestError::ReturnUrlConfigurationMismatch)
            }
        }
    }
}

impl Session<WaitingForResponse> {
    /// Process the user's encrypted `VpAuthorizationResponse`, i.e. its disclosure,
    /// returning a response to answer the device with and the next session state.
    ///
    /// Unlike many similar method, this method does not have an `_inner()` version that returns `Result<_,_>`
    /// because it differs from similar methods in the following aspect: in some cases (to wit, if the user
    /// sent an error instead of a disclosure) then we should respond with HTTP 200 to the user (mandated by
    /// the OpenID4VP spec), while we fail our session. This does not neatly fit in the `_inner()` method pattern.
    fn process_authorization_response(
        self,
        wallet_response: WalletAuthResponse,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> (
        Result<VpResponse, WithRedirectUri<PostAuthResponseError>>,
        Session<Done>,
    ) {
        debug!("Session({}): process response", self.state.token);

        let jwe = match wallet_response {
            WalletAuthResponse::Response(VpToken { vp_token }) => vp_token,
            WalletAuthResponse::Error(err) => {
                // Check if the error code indicates that the user refused to disclose.
                let user_refused = matches!(
                    err.error,
                    VpAuthorizationErrorCode::AuthorizationError(AuthorizationErrorCode::AccessDenied)
                );

                let response = self.ok_response();
                let next = if user_refused {
                    self.transition_abort()
                } else {
                    // If the user sent any other error, fail the session.
                    self.transition_fail(&UserError(err))
                };
                // Return a non-error response to the wallet (including the redirect URI) to indicate
                // we successfully processed its error response.
                return (Ok(response), next);
            }
        };

        debug!(
            "Session({}): process response: decrypting and deserializing Authorization Response JWE",
            self.state.token
        );
        let (result, next) = match VpAuthorizationResponse::decrypt_and_verify(
            &jwe,
            self.state().encryption_key.as_ref(),
            &self.state().auth_request,
            time,
            trust_anchors,
        ) {
            Ok(disclosed) => {
                let redirect_uri_nonce = self.state().redirect_uri.as_ref().map(|u| u.nonce.clone());
                let response = self.ok_response();
                let next = self.transition_finish(disclosed, redirect_uri_nonce);
                (Ok(response), next)
            }
            Err(err) => {
                let redirect_uri = self.state().redirect_uri.as_ref().map(|u| u.uri.clone());
                let next = self.transition_fail(&err);
                (Err(WithRedirectUri::new(err.into(), redirect_uri)), next)
            }
        };

        (result, next)
    }

    fn ok_response(&self) -> VpResponse {
        VpResponse {
            redirect_uri: self.state().redirect_uri.as_ref().map(|u| u.uri.clone()),
        }
    }

    fn transition_finish(self, disclosed_attributes: DisclosedAttributes, nonce: Option<String>) -> Session<Done> {
        self.transition(Done {
            session_result: SessionResult::Done {
                disclosed_attributes,
                redirect_uri_nonce: nonce,
            },
        })
    }

    fn transition_abort(self) -> Session<Done> {
        self.transition(Done {
            session_result: SessionResult::Cancelled,
        })
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use chrono::DateTime;
    use chrono::Duration;
    use chrono::Utc;
    use indexmap::IndexMap;
    use itertools::Itertools;
    use ring::hmac;
    use ring::rand;
    use rstest::rstest;

    use nl_wallet_mdoc::server_keys::generate::SelfSignedCa;
    use nl_wallet_mdoc::utils::reader_auth::ReaderRegistration;
    use nl_wallet_mdoc::ItemsRequest;
    use wallet_common::generator::Generator;
    use wallet_common::generator::TimeGenerator;

    use crate::server_state::MemorySessionStore;
    use crate::server_state::SessionToken;

    use super::AuthorizationErrorCode;
    use super::DisclosedAttributesError;
    use super::DisclosureData;
    use super::Done;
    use super::ErrorResponse;
    use super::GetAuthRequestError;
    use super::HashMap;
    use super::ItemsRequests;
    use super::NewSessionError;
    use super::SessionError;
    use super::SessionResult;
    use super::SessionState;
    use super::SessionStatus;
    use super::SessionStore;
    use super::SessionType;
    use super::SessionTypeReturnUrl;
    use super::StatusResponse;
    use super::UseCase;
    use super::Verifier;
    use super::VpAuthorizationErrorCode;
    use super::VpRequestUriObject;
    use super::WalletAuthResponse;
    use super::EPHEMERAL_ID_VALIDITY_SECONDS;

    const DISCLOSURE_DOC_TYPE: &str = "example_doctype";
    const DISCLOSURE_NAME_SPACE: &str = "example_namespace";
    const DISCLOSURE_ATTRS: [(&str, bool); 2] = [("first_name", true), ("family_name", false)];

    const DISCLOSURE_USECASE_NO_REDIRECT_URI: &str = "example_usecase_no_redirect_uri";
    const DISCLOSURE_USECASE: &str = "example_usecase";
    const DISCLOSURE_USECASE_ALL_REDIRECT_URI: &str = "example_usecase_all_redirect_uri";

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

    fn create_verifier() -> Verifier<MemorySessionStore<DisclosureData>> {
        // Initialize server state
        let ca = SelfSignedCa::generate_reader_mock_ca().unwrap();
        let trust_anchors = vec![ca.to_trust_anchor().to_owned()];
        let reader_registration = Some(ReaderRegistration::new_mock());

        let use_cases = HashMap::from([
            (
                DISCLOSURE_USECASE_NO_REDIRECT_URI.to_string(),
                UseCase {
                    key_pair: ca.generate_reader_mock(reader_registration.clone()).unwrap(),
                    session_type_return_url: SessionTypeReturnUrl::Neither,
                    client_id: "client_id".to_string(),
                },
            ),
            (
                DISCLOSURE_USECASE.to_string(),
                UseCase {
                    key_pair: ca.generate_reader_mock(reader_registration.clone()).unwrap(),
                    session_type_return_url: SessionTypeReturnUrl::SameDevice,
                    client_id: "client_id".to_string(),
                },
            ),
            (
                DISCLOSURE_USECASE_ALL_REDIRECT_URI.to_string(),
                UseCase {
                    key_pair: ca.generate_reader_mock(reader_registration).unwrap(),
                    session_type_return_url: SessionTypeReturnUrl::Both,
                    client_id: "client_id".to_string(),
                },
            ),
        ])
        .into();

        let session_store = MemorySessionStore::default();

        Verifier::new(
            use_cases,
            session_store,
            trust_anchors,
            hmac::Key::generate(hmac::HMAC_SHA256, &rand::SystemRandom::new()).unwrap(),
        )
    }

    #[rstest]
    #[case(DISCLOSURE_USECASE_NO_REDIRECT_URI, false, true)]
    #[case(DISCLOSURE_USECASE_NO_REDIRECT_URI, true, false)]
    #[case(DISCLOSURE_USECASE, false, false)]
    #[case(DISCLOSURE_USECASE, true, true)]
    #[case(DISCLOSURE_USECASE_ALL_REDIRECT_URI, false, false)]
    #[case(DISCLOSURE_USECASE_ALL_REDIRECT_URI, true, true)]
    #[tokio::test]
    async fn test_verifier_new_session_redirect_uri_configuration_mismatch(
        #[case] usecase_id: &str,
        #[case] has_return_url: bool,
        #[case] should_succeed: bool,
    ) {
        let verifier = create_verifier();
        let return_url_template = has_return_url.then(|| "https://example.com/{session_token}".parse().unwrap());

        let result = verifier
            .new_session(new_disclosure_request(), usecase_id.to_string(), return_url_template)
            .await;

        if should_succeed {
            let _ = result.expect("creating a new session should succeed");
        } else {
            let error = result.expect_err("creating a new session should not succeed");
            assert_matches!(error, NewSessionError::ReturnUrlConfigurationMismatch);
        }
    }

    async fn init_and_start_disclosure(
        time: &impl Generator<DateTime<Utc>>,
    ) -> (
        Verifier<MemorySessionStore<DisclosureData>>,
        SessionToken,
        VpRequestUriObject,
    ) {
        let verifier = create_verifier();

        // Start session
        let session_token = verifier
            .new_session(
                new_disclosure_request(),
                DISCLOSURE_USECASE.to_string(),
                Some("https://example.com/{session_token}".parse().unwrap()),
            )
            .await
            .unwrap();

        // Invoke status endpoint to get the UL for the wallet from it
        let response = verifier
            .status_response(
                &session_token,
                Some(SessionType::SameDevice),
                &"https://app.example.com/app".parse().unwrap(),
                format!("https://example.com/disclosure/{session_token}")
                    .parse()
                    .unwrap(),
                time,
            )
            .await
            .expect("should result in status response for session");

        let StatusResponse::Created { ul } = response else {
            panic!("should match DisclosureData::Created")
        };

        let request_query_object: VpRequestUriObject =
            serde_urlencoded::from_str(ul.unwrap().as_ref().query().unwrap()).unwrap();

        (verifier, session_token, request_query_object)
    }

    #[tokio::test]
    async fn disclosure() {
        let (verifier, session_token, request_uri_object) = init_and_start_disclosure(&TimeGenerator).await;

        // Getting the Authorization Request should succeed
        verifier
            .process_get_request(
                &session_token,
                format!("https://example.com/disclosure/{session_token}/response_uri")
                    .parse()
                    .unwrap(),
                request_uri_object.request_uri.as_ref().query(),
                None,
            )
            .await
            .unwrap();

        // We have no mdoc in this test to actually disclose, so we let the wallet terminate the session
        let end_session_message = WalletAuthResponse::Error(ErrorResponse {
            error: VpAuthorizationErrorCode::AuthorizationError(AuthorizationErrorCode::AccessDenied),
            error_description: None,
            error_uri: None,
        });
        let ended_session_response = verifier
            .process_authorization_response(&session_token, end_session_message, &TimeGenerator)
            .await
            .unwrap();
        assert!(ended_session_response.redirect_uri.is_some());

        // Session state should show the session has been cancelled
        let DisclosureData::Done(session_state) = verifier.sessions.get(&session_token).await.unwrap().unwrap().data
        else {
            panic!("unexpected session state")
        };
        assert_matches!(session_state.session_result, SessionResult::Cancelled);
    }

    struct ExpiredEphemeralIdGenerator;

    impl Generator<DateTime<Utc>> for ExpiredEphemeralIdGenerator {
        fn generate(&self) -> DateTime<Utc> {
            Utc::now() - EPHEMERAL_ID_VALIDITY_SECONDS - Duration::seconds(1)
        }
    }

    #[tokio::test]
    async fn disclosure_expired_id() {
        let (verifier, session_token, request_uri_object) =
            init_and_start_disclosure(&ExpiredEphemeralIdGenerator).await;

        let error = verifier
            .process_get_request(
                &session_token,
                format!("https://example.com/disclosure/{session_token}/response_uri")
                    .parse()
                    .unwrap(),
                request_uri_object.request_uri.as_ref().query(),
                None,
            )
            .await
            .expect_err("should result in VerificationError::ExpiredEphemeralId");

        let ephemeral_id = request_uri_object
            .request_uri
            .as_ref()
            .query_pairs()
            .find_map(|(key, value)| (key == "ephemeral_id").then(|| hex::decode(value.as_bytes()).unwrap()))
            .unwrap();

        assert!(matches!(
            error.error,
            GetAuthRequestError::ExpiredEphemeralId(id) if id == ephemeral_id
        ));
    }

    #[tokio::test]
    async fn disclosure_invalid_id() {
        let (verifier, session_token, request_uri_object) = init_and_start_disclosure(&TimeGenerator).await;

        let invalid_ephemeral_id = b"\xde\xad\xbe\xef".to_vec();

        // set an invalid ephemeral id
        let mut request_uri = request_uri_object.request_uri.into_inner();
        let query = request_uri
            .query_pairs()
            .filter_map(|(key, value)| (key != "ephemeral_id").then(|| (key.into_owned(), value.into_owned())))
            .collect_vec();
        request_uri
            .query_pairs_mut()
            .clear()
            .extend_pairs(query)
            .append_pair("ephemeral_id", &hex::encode(&invalid_ephemeral_id));
        let request_uri_object = VpRequestUriObject {
            request_uri: request_uri.try_into().unwrap(),
            ..request_uri_object
        };

        let error = verifier
            .process_get_request(
                &session_token,
                format!("https://example.com/disclosure/{session_token}/response_uri")
                    .parse()
                    .unwrap(),
                request_uri_object.request_uri.as_ref().query(),
                None,
            )
            .await
            .expect_err("should result in VerificationError::InvalidEphemeralId(...)");

        assert!(matches!(
            error.error,
            GetAuthRequestError::InvalidEphemeralId(id) if id == invalid_ephemeral_id
        ));
    }

    #[tokio::test]
    async fn test_verifier_disclosed_attributes() {
        let verifier = create_verifier();

        // Add three sessions to the store:
        // * One with disclosed attributes and a return URL
        // * One with disclosed attributes and no return URL
        // * One expired session
        let session1 = SessionState::new(
            "token1".into(),
            DisclosureData::Done(Done {
                session_result: SessionResult::Done {
                    disclosed_attributes: Default::default(),
                    redirect_uri_nonce: None,
                },
            }),
        );
        let session2 = SessionState::new(
            "token2".into(),
            DisclosureData::Done(Done {
                session_result: SessionResult::Done {
                    disclosed_attributes: Default::default(),
                    redirect_uri_nonce: "this-is-the-nonce".to_string().into(),
                },
            }),
        );
        let session3 = SessionState::new(
            "token3".into(),
            DisclosureData::Done(Done {
                session_result: SessionResult::Expired,
            }),
        );

        verifier.sessions.write(session1, true).await.unwrap();
        verifier.sessions.write(session2, true).await.unwrap();
        verifier.sessions.write(session3, true).await.unwrap();

        // The finished session without a return URL should return the
        // attributes, regardless of the return URL nonce provided.
        assert!(verifier
            .disclosed_attributes(&"token1".into(), None)
            .await
            .expect("should return disclosed attributes")
            .is_empty());
        assert!(verifier
            .disclosed_attributes(&"token1".into(), "nonsense".to_string().into())
            .await
            .expect("should return disclosed attributes")
            .is_empty());

        // The finished session with a return URL should only return the
        // disclosed attributes when given the correct return URL nonce.
        assert!(verifier
            .disclosed_attributes(&"token2".into(), "this-is-the-nonce".to_string().into())
            .await
            .expect("should return disclosed attributes")
            .is_empty());
        assert_matches!(
            verifier
                .disclosed_attributes(&"token2".into(), "incorrect".to_string().into())
                .await
                .expect_err("should fail to return disclosed attributes"),
                DisclosedAttributesError::RedirectUriNonceMismatch(nonce) if nonce == "incorrect"
        );
        assert_matches!(
            verifier
                .disclosed_attributes(&"token2".into(), None)
                .await
                .expect_err("should fail to return disclosed attributes"),
            DisclosedAttributesError::RedirectUriNonceMissing
        );

        // The expired session should always return an error, with or without a nonce.
        assert_matches!(
            verifier
                .disclosed_attributes(&"token3".into(), None)
                .await
                .expect_err("should fail to return disclosed attributes"),
            DisclosedAttributesError::Session(SessionError::UnexpectedState(SessionStatus::Expired))
        );
        assert_matches!(
            verifier
                .disclosed_attributes(&"token3".into(), "nonsense".to_string().into())
                .await
                .expect_err("should fail to return disclosed attributes"),
            DisclosedAttributesError::Session(SessionError::UnexpectedState(SessionStatus::Expired))
        );
    }

    #[test]
    fn test_verifier_url() {
        let ephemeral_id_secret = hmac::Key::generate(hmac::HMAC_SHA256, &rand::SystemRandom::new()).unwrap();

        let session_token = "session_token".into();
        let time_str = "1969-07-21T02:56:15Z";
        let time = time_str.parse().unwrap();

        // Create a UL for the wallet, given the provided parameters.
        let verifier_url = Verifier::<()>::format_ul(
            &"https://app-ul.example.com".parse().unwrap(),
            "https://rp.example.com".parse().unwrap(),
            time,
            Verifier::<()>::generate_ephemeral_id(&ephemeral_id_secret, &session_token, &time),
            SessionType::CrossDevice,
            "client_id".to_string(),
        )
        .unwrap();

        // Format the ephemeral ID and sign it as a HMAC, then include it as hex in the URL we expect.
        let ephemeral_id = hmac::sign(
            &ephemeral_id_secret,
            (session_token.to_string() + "|" + time_str).as_bytes(),
        );
        let expected_url = format!(
            "https://app-ul.example.com/?request_uri=https%3A%2F%2Frp.example.com%2F%3Fsession_type%3Dcross_device\
            %26ephemeral_id%3D{}%26time%3D1969-07-21T02%253A56%253A15Z&request_uri_method=post&client_id=client_id",
            hex::encode(ephemeral_id)
        );

        assert_eq!(verifier_url.as_ref().as_str(), expected_url);
    }
}
