use std::{error::Error, fmt::Display};

use anyhow::Chain;
use serde::Serialize;
use serde_with::skip_serializing_none;
use url::Url;

use wallet::{
    errors::{
        mdoc::{self, HolderError},
        openid4vc::{IssuanceSessionError, OidcError, VpClientError},
        reqwest, AccountProviderError, DigidSessionError, DisclosureError, HistoryError, InstructionError,
        PidIssuanceError, ResetError, UriIdentificationError, WalletInitError, WalletRegistrationError,
        WalletUnlockError,
    },
    mdoc::SessionType,
};

/// A type encapsulating data about a Flutter error that
/// is to be serialized to JSON and sent to Flutter.
#[derive(Debug, Serialize)]
pub struct FlutterApiError {
    #[serde(rename = "type")]
    typ: FlutterApiErrorType,
    description: String,
    data: Option<serde_json::Value>,
    /// This property is present only for debug logging purposes and will not be encoded to JSON.
    #[serde(skip)]
    source: Box<dyn Error>,
}

#[derive(Debug, Serialize)]
enum FlutterApiErrorType {
    /// A network connection has timed-out, was unable to connect or something else went wrong during the request.
    Networking,

    /// The request failed, but the server did send a response.
    Server,

    /// The wallet is in an unexpected state.
    WalletState,

    /// Failed to finish the DigiD session and get an authorization code.
    RedirectUri,

    /// Device does not support hardware backed keys.
    HardwareKeyUnsupported,

    /// The disclosure URI source (universal link or QR code) does not match the received session type.
    DisclosureSourceMismatch,

    /// Indicating something unexpected went wrong.
    Generic,
}

trait FlutterApiErrorFields {
    fn typ(&self) -> FlutterApiErrorType {
        FlutterApiErrorType::Generic
    }

    fn data(&self) -> Option<serde_json::Value> {
        None
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
        match self {
            WalletRegistrationError::AlreadyRegistered => FlutterApiErrorType::WalletState,
            WalletRegistrationError::HardwarePublicKey(_) => FlutterApiErrorType::HardwareKeyUnsupported,
            WalletRegistrationError::ChallengeRequest(e) => FlutterApiErrorType::from(e),
            WalletRegistrationError::RegistrationRequest(e) => FlutterApiErrorType::from(e),
            _ => FlutterApiErrorType::Generic,
        }
    }
}

impl FlutterApiErrorFields for WalletUnlockError {
    fn typ(&self) -> FlutterApiErrorType {
        match self {
            WalletUnlockError::NotRegistered | WalletUnlockError::NotLocked => FlutterApiErrorType::WalletState,
            WalletUnlockError::Instruction(e) => FlutterApiErrorType::from(e),
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

impl FlutterApiErrorFields for PidIssuanceError {
    fn typ(&self) -> FlutterApiErrorType {
        if let Some(network_error) = detect_networking_error(self) {
            return network_error;
        }

        match self {
            PidIssuanceError::NotRegistered | PidIssuanceError::Locked | PidIssuanceError::SessionState => {
                FlutterApiErrorType::WalletState
            }

            PidIssuanceError::DigidSessionFinish(DigidSessionError::Oidc(OidcError::RedirectUriError(_))) => {
                FlutterApiErrorType::RedirectUri
            }

            PidIssuanceError::PidIssuer(IssuanceSessionError::TokenRequest(_))
            | PidIssuanceError::PidIssuer(IssuanceSessionError::CredentialRequest(_))
            | PidIssuanceError::DigidSessionStart(DigidSessionError::Oidc(OidcError::RedirectUriError(_)))
            | PidIssuanceError::DigidSessionStart(DigidSessionError::Oidc(OidcError::RequestingAccessToken(_)))
            | PidIssuanceError::DigidSessionStart(DigidSessionError::Oidc(OidcError::RequestingUserInfo(_)))
            | PidIssuanceError::DigidSessionFinish(DigidSessionError::Oidc(OidcError::RequestingAccessToken(_)))
            | PidIssuanceError::DigidSessionFinish(DigidSessionError::Oidc(OidcError::RequestingUserInfo(_))) => {
                FlutterApiErrorType::Server
            }
            _ => FlutterApiErrorType::Generic,
        }
    }

    fn data(&self) -> Option<serde_json::Value> {
        match self {
            Self::DigidSessionFinish(DigidSessionError::Oidc(OidcError::RedirectUriError(err))) => {
                [("redirect_error", format!("{:?}", &err.error))]
                    .into_iter()
                    .collect::<serde_json::Value>()
                    .into()
            }
            _ => None,
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
struct DisclosureErrorData<'a> {
    session_type: Option<SessionType>,
    return_url: Option<&'a Url>,
}

impl FlutterApiErrorFields for DisclosureError {
    fn typ(&self) -> FlutterApiErrorType {
        match self {
            DisclosureError::NotRegistered | DisclosureError::Locked | DisclosureError::SessionState => {
                FlutterApiErrorType::WalletState
            }
            DisclosureError::IsoDisclosureSession(mdoc::Error::Holder(HolderError::DisclosureUriSourceMismatch(
                _,
                _,
            )))
            | DisclosureError::VpDisclosureSession(VpClientError::DisclosureUriSourceMismatch(_, _)) => {
                FlutterApiErrorType::DisclosureSourceMismatch
            }
            DisclosureError::IsoDisclosureSession(error) => {
                detect_networking_error(error).unwrap_or(FlutterApiErrorType::Generic)
            }
            DisclosureError::VpDisclosureSession(error) => {
                detect_networking_error(error).unwrap_or(FlutterApiErrorType::Generic)
            }
            DisclosureError::Instruction(error) => FlutterApiErrorType::from(error),
            _ => FlutterApiErrorType::Generic,
        }
    }

    fn data(&self) -> Option<serde_json::Value> {
        let session_type = match self {
            DisclosureError::IsoDisclosureSession(mdoc::Error::Holder(HolderError::DisclosureUriSourceMismatch(
                session_type,
                _,
            )))
            | DisclosureError::VpDisclosureSession(VpClientError::DisclosureUriSourceMismatch(session_type, _)) => {
                Some(*session_type)
            }
            _ => None,
        };
        let return_url = self.return_url();

        (session_type.is_some() || return_url.is_some()).then(|| {
            serde_json::to_value(DisclosureErrorData {
                session_type,
                return_url,
            })
            .unwrap() // This conversion should never fail.
        })
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

impl FlutterApiErrorFields for HistoryError {
    fn typ(&self) -> FlutterApiErrorType {
        match self {
            HistoryError::NotRegistered | HistoryError::Locked => FlutterApiErrorType::WalletState,
            _ => FlutterApiErrorType::Generic,
        }
    }
}

impl FlutterApiErrorFields for ResetError {
    fn typ(&self) -> FlutterApiErrorType {
        match self {
            ResetError::NotRegistered => FlutterApiErrorType::WalletState,
        }
    }
}
