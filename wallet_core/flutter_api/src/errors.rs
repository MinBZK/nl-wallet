use std::error::Error;
use std::fmt::Display;

use anyhow::Chain;
use serde::Serialize;
use serde_with::skip_serializing_none;
use url::Url;

use utils::single_unique::NotUnique;
use utils::single_unique::SingleUniqueExt;
use wallet::errors::openid4vc::AuthorizationErrorCode;
use wallet::errors::openid4vc::IssuanceSessionError;
use wallet::errors::openid4vc::OidcError;
use wallet::errors::openid4vc::VpClientError;
use wallet::errors::openid4vc::VpMessageClientError;
use wallet::errors::openid4vc::VpMessageClientErrorType;
use wallet::errors::reqwest;
use wallet::errors::AccountProviderError;
use wallet::errors::ChangePinError;
use wallet::errors::DigidSessionError;
use wallet::errors::DisclosureError;
use wallet::errors::HistoryError;
use wallet::errors::HttpClientError;
use wallet::errors::InstructionError;
use wallet::errors::PidIssuanceError;
use wallet::errors::ResetError;
use wallet::errors::UpdatePolicyError;
use wallet::errors::UriIdentificationError;
use wallet::errors::WalletInitError;
use wallet::errors::WalletRegistrationError;
use wallet::errors::WalletUnlockError;
use wallet::mdoc::LocalizedStrings;
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
            .or_else(|e| e.downcast::<PidIssuanceError>().map(Self::from))
            .or_else(|e| e.downcast::<DisclosureError>().map(Self::from))
            .or_else(|e| e.downcast::<HistoryError>().map(Self::from))
            .or_else(|e| e.downcast::<ResetError>().map(Self::from))
            .or_else(|e| e.downcast::<url::ParseError>().map(Self::from))
            .or_else(|e| e.downcast::<ChangePinError>().map(Self::from))
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
            _ => FlutterApiErrorType::Generic,
        }
    }
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
}

impl FlutterApiErrorFields for PidIssuanceError {
    fn typ(&self) -> FlutterApiErrorType {
        if let Some(network_error) = detect_networking_error(self) {
            return network_error;
        }

        match self {
            PidIssuanceError::VersionBlocked => FlutterApiErrorType::VersionBlocked,
            PidIssuanceError::NotRegistered | PidIssuanceError::Locked | PidIssuanceError::SessionState => {
                FlutterApiErrorType::WalletState
            }
            PidIssuanceError::DigidSessionFinish(DigidSessionError::Oidc(OidcError::RedirectUriError(_))) => {
                FlutterApiErrorType::RedirectUri
            }
            PidIssuanceError::IssuanceSession(IssuanceSessionError::TokenRequest(_))
            | PidIssuanceError::IssuanceSession(IssuanceSessionError::CredentialRequest(_))
            | PidIssuanceError::DigidSessionStart(DigidSessionError::Oidc(OidcError::RedirectUriError(_)))
            | PidIssuanceError::DigidSessionStart(DigidSessionError::Oidc(OidcError::RequestingAccessToken(_)))
            | PidIssuanceError::DigidSessionStart(DigidSessionError::Oidc(OidcError::RequestingUserInfo(_)))
            | PidIssuanceError::DigidSessionFinish(DigidSessionError::Oidc(OidcError::RequestingAccessToken(_)))
            | PidIssuanceError::DigidSessionFinish(DigidSessionError::Oidc(OidcError::RequestingUserInfo(_))) => {
                FlutterApiErrorType::Server
            }
            PidIssuanceError::InvalidIssuerCertificate(_)
            | PidIssuanceError::MissingIssuerRegistration
            | PidIssuanceError::AttestationPreview(_)
            | PidIssuanceError::Attestation { .. }
            | PidIssuanceError::IssuerServer { .. } => FlutterApiErrorType::Issuer,
            PidIssuanceError::UpdatePolicy(e) => FlutterApiErrorType::from(e),
            _ => FlutterApiErrorType::Generic,
        }
    }

