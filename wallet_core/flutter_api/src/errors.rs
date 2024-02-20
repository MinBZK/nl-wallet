use std::{error::Error, fmt::Display};

use anyhow::Chain;
use serde::Serialize;

use wallet::errors::{
    openid, reqwest, AccountProviderError, DigidError, DisclosureError, HistoryError, InstructionError,
    PidIssuanceError, ResetError, UriIdentificationError, WalletInitError, WalletRegistrationError, WalletUnlockError,
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
    Generic,
    Networking,
    WalletState,
    RedirectUri,
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

impl FlutterApiErrorFields for PidIssuanceError {
    fn typ(&self) -> FlutterApiErrorType {
        // Since a `reqwest::Error` can occur in multiple locations
        // within the error tree, just look for it with some help
        // from the `anyhow::Chain` iterator.
        for source in Chain::new(self) {
            // Unfortunately `openid::error::Error` is a special case, because one of its
            // variants holds a `reqwest::Error` with the `transparent` error attribute.
            // This means that the `.source()` method will be forwarded directly to the contained
            // error and the reqwest error itself will be skipped in the source chain!
            // For this reason we need to extract it manually.
            if let Some(openid::Error::Http(_)) = source.downcast_ref::<openid::Error>() {
                return FlutterApiErrorType::Networking;
            }

            if source.is::<reqwest::Error>() {
                return FlutterApiErrorType::Networking;
            }
        }

        match self {
            PidIssuanceError::NotRegistered | PidIssuanceError::Locked | PidIssuanceError::SessionState => {
                FlutterApiErrorType::WalletState
            }
            PidIssuanceError::DigidSessionFinish(DigidError::RedirectUriError {
                error: _,
                error_description: _,
            }) => FlutterApiErrorType::RedirectUri,
            _ => FlutterApiErrorType::Generic,
        }
    }

    fn data(&self) -> Option<serde_json::Value> {
        match self {
            Self::DigidSessionFinish(DigidError::RedirectUriError {
                error,
                error_description: _,
            }) => [("redirect_error", error.clone())]
                .into_iter()
                .collect::<serde_json::Value>()
                .into(),
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
                if Chain::new(error).any(|source| source.is::<reqwest::Error>()) {
                    return FlutterApiErrorType::Networking;
                }

                FlutterApiErrorType::Generic
            }
            _ => FlutterApiErrorType::Generic,
        }
    }
}

impl FlutterApiErrorFields for url::ParseError {
    fn typ(&self) -> FlutterApiErrorType {
        FlutterApiErrorType::WalletState
    }
}

impl From<&AccountProviderError> for FlutterApiErrorType {
    fn from(_value: &AccountProviderError) -> Self {
        Self::Networking
    }
}

impl From<&InstructionError> for FlutterApiErrorType {
    fn from(value: &InstructionError) -> Self {
        match value {
            InstructionError::ServerError(e) => FlutterApiErrorType::from(e),
            InstructionError::InstructionValidation => FlutterApiErrorType::Networking,
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
