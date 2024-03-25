use std::{error::Error, fmt::Display};

use anyhow::Chain;
use serde::Serialize;

use wallet::{
    errors::{
        reqwest, AccountProviderError, DisclosureError, HistoryError, InstructionError, PidIssuanceError, ResetError,
        UriIdentificationError, WalletInitError, WalletRegistrationError, WalletUnlockError,
    },
    openid4vc::{IssuanceSessionError, OidcError},
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

            PidIssuanceError::DigidSessionFinish(OidcError::RedirectUriError(_)) => FlutterApiErrorType::RedirectUri,

            PidIssuanceError::PidIssuer(IssuanceSessionError::TokenRequest(_))
            | PidIssuanceError::PidIssuer(IssuanceSessionError::CredentialRequest(_))
            | PidIssuanceError::DigidSessionStart(OidcError::RedirectUriError(_))
            | PidIssuanceError::DigidSessionStart(OidcError::RequestingAccessToken(_))
            | PidIssuanceError::DigidSessionStart(OidcError::RequestingUserInfo(_))
            | PidIssuanceError::DigidSessionFinish(OidcError::RequestingAccessToken(_))
            | PidIssuanceError::DigidSessionFinish(OidcError::RequestingUserInfo(_)) => {
                crate::errors::FlutterApiErrorType::Server
            }
            _ => FlutterApiErrorType::Generic,
        }
    }

    fn data(&self) -> Option<serde_json::Value> {
        match self {
            Self::DigidSessionFinish(OidcError::RedirectUriError(err)) => {
                [("redirect_error", format!("{:?}", &err.error))]
                    .into_iter()
                    .collect::<serde_json::Value>()
                    .into()
            }
            _ => None,
        }
    }
}

impl FlutterApiErrorFields for DisclosureError {
    fn typ(&self) -> FlutterApiErrorType {
        match self {
            DisclosureError::NotRegistered | DisclosureError::Locked | DisclosureError::SessionState => {
                FlutterApiErrorType::WalletState
            }
            DisclosureError::DisclosureSession(error) => {
                detect_networking_error(error).unwrap_or(FlutterApiErrorType::Generic)
            }
            DisclosureError::Instruction(error) => FlutterApiErrorType::from(error),
            _ => FlutterApiErrorType::Generic,
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
