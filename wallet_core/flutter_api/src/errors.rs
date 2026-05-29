use std::error::Error;
use std::fmt::Display;

use anyhow::Chain;
use serde::Serialize;
use serde_with::skip_serializing_none;
use url::Url;
use wallet::AccountRevokedData;
use wallet::attestation_data::LocalizedStrings;
use wallet::errors::AccountProviderError;
use wallet::errors::CancelSessionError;
use wallet::errors::ChangePinError;
use wallet::errors::CheckPreconditionsError;
use wallet::errors::CloseProximityDisclosureError;
use wallet::errors::DeleteAttestationError;
use wallet::errors::DisclosureBasedIssuanceError;
use wallet::errors::DisclosureError;
use wallet::errors::HistoryError;
use wallet::errors::HttpClientError;
use wallet::errors::InstructionError;
use wallet::errors::IssuanceError;
use wallet::errors::PinRecoveryError;
use wallet::errors::RecoveryCodeError;
use wallet::errors::ResetError;
use wallet::errors::RevocationCodeError;
use wallet::errors::TransferError;
use wallet::errors::UpdatePolicyError;
use wallet::errors::UriIdentificationError;
use wallet::errors::WalletInitError;
use wallet::errors::WalletRegistrationError;
use wallet::errors::WalletUnlockError;
use wallet::errors::openid4vc::AuthorizationErrorCode;
use wallet::errors::openid4vc::OAuthError;
use wallet::errors::openid4vc::VpClientError;
use wallet::errors::openid4vc::VpMessageClientError;
use wallet::errors::openid4vc::VpMessageClientErrorType;
use wallet::errors::openid4vc::VpVerifierError;
use wallet::errors::openid4vc::WalletIssuanceError;
use wallet::errors::reqwest;
use wallet::openid4vc::SessionType;

/// A type encapsulating data about a Flutter error that
/// is to be serialized to JSON and sent to Flutter.
#[derive(Debug, Serialize)]
pub struct FlutterApiError {
    #[serde(rename = "type")]
    typ: FlutterApiErrorType,
    description: String,
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    data: serde_json::Value,
    /// This property is present only for debug logging purposes and will not be encoded to JSON.
    #[serde(skip)]
    source: Box<dyn Error>,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
enum FlutterApiErrorType {
    /// This version of the app is blocked.
    VersionBlocked,

    /// A network connection has timed-out, was unable to connect or something else went wrong during the request.
    Networking,

    /// The request failed, but the server did send a response.
    Server,

    /// Something went wrong during issuance that's caused by the issuer.
    Issuer,

    /// Something went wrong during disclosure that's caused by the verifier.
    Verifier,

    /// The wallet is in an unexpected state.
    WalletState,

    /// Failed to finish the DigiD session and get an authorization code.
    RedirectUri,

    /// Device does not support hardware backed keys.
    HardwareKeyUnsupported,

    /// The disclosure URI source (universal link or QR code) does not match the received session type.
    DisclosureSourceMismatch,

    /// A remote session is expired, the user may or may not be able to retry the operation.
    ExpiredSession,

    /// A remote session is cancelled.
    CancelledSession,

    /// The wrong DigiD was used for PID renewal or PIN recovery.
    WrongDigid,

    /// DigiD authentication was cancelled.
    DeniedDigid,

    /// Wallet has been revoked.
    Revoked,

    /// Indicating something unexpected went wrong.
    Generic,
}

trait FlutterApiErrorFields {
    fn typ(&self) -> FlutterApiErrorType {
        FlutterApiErrorType::Generic
    }

    fn data(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

impl FlutterApiError {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Display for FlutterApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // This is effectively the same as forwarding the call to self.source,
        // since that is what we got the description from in the first place.
        write!(f, "{}", self.description)
    }
}

impl Error for FlutterApiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

/// Allow conversion from a [`anyhow::Error`] to a [`FlutterApiError`] through downcasting.
/// If the conversion fails, the original [`anyhow::Error`] is contained in the [`Result`].
impl TryFrom<anyhow::Error> for FlutterApiError {
    type Error = anyhow::Error;