    fn data(&self) -> serde_json::Value {
        let redirect_error =
            if let Self::DigidSessionFinish(DigidSessionError::Oidc(OidcError::RedirectUriError(err))) = self {
                Some(err.error.clone())
            } else {
                None
            };

        let organization_name = match self {
            PidIssuanceError::Attestation { organization, .. } => Some(organization.display_name.clone()),
            PidIssuanceError::IssuerServer { organizations, .. } => match organizations
                .iter()
                .map(|organization| organization.display_name.clone())
                .single_unique()
            {
                Ok(Some(organization)) => Some(organization),
                Ok(None) => None,
                Err(NotUnique) => {
                    tracing::warn!("multiple issuer organizations detected in issuance session, returning first");
                    organizations.first().map(|o| o.display_name.clone())
                }
            },
            _ => None,
        };

        if redirect_error.is_some() || organization_name.is_some() {
            serde_json::to_value(IssuanceErrorData {
                redirect_error,
                organization_name,
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
            DisclosureError::VersionBlocked => FlutterApiErrorType::VersionBlocked,
            DisclosureError::NotRegistered | DisclosureError::Locked | DisclosureError::SessionState => {
                FlutterApiErrorType::WalletState
            }
            DisclosureError::VpClient(VpClientError::DisclosureUriSourceMismatch(_, _))
            | DisclosureError::VpVerifierServer {
                error: VpClientError::DisclosureUriSourceMismatch(_, _),
                ..
            } => FlutterApiErrorType::DisclosureSourceMismatch,
            DisclosureError::VpClient(VpClientError::Request(error)) => {
                type_for_vp_message_client(error).unwrap_or(FlutterApiErrorType::Generic)
            }
            DisclosureError::VpVerifierServer {
                error: VpClientError::Request(error),
                ..
            } => type_for_vp_message_client(error).unwrap_or(FlutterApiErrorType::Verifier),
            DisclosureError::VpClient(error) => detect_networking_error(error).unwrap_or(FlutterApiErrorType::Generic),
            DisclosureError::VpVerifierServer { error, .. } => {
                detect_networking_error(error).unwrap_or(FlutterApiErrorType::Verifier)
            }
            DisclosureError::Instruction(error) => FlutterApiErrorType::from(error),
            DisclosureError::UpdatePolicy(error) => FlutterApiErrorType::from(error),
            _ => FlutterApiErrorType::Generic,
        }
    }

    fn data(&self) -> serde_json::Value {
        let session_type = match self {
            DisclosureError::VpClient(VpClientError::DisclosureUriSourceMismatch(session_type, _))
            | DisclosureError::VpVerifierServer {
                error: VpClientError::DisclosureUriSourceMismatch(session_type, _),
                ..
            } => Some(*session_type),
            _ => None,
        };
        let can_retry = match self {
            DisclosureError::VpClient(VpClientError::Request(error))
            | DisclosureError::VpVerifierServer {
                error: VpClientError::Request(error),
                ..
            } => match error.error_type() {
                VpMessageClientErrorType::Expired { can_retry } => Some(can_retry),
                VpMessageClientErrorType::Cancelled | VpMessageClientErrorType::Other => None,
            },
            _ => None,
        };
        let return_url = self.return_url();
        let organization_name = match self {
            DisclosureError::VpVerifierServer { organization, .. } => Some(organization.display_name.clone()),
            _ => None,
        };

        if session_type.is_some() || can_retry.is_some() || return_url.is_some() {
            serde_json::to_value(DisclosureErrorData {
                session_type,
                can_retry,
                return_url,
                organization_name,
            })
            .unwrap() // This conversion should never fail.
        } else {
            serde_json::Value::Null
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
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use rstest::rstest;

    use serde_json::json;
    use wallet::errors::openid4vc::AuthorizationErrorCode;
    use wallet::errors::openid4vc::ErrorResponse;
    use wallet::errors::openid4vc::OidcError;
    use wallet::errors::DigidSessionError;
    use wallet::errors::PidIssuanceError;

    use super::FlutterApiError;
    use super::FlutterApiErrorType;

    // TODO: (PVW-4073) Add more error test cases.
    #[rstest]
    #[case(
        PidIssuanceError::VersionBlocked,
        FlutterApiErrorType::VersionBlocked,
        serde_json::Value::Null
    )]
    #[case(
        PidIssuanceError::NotRegistered,
        FlutterApiErrorType::WalletState,
        serde_json::Value::Null
    )]
    #[case(PidIssuanceError::Locked, FlutterApiErrorType::WalletState, serde_json::Value::Null)]
    #[case(
        PidIssuanceError::SessionState,
        FlutterApiErrorType::WalletState,
        serde_json::Value::Null
    )]
    #[case(
        PidIssuanceError::DigidSessionFinish(DigidSessionError::Oidc(OidcError::RedirectUriError(
            Box::new(ErrorResponse {
                error: AuthorizationErrorCode::InvalidRequest,
                error_description: None,
                error_uri: None,
            })
        ))),
        FlutterApiErrorType::RedirectUri,
        json!({"redirect_error": "invalid_request"})
    )]
    #[case(
        PidIssuanceError::DigidSessionFinish(DigidSessionError::Oidc(OidcError::RedirectUriError(
            Box::new(ErrorResponse {
                error: AuthorizationErrorCode::Other("some_error".to_string()),
                error_description: None,
                error_uri: None,
            })
        ))),
        FlutterApiErrorType::RedirectUri,
        json!({"redirect_error": "some_error"})
    )]
    #[case(
        PidIssuanceError::DigidSessionStart(DigidSessionError::Oidc(OidcError::RedirectUriError(Box::new(ErrorResponse {
            error: AuthorizationErrorCode::InvalidRequest,
            error_description: None,
            error_uri: None,
        })))),
        FlutterApiErrorType::Server,
        serde_json::Value::Null
    )]
    #[case(
        PidIssuanceError::MissingSignature,
        FlutterApiErrorType::Generic,
        serde_json::Value::Null
    )]
    fn test_pid_issuance_error<E>(
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