    fn try_from(value: anyhow::Error) -> Result<Self, Self::Error> {
        value
            .downcast::<WalletInitError>()
            .map(Self::from)
            .or_else(|e| e.downcast::<WalletRegistrationError>().map(Self::from))
            .or_else(|e| e.downcast::<WalletUnlockError>().map(Self::from))
            .or_else(|e| e.downcast::<UriIdentificationError>().map(Self::from))
            .or_else(|e| e.downcast::<IssuanceError>().map(Self::from))
            .or_else(|e| e.downcast::<DisclosureError>().map(Self::from))
            .or_else(|e| e.downcast::<DisclosureBasedIssuanceError>().map(Self::from))
            .or_else(|e| e.downcast::<CancelSessionError>().map(Self::from))
            .or_else(|e| e.downcast::<HistoryError>().map(Self::from))
            .or_else(|e| e.downcast::<ResetError>().map(Self::from))
            .or_else(|e| e.downcast::<url::ParseError>().map(Self::from))
            .or_else(|e| e.downcast::<ChangePinError>().map(Self::from))
            .or_else(|e| e.downcast::<PinRecoveryError>().map(Self::from))
            .or_else(|e| e.downcast::<TransferError>().map(Self::from))
            .or_else(|e| e.downcast::<RevocationCodeError>().map(Self::from))
            .or_else(|e| e.downcast::<DeleteAttestationError>().map(Self::from))
    }
}

/// Allow conversion from any error for which a reference can be converted to a FlutterApiErrorType.
impl<E> From<E> for FlutterApiError
where
    E: Error + FlutterApiErrorFields + 'static,
{
    fn from(value: E) -> Self {
        FlutterApiError {
            typ: value.typ(),
            description: value.to_string(),
            data: value.data(),
            source: Box::new(value),
        }
    }
}

// The below traits will output the correct FlutterApiErrorType and data for a given
// error that can be returned from the Wallet. This can possibly be several layers deep.
impl FlutterApiErrorFields for WalletInitError {}

impl FlutterApiErrorFields for WalletRegistrationError {
    fn typ(&self) -> FlutterApiErrorType {
        if self.is_attestation_not_supported() {
            return FlutterApiErrorType::HardwareKeyUnsupported;
        }

        match self {
            WalletRegistrationError::VersionBlocked => FlutterApiErrorType::VersionBlocked,
            WalletRegistrationError::AlreadyRegistered => FlutterApiErrorType::WalletState,
            WalletRegistrationError::ChallengeRequest(e) => FlutterApiErrorType::from(e),
            WalletRegistrationError::RegistrationRequest(e) => FlutterApiErrorType::from(e),
            WalletRegistrationError::UpdatePolicy(e) => FlutterApiErrorType::from(e),
            WalletRegistrationError::AccountRevoked(_) => FlutterApiErrorType::Revoked,
            _ => FlutterApiErrorType::Generic,
        }
    }

    fn data(&self) -> serde_json::Value {
        match self {
            WalletRegistrationError::AccountRevoked(data) => {
                serde_json::to_value(RevocationErrorData { revocation_data: *data }).unwrap() // This conversion should never fail.
            }
            _ => serde_json::Value::Null,
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
struct RevocationErrorData {
    revocation_data: AccountRevokedData,
}

impl FlutterApiErrorFields for WalletUnlockError {
    fn typ(&self) -> FlutterApiErrorType {
        match self {
            WalletUnlockError::VersionBlocked => FlutterApiErrorType::VersionBlocked,
            WalletUnlockError::NotRegistered
            | WalletUnlockError::NotLocked
            | WalletUnlockError::Locked
            | WalletUnlockError::BiometricsUnlockingNotEnabled => FlutterApiErrorType::WalletState,
            WalletUnlockError::Instruction(e) => FlutterApiErrorType::from(e),
            WalletUnlockError::ChangePin(e) => e.typ(),
            WalletUnlockError::UpdatePolicy(e) => FlutterApiErrorType::from(e),
            WalletUnlockError::UnlockMethodStorage(_) => FlutterApiErrorType::Generic,
        }
    }

    fn data(&self) -> serde_json::Value {
        match self {
            WalletUnlockError::Instruction(InstructionError::AccountRevoked(data)) => {
                serde_json::to_value(RevocationErrorData { revocation_data: *data }).unwrap() // This conversion should never fail.
            }
            _ => serde_json::Value::Null,
        }
    }
}

impl FlutterApiErrorFields for UriIdentificationError {}

fn detect_networking_error(error: &(dyn Error + 'static)) -> Option<FlutterApiErrorType> {
    // Since a `reqwest::Error` can occur in multiple locations
    // within the error tree, just look for it with some help
    // from the `anyhow::Chain` iterator.
    for source in Chain::new(error) {
        if let Some(err) = source.downcast_ref::<reqwest::Error>() {
            return Some(FlutterApiErrorType::from(err));
        }
    }

    None
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
struct IssuanceErrorData {
    redirect_error: Option<AuthorizationErrorCode>,
    organization_name: Option<LocalizedStrings>,
    revocation_data: Option<AccountRevokedData>,
}

impl FlutterApiErrorFields for IssuanceError {
    fn typ(&self) -> FlutterApiErrorType {
        if let Some(network_error) = detect_networking_error(self) {
            return network_error;
        }

        match self {
            IssuanceError::CheckPreconditions(CheckPreconditionsError::VersionBlocked) => {
                FlutterApiErrorType::VersionBlocked
            }
            IssuanceError::CheckPreconditions(_) | IssuanceError::SessionState => FlutterApiErrorType::WalletState,
            IssuanceError::IssuanceSession(WalletIssuanceError::OAuth(OAuthError::RedirectUriError(_))) => {
                FlutterApiErrorType::RedirectUri
            }
            IssuanceError::IssuanceSession(WalletIssuanceError::TokenRequest(_))
            | IssuanceError::IssuanceSession(WalletIssuanceError::CredentialPreview(_))
            | IssuanceError::IssuanceSession(WalletIssuanceError::CredentialRequest(_))
            | IssuanceError::IssuanceSession(WalletIssuanceError::CredentialRejection(_)) => {
                FlutterApiErrorType::Server
            }
            IssuanceError::AttestationPreview(_)
            | IssuanceError::Attestation { .. }
            | IssuanceError::IssuerServer { .. } => FlutterApiErrorType::Issuer,
            IssuanceError::DeniedDigiD => FlutterApiErrorType::DeniedDigid,
            IssuanceError::RecoveryCode(RecoveryCodeError::IncorrectRecoveryCode { .. }) => {
                FlutterApiErrorType::WrongDigid
            }
            IssuanceError::Instruction(error) => FlutterApiErrorType::from(error),
            IssuanceError::PidAlreadyPresent
            | IssuanceError::NoPidPresent
            | IssuanceError::IssuerMetadataDiscovery(_)
            | IssuanceError::Signature(_)
            | IssuanceError::MissingSignature
            | IssuanceError::AttestationStorage(_)
            | IssuanceError::AttestationQuery(_)
            | IssuanceError::KeyNotFound(_)
            | IssuanceError::Attestations(_)
            | IssuanceError::Notifications(_)
            | IssuanceError::Events(_)
            | IssuanceError::ChangePin(_)
            | IssuanceError::JwtCredential(_)
            | IssuanceError::Certificate(_)
            | IssuanceError::MissingPidSdJwt
            | IssuanceError::RecoveryCodeDisclosure(_)
            | IssuanceError::TransferDataStorage(_)
            | IssuanceError::IssuanceSession(_)
            | IssuanceError::RecoveryCode(_) => FlutterApiErrorType::Generic,
        }
    }

    fn data(&self) -> serde_json::Value {
        let redirect_error =
            if let Self::IssuanceSession(WalletIssuanceError::OAuth(OAuthError::RedirectUriError(err))) = self {
                Some(err.error.clone())
            } else {
                None
            };

        let organization_name = match self {
            IssuanceError::Attestation { organization, .. } | IssuanceError::IssuerServer { organization, .. } => {
                Some(organization.display_name.clone())
            }
            _ => None,
        };

        let revocation_data = if let Self::Instruction(InstructionError::AccountRevoked(data)) = self {
            Some(*data)
        } else {
            None
        };

        if redirect_error.is_some() || organization_name.is_some() || revocation_data.is_some() {
            serde_json::to_value(IssuanceErrorData {
                redirect_error,
                organization_name,
                revocation_data,
            })
            .unwrap() // This conversion should never fail.
        } else {
            serde_json::Value::Null
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Serialize)]
struct DisclosureErrorData<'a> {
    session_type: Option<SessionType>,
    can_retry: Option<bool>,
    return_url: Option<&'a Url>,
    organization_name: Option<LocalizedStrings>,
    revocation_data: Option<AccountRevokedData>,
}

fn type_for_vp_message_client(error: &VpMessageClientError) -> Option<FlutterApiErrorType> {
    match error.error_type() {
        VpMessageClientErrorType::Expired { .. } => Some(FlutterApiErrorType::ExpiredSession),
        VpMessageClientErrorType::Cancelled => Some(FlutterApiErrorType::CancelledSession),
        _ => detect_networking_error(error),
    }
}

impl FlutterApiErrorFields for DisclosureError {
    fn typ(&self) -> FlutterApiErrorType {
        match self {
            DisclosureError::CheckPreconditions(CheckPreconditionsError::VersionBlocked) => {
                FlutterApiErrorType::VersionBlocked
            }
            DisclosureError::CheckPreconditions(_) | DisclosureError::SessionState => FlutterApiErrorType::WalletState,
            DisclosureError::VpClient(VpClientError::DisclosureUriSourceMismatch(_, _)) => {
                FlutterApiErrorType::DisclosureSourceMismatch
            }
            DisclosureError::VpClient(VpClientError::Request(error)) => {
                type_for_vp_message_client(error).unwrap_or(FlutterApiErrorType::Generic)
            }
            DisclosureError::VpVerifierServer {
                error: VpVerifierError::Request(error),
                ..
            } => type_for_vp_message_client(error).unwrap_or(FlutterApiErrorType::Verifier),
            DisclosureError::VpClient(error) => detect_networking_error(error).unwrap_or(FlutterApiErrorType::Generic),
            DisclosureError::VpVerifierServer { error, .. } => {
                detect_networking_error(error).unwrap_or(FlutterApiErrorType::Verifier)
            }
            DisclosureError::Instruction(error) => FlutterApiErrorType::from(error),
            DisclosureError::NonSelectivelyDisclosableClaim(_, _)
            | DisclosureError::NonSelectivelyDisclosableClaimsNotRequested(_, _, _)
            | DisclosureError::DisclosureUriQuery(_)
            | DisclosureError::RecoveryCodeRequested { .. }
            | DisclosureError::UnexpectedRedirectUriPurpose { .. } => FlutterApiErrorType::Verifier,
            DisclosureError::DisclosureUri(_)
            | DisclosureError::HistoryRetrieval(_)
            | DisclosureError::AttestationRetrieval(_)
            | DisclosureError::AttributesNotAvailable(_)
            | DisclosureError::IncrementUsageCount(_)
            | DisclosureError::EventStorage(_)
            | DisclosureError::ChangePin(_)
            | DisclosureError::PlatformCloseProximityDisclosureSessionError(_) => FlutterApiErrorType::Generic,
            DisclosureError::CloseProximityDisclosureSessionError(error) => error.into(),
        }
    }

    fn data(&self) -> serde_json::Value {
        let can_retry = match self {
            DisclosureError::VpClient(VpClientError::Request(error))
            | DisclosureError::VpVerifierServer {
                error: VpVerifierError::Request(error),
                ..
            } => match error.error_type() {
                VpMessageClientErrorType::Expired { can_retry } => Some(can_retry),
                VpMessageClientErrorType::Cancelled | VpMessageClientErrorType::Other => None,
            },
            DisclosureError::NonSelectivelyDisclosableClaim(_, _)
            | DisclosureError::NonSelectivelyDisclosableClaimsNotRequested(_, _, _) => Some(false),
            DisclosureError::CloseProximityDisclosureSessionError(inner) => match inner {
                // Platform errors are transient and worth retrying.
                CloseProximityDisclosureError::PlatformError(_) => Some(true),
                CloseProximityDisclosureError::DeviceResponse(error) => detect_networking_error(error).map(|_| true),
                CloseProximityDisclosureError::DeviceResponseEncoding(_) => None,
                _ => Some(false),
            },
            _ => None,
        };
        let organization_name = match self {
            DisclosureError::VpVerifierServer { organization, .. } => organization
                .as_ref()
                .map(|organization| organization.display_name.clone()),
            DisclosureError::NonSelectivelyDisclosableClaim(organization, _)
            | DisclosureError::NonSelectivelyDisclosableClaimsNotRequested(organization, _, _)
            | DisclosureError::RecoveryCodeRequested(organization) => Some(organization.display_name.clone()),
            DisclosureError::CloseProximityDisclosureSessionError(
                CloseProximityDisclosureError::ReaderAuthValidation { organization, .. }
                | CloseProximityDisclosureError::InvalidCertificate { organization, .. }
                | CloseProximityDisclosureError::MissingCommonName { organization },
            ) => Some(organization.display_name.clone()),
            _ => None,
        };

        let session_type = self.session_type();
        let return_url = self.return_url();
        let revocation_data = self.revocation_data();

        if session_type.is_some()
            || can_retry.is_some()
            || return_url.is_some()
            || organization_name.is_some()
            || revocation_data.is_some()
        {
            serde_json::to_value(DisclosureErrorData {
                session_type,
                can_retry,
                return_url,
                organization_name,
                revocation_data,
            })
            .unwrap() // This conversion should never fail.
        } else {
            serde_json::Value::Null
        }
    }
}

impl FlutterApiErrorFields for CloseProximityDisclosureError {
    fn typ(&self) -> FlutterApiErrorType {
        self.into()
    }

    fn data(&self) -> serde_json::Value {
        let organization_name = match self {
            CloseProximityDisclosureError::ReaderAuthValidation { organization, .. }
            | CloseProximityDisclosureError::InvalidCertificate { organization, .. }
            | CloseProximityDisclosureError::MissingCommonName { organization } => {
                Some(organization.display_name.clone())
            }
            _ => None,
        };

        let can_retry = match self {
            // Platform errors are transient and worth retrying.
            CloseProximityDisclosureError::PlatformError(_) => Some(true),
            CloseProximityDisclosureError::DeviceResponse(error) => detect_networking_error(error).map(|_| true),
            CloseProximityDisclosureError::DeviceResponseEncoding(_) => None,
            _ => Some(false),
        };

        // All close proximity disclosure sessions are cross-device, `return_url` is not
        // applicable to close proximity disclosure, and `revocation_data` always surfaces as
        // `DisclosureError::Instruction`.
        serde_json::to_value(DisclosureErrorData {
            session_type: Some(SessionType::CrossDevice),
            can_retry,
            return_url: None,
            organization_name,
            revocation_data: None,
        })
        .unwrap() // This conversion should never fail.
    }
}

#[derive(Debug, Clone, Serialize)]
struct DisclosureBasedIssuanceErrorData {
    organization_name: LocalizedStrings,
}

impl FlutterApiErrorFields for DisclosureBasedIssuanceError {
    fn typ(&self) -> FlutterApiErrorType {
        match self {
            Self::CheckPreconditions(error) => error.typ(),
            Self::Disclosure(error) => error.typ(),
            Self::Issuance(error) => error.typ(),
            Self::NoPreAuthorizedCode(_) | Self::MissingRedirectUri(_) | Self::UnexpectedScheme(_, _) => {
                FlutterApiErrorType::Issuer
            }
        }
    }

    fn data(&self) -> serde_json::Value {
        match self {
            Self::CheckPreconditions(error) => error.data(),
            Self::Disclosure(error) => error.data(),
            Self::Issuance(error) => error.data(),
            Self::NoPreAuthorizedCode(organization)
            | Self::MissingRedirectUri(organization)
            | Self::UnexpectedScheme(_, organization) => serde_json::to_value(DisclosureBasedIssuanceErrorData {
                organization_name: organization.display_name.clone(),
            })
            .unwrap(),
        }
    }
}

impl FlutterApiErrorFields for CheckPreconditionsError {
    fn typ(&self) -> FlutterApiErrorType {
        match self {
            CheckPreconditionsError::VersionBlocked => FlutterApiErrorType::VersionBlocked,
            CheckPreconditionsError::NotRegistered | CheckPreconditionsError::Locked => {
                FlutterApiErrorType::WalletState
            }
            CheckPreconditionsError::UpdatePolicy(error) => error.into(),
        }
    }
}

impl FlutterApiErrorFields for CancelSessionError {
    fn typ(&self) -> FlutterApiErrorType {
        match self {
            CancelSessionError::Preconditions(error) => error.typ(),
            CancelSessionError::Issuance(error) => error.typ(),
            CancelSessionError::Disclosure(error) => error.typ(),
            CancelSessionError::SessionState => FlutterApiErrorType::WalletState,
        }
    }

    fn data(&self) -> serde_json::Value {
        match self {
            CancelSessionError::Issuance(error) => error.data(),
            CancelSessionError::Disclosure(error) => error.data(),
            CancelSessionError::SessionState | CancelSessionError::Preconditions(_) => serde_json::Value::Null,
        }
    }
}

impl FlutterApiErrorFields for url::ParseError {
    fn typ(&self) -> FlutterApiErrorType {
        FlutterApiErrorType::WalletState
    }
}

impl From<&reqwest::Error> for FlutterApiErrorType {
    fn from(value: &reqwest::Error) -> Self {
        match () {
            _ if value.is_timeout() || value.is_request() || value.is_connect() => FlutterApiErrorType::Networking,
            _ if value.is_status() => FlutterApiErrorType::Server,
            _ => FlutterApiErrorType::Generic,
        }
    }
}

impl From<&AccountProviderError> for FlutterApiErrorType {
    fn from(value: &AccountProviderError) -> Self {
        match value {
            AccountProviderError::Response(_) => FlutterApiErrorType::Server,
            AccountProviderError::Networking(e) => FlutterApiErrorType::from(e),
            _ => FlutterApiErrorType::Generic,
        }
    }
}

impl From<&InstructionError> for FlutterApiErrorType {
    fn from(value: &InstructionError) -> Self {
        match value {
            InstructionError::AccountRevoked(_) => FlutterApiErrorType::Revoked,
            InstructionError::ServerError(e) => FlutterApiErrorType::from(e),
            InstructionError::InstructionValidation => FlutterApiErrorType::Server,
            _ => FlutterApiErrorType::Generic,
        }
    }
}

impl From<&UpdatePolicyError> for FlutterApiErrorType {
    fn from(value: &UpdatePolicyError) -> Self {
        match value {
            UpdatePolicyError::HttpClient(e) => FlutterApiErrorType::from(e),
            _ => FlutterApiErrorType::Generic,
        }
    }
}

impl From<&HttpClientError> for FlutterApiErrorType {
    fn from(value: &HttpClientError) -> Self {
        match value {
            HttpClientError::Parse(_) | HttpClientError::EmptyBody | HttpClientError::Response(_, _) => {
                FlutterApiErrorType::Server
            }
            HttpClientError::Networking(_) => FlutterApiErrorType::Networking,
            _ => FlutterApiErrorType::Generic,
        }
    }
}

impl From<&CloseProximityDisclosureError> for FlutterApiErrorType {
    fn from(value: &CloseProximityDisclosureError) -> Self {
        match value {
            CloseProximityDisclosureError::MissingReaderAuth
            | CloseProximityDisclosureError::InconsistentReaderAuths
            | CloseProximityDisclosureError::InvalidDocRequest(_)
            | CloseProximityDisclosureError::MissingReaderRegistration
            | CloseProximityDisclosureError::InvalidCertificateType(_)
            | CloseProximityDisclosureError::ReaderAuthValidation { .. }
            | CloseProximityDisclosureError::MalformedDeviceRequest(_)
            | CloseProximityDisclosureError::InvalidDeviceRequest(_)
            | CloseProximityDisclosureError::InvalidCertificate { .. }
            | CloseProximityDisclosureError::MissingCommonName { .. } => FlutterApiErrorType::Verifier,
            CloseProximityDisclosureError::DeviceResponseEncoding(_)
            | CloseProximityDisclosureError::PlatformError(_)
            | CloseProximityDisclosureError::DeviceResponse(_) => FlutterApiErrorType::Generic,
        }
    }
}

impl FlutterApiErrorFields for HistoryError {
    fn typ(&self) -> FlutterApiErrorType {
        match self {
            HistoryError::VersionBlocked => FlutterApiErrorType::VersionBlocked,
            HistoryError::NotRegistered | HistoryError::Locked => FlutterApiErrorType::WalletState,
            _ => FlutterApiErrorType::Generic,
        }
    }
}

impl FlutterApiErrorFields for ResetError {
    fn typ(&self) -> FlutterApiErrorType {
        match self {
            ResetError::VersionBlocked => FlutterApiErrorType::VersionBlocked,
            ResetError::NotRegistered => FlutterApiErrorType::WalletState,
        }
    }
}

impl FlutterApiErrorFields for ChangePinError {
    fn typ(&self) -> FlutterApiErrorType {
        match self {
            Self::VersionBlocked => FlutterApiErrorType::VersionBlocked,
            Self::NotRegistered | Self::Locked | Self::ChangePinAlreadyInProgress | Self::NoChangePinInProgress => {
                FlutterApiErrorType::WalletState
            }
            Self::Instruction(e) => FlutterApiErrorType::from(e),
            Self::UpdatePolicy(e) => FlutterApiErrorType::from(e),
            Self::Storage(_)
            | Self::PinValidation(_)
            | Self::CertificateValidation(_)
            | Self::PublicKeyMismatch
            | Self::WalletIdMismatch => FlutterApiErrorType::Generic,
        }
    }

    fn data(&self) -> serde_json::Value {
        match self {
            ChangePinError::Instruction(InstructionError::AccountRevoked(data)) => {
                serde_json::to_value(RevocationErrorData { revocation_data: *data }).unwrap() // This conversion should never fail.
            }
            _ => serde_json::Value::Null,
        }
    }
}

impl FlutterApiErrorFields for PinRecoveryError {
    fn typ(&self) -> FlutterApiErrorType {
        if let Some(network_error) = detect_networking_error(self) {
            return network_error;
        }

        match self {
            PinRecoveryError::VersionBlocked => FlutterApiErrorType::VersionBlocked,
            PinRecoveryError::NotRegistered | PinRecoveryError::SessionState => FlutterApiErrorType::WalletState,
            PinRecoveryError::Issuance(issuance_error) => issuance_error.typ(),
            PinRecoveryError::DeniedDigiD => FlutterApiErrorType::DeniedDigid,
            PinRecoveryError::RecoveryCode(RecoveryCodeError::IncorrectRecoveryCode { .. }) => {
                FlutterApiErrorType::WrongDigid
            }
            PinRecoveryError::RecoveryCode(RecoveryCodeError::MissingPid)
            | PinRecoveryError::RecoveryCode(RecoveryCodeError::MissingRecoveryCode) => FlutterApiErrorType::Issuer,
            PinRecoveryError::DiscloseRecoveryCode(InstructionError::AccountRevoked(_)) => FlutterApiErrorType::Revoked,
            PinRecoveryError::DiscloseRecoveryCode(..) => FlutterApiErrorType::Server,
            _ => FlutterApiErrorType::Generic,
        }
    }

    fn data(&self) -> serde_json::Value {
        match self {
            Self::Issuance(issuance_error) => issuance_error.data(),
            Self::DiscloseRecoveryCode(InstructionError::AccountRevoked(data)) => {
                serde_json::to_value(RevocationErrorData { revocation_data: *data }).unwrap() // This conversion should never fail.
            }
            _ => serde_json::Value::Null,
        }
    }
}

impl FlutterApiErrorFields for TransferError {
    fn typ(&self) -> FlutterApiErrorType {
        if let Some(network_error) = detect_networking_error(self) {
            return network_error;
        }

        match self {
            TransferError::VersionBlocked => FlutterApiErrorType::VersionBlocked,
            TransferError::NotRegistered | TransferError::IllegalWalletState => FlutterApiErrorType::WalletState,
            TransferError::Instruction(e) => FlutterApiErrorType::from(e),
            TransferError::UpdatePolicy(e) => FlutterApiErrorType::from(e),
            TransferError::ChangePin(e) => e.typ(),
            _ => FlutterApiErrorType::Generic,
        }
    }

    fn data(&self) -> serde_json::Value {
        match self {
            TransferError::Instruction(InstructionError::AccountRevoked(data)) => {
                serde_json::to_value(RevocationErrorData { revocation_data: *data }).unwrap() // This conversion should never fail.
            }
            _ => serde_json::Value::Null,
        }
    }
}

impl FlutterApiErrorFields for RevocationCodeError {
    fn typ(&self) -> FlutterApiErrorType {
        match self {
            Self::VersionBlocked => FlutterApiErrorType::VersionBlocked,
            Self::NotRegistered | Self::PidPresent => FlutterApiErrorType::WalletState,
            Self::PidRetrieval(_) => FlutterApiErrorType::Generic,
            Self::Unlock(error) => error.typ(),
        }
    }

    fn data(&self) -> serde_json::Value {
        match self {
            RevocationCodeError::Unlock(WalletUnlockError::Instruction(InstructionError::AccountRevoked(data))) => {
                serde_json::to_value(RevocationErrorData { revocation_data: *data }).unwrap() // This conversion should never fail.
            }
            _ => serde_json::Value::Null,
        }
    }
}

impl FlutterApiErrorFields for DeleteAttestationError {
    fn typ(&self) -> FlutterApiErrorType {
        match self {
            Self::VersionBlocked => FlutterApiErrorType::VersionBlocked,
            Self::NotRegistered | Self::Locked => FlutterApiErrorType::WalletState,
            Self::Instruction(e) => FlutterApiErrorType::from(e),
            Self::UpdatePolicy(e) => FlutterApiErrorType::from(e),
            Self::ChangePin(e) => e.typ(),
            _ => FlutterApiErrorType::Generic,
        }
    }

    fn data(&self) -> serde_json::Value {
        match self {
            Self::Instruction(InstructionError::AccountRevoked(data)) => {
                serde_json::to_value(RevocationErrorData { revocation_data: *data }).unwrap() // This conversion should never fail.
            }
            _ => serde_json::Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use rstest::rstest;
    use serde_json::json;
    use wallet::AccountRevokedData;
    use wallet::RevocationReason;
    use wallet::attestation_data::AttributeValue;
    use wallet::errors::CancelSessionError;
    use wallet::errors::ChangePinError;
    use wallet::errors::CheckPreconditionsError;
    use wallet::errors::DeleteAttestationError;
    use wallet::errors::DisclosureError;
    use wallet::errors::InstructionError;
    use wallet::errors::IssuanceError;
    use wallet::errors::PinRecoveryError;
    use wallet::errors::RecoveryCodeError;
    use wallet::errors::RevocationCodeError;
    use wallet::errors::TransferError;
    use wallet::errors::WalletUnlockError;
    use wallet::errors::openid4vc::AuthorizationErrorCode;
    use wallet::errors::openid4vc::DisclosureErrorResponse;
    use wallet::errors::openid4vc::ErrorResponse;
    use wallet::errors::openid4vc::OAuthError;
    use wallet::errors::openid4vc::PostAuthResponseErrorCode;
    use wallet::errors::openid4vc::VpClientError;
    use wallet::errors::openid4vc::VpMessageClientError;
    use wallet::errors::openid4vc::WalletIssuanceError;

    use super::FlutterApiError;
    use super::FlutterApiErrorType;

    // TODO: (PVW-4073) Add more error test cases.
    #[rstest]
    #[case::issuance_checkpreconditions_versionblocked(
        IssuanceError::CheckPreconditions(CheckPreconditionsError::VersionBlocked),
        FlutterApiErrorType::VersionBlocked,
        serde_json::Value::Null
    )]
    #[case::issuance_checkpreconditions_notregistered(
        IssuanceError::CheckPreconditions(CheckPreconditionsError::NotRegistered),
        FlutterApiErrorType::WalletState,
        serde_json::Value::Null
    )]
    #[case::issuance_checkpreconditions_locked(
        IssuanceError::CheckPreconditions(CheckPreconditionsError::Locked),
        FlutterApiErrorType::WalletState,
        serde_json::Value::Null
    )]
    #[case::issuance_sessionstate(
        IssuanceError::SessionState,
        FlutterApiErrorType::WalletState,
        serde_json::Value::Null
    )]
    #[case::issuance_oauth_session(
        IssuanceError::IssuanceSession(WalletIssuanceError::OAuth(OAuthError::RedirectUriError(
            Box::new(ErrorResponse {
                error: AuthorizationErrorCode::InvalidRequest,
                error_description: None,
                error_uri: None,
            })
        ))),
        FlutterApiErrorType::RedirectUri,
        json!({"redirect_error": "invalid_request"})
    )]
    #[case::issuance_oauth_session_other(
        IssuanceError::IssuanceSession(WalletIssuanceError::OAuth(OAuthError::RedirectUriError(
            Box::new(ErrorResponse {
                error: AuthorizationErrorCode::Other("some_error".to_string()),
                error_description: None,
                error_uri: None,
            })
        ))),
        FlutterApiErrorType::RedirectUri,
        json!({"redirect_error": "some_error"})
    )]
    #[case::issuance_missingsignature(
        IssuanceError::MissingSignature,
        FlutterApiErrorType::Generic,
        serde_json::Value::Null
    )]
    #[case::issuance::recoverycode(
        IssuanceError::RecoveryCode(RecoveryCodeError::IncorrectRecoveryCode {
            expected: AttributeValue::Text("a".to_string()),
            received: AttributeValue::Text("b".to_string())
        }),
        FlutterApiErrorType::WrongDigid,
        serde_json::Value::Null
    )]
    #[case::issuance_instruction(
        IssuanceError::Instruction(InstructionError::AccountRevoked(AccountRevokedData {
            revocation_reason: RevocationReason::UserRequest,
            can_register_new_account: true
        })),
        FlutterApiErrorType::Revoked,
        json!({
            "revocation_data": {
                "revocation_reason": "user_request",
                "can_register_new_account": true
            }
        })
    )]
    #[case::unlock_instruction(
        WalletUnlockError::Instruction(InstructionError::AccountRevoked(AccountRevokedData {
            revocation_reason: RevocationReason::UserRequest,
            can_register_new_account: true
        })),
        FlutterApiErrorType::Revoked,
        json!({
            "revocation_data": {
                "revocation_reason": "user_request",
                "can_register_new_account": true
            }
        })
    )]
    #[case::disclosure_instruction(
        DisclosureError::Instruction(InstructionError::AccountRevoked(AccountRevokedData {
            revocation_reason: RevocationReason::UserRequest,
            can_register_new_account: true
        })),
        FlutterApiErrorType::Revoked,
        json!({
            "revocation_data": {
                "revocation_reason": "user_request",
                "can_register_new_account": true
            }
        })
    )]
    #[case::disclosure_returnurl(
        DisclosureError::VpClient(VpClientError::Request(VpMessageClientError::AuthPostResponse(
            Box::new(
                DisclosureErrorResponse {
                    error_response: ErrorResponse {
                        error: PostAuthResponseErrorCode::CancelledSession,
                        error_description: None,
                        error_uri: None
                    },
                    redirect_uri: Some("http://example.com/redirect_uri".parse().unwrap())
                }
            )
        ))),
        FlutterApiErrorType::CancelledSession,
        json!({ "return_url": "http://example.com/redirect_uri" })
    )]
    #[case::cancel_versionblocked(
        CancelSessionError::Preconditions(CheckPreconditionsError::VersionBlocked),
        FlutterApiErrorType::VersionBlocked,
        serde_json::Value::Null
    )]
    #[case::cancel_disclosure(
        CancelSessionError::Disclosure(
            DisclosureError::VpClient(VpClientError::Request(VpMessageClientError::AuthPostResponse(
                Box::new(
                    DisclosureErrorResponse {
                        error_response: ErrorResponse {
                            error: PostAuthResponseErrorCode::CancelledSession,
                            error_description: None,
                            error_uri: None
                        },
                        redirect_uri: Some("http://example.com/redirect_uri".parse().unwrap())
                    }
                )
            )))
        ),
        FlutterApiErrorType::CancelledSession,
        json!({ "return_url": "http://example.com/redirect_uri" })
    )]
    #[case::cancel_issuance(
        CancelSessionError::Issuance(
            IssuanceError::IssuanceSession(WalletIssuanceError::OAuth(OAuthError::RedirectUriError(
                Box::new(ErrorResponse {
                    error: AuthorizationErrorCode::Other("some_error".to_string()),
                    error_description: None,
                    error_uri: None,
                })
            )))
        ),
        FlutterApiErrorType::RedirectUri,
        json!({ "redirect_error": "some_error" })
    )]
    #[case::cancel_state(
        CancelSessionError::SessionState,
        FlutterApiErrorType::WalletState,
        serde_json::Value::Null
    )]
    #[case::changepin_instruction(
        ChangePinError::Instruction(InstructionError::AccountRevoked(AccountRevokedData {
            revocation_reason: RevocationReason::UserRequest,
            can_register_new_account: true
        })),
        FlutterApiErrorType::Revoked,
        json!({
            "revocation_data": {
                "revocation_reason": "user_request",
                "can_register_new_account": true
            }
        })
    )]
    #[case::pinrecovery_discloserecoverycode(
        PinRecoveryError::DiscloseRecoveryCode(InstructionError::AccountRevoked(AccountRevokedData {
            revocation_reason: RevocationReason::UserRequest,
            can_register_new_account: true
        })),
        FlutterApiErrorType::Revoked,
        json!({
            "revocation_data": {
                "revocation_reason": "user_request",
                "can_register_new_account": true
            }
        })
    )]
    #[case::pinrecovery_issuance(
        PinRecoveryError::Issuance(IssuanceError::Instruction(InstructionError::AccountRevoked(AccountRevokedData {
            revocation_reason: RevocationReason::UserRequest,
            can_register_new_account: true
        }))),
        FlutterApiErrorType::Revoked,
        json!({
            "revocation_data": {
                "revocation_reason": "user_request",
                "can_register_new_account": true
            }
        })
    )]
    #[case::transfer_instruction(
        TransferError::Instruction(InstructionError::AccountRevoked(AccountRevokedData {
            revocation_reason: RevocationReason::UserRequest,
            can_register_new_account: true
        })),
        FlutterApiErrorType::Revoked,
        json!({
            "revocation_data": {
                "revocation_reason": "user_request",
                "can_register_new_account": true
            }
        })
    )]
    #[case::revocation_unlock(
        RevocationCodeError::Unlock(WalletUnlockError::Instruction(InstructionError::AccountRevoked(AccountRevokedData {
            revocation_reason: RevocationReason::UserRequest,
            can_register_new_account: true
        }))),
        FlutterApiErrorType::Revoked,
        json!({
            "revocation_data": {
                "revocation_reason": "user_request",
                "can_register_new_account": true
            }
        })
    )]
    #[case::deleteattestation_instruction(
        DeleteAttestationError::Instruction(InstructionError::AccountRevoked(AccountRevokedData {
            revocation_reason: RevocationReason::UserRequest,
            can_register_new_account: true
        })),
        FlutterApiErrorType::Revoked,
        json!({
            "revocation_data": {
                "revocation_reason": "user_request",
                "can_register_new_account": true
            }
        })
    )]
    fn test_errors<E>(
        #[case] source_error: E,
        #[case] expected_type: FlutterApiErrorType,
        #[case] expected_data: serde_json::Value,
    ) where
        E: Error + Send + Sync + 'static,
    {
        let anyhow_error = anyhow::Error::new(source_error);
        let flutter_api_error =
            FlutterApiError::try_from(anyhow_error).expect("error should convert to FlutterApiError successfully");

        assert_eq!(flutter_api_error.typ, expected_type);
        assert_eq!(flutter_api_error.data, expected_data);
        assert!(flutter_api_error.source().unwrap().is::<E>());
    }
}
