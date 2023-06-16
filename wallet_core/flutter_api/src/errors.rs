use std::{error::Error, fmt::Display};

use serde::Serialize;

use wallet::wallet::{AccountServerClientError, WalletInitError, WalletRegistrationError, WalletUnlockError};

/// A type encapsulating data about a Flutter error that
/// is to be serialized to JSON and sent to Flutter.
#[derive(Debug, Serialize)]
pub struct FlutterApiError {
    #[serde(rename = "type")]
    typ: FlutterApiErrorType,
    description: String,
    /// This property is present only for debug logging purposes and will not be encoded to JSON.
    #[serde(skip)]
    source: Box<dyn Error>,
}

#[derive(Debug, Serialize)]
pub enum FlutterApiErrorType {
    Generic,
    Networking, // TODO: have different networking error types
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
    }
}

/// Allow conversion from any error for which a reference can be converted to a FlutterApiErrorType.
impl<E> From<E> for FlutterApiError
where
    E: Error + 'static,
    for<'a> &'a E: Into<FlutterApiErrorType>,
{
    fn from(value: E) -> Self {
        FlutterApiError {
            typ: (&value).into(),
            description: value.to_string(),
            source: Box::new(value),
        }
    }
}

// The below traits will output the correct FlutterApiErrorType for a given error
// that can be returned from the Wallet. This can possibly be several layers deep.
impl From<&WalletInitError> for FlutterApiErrorType {
    fn from(_value: &WalletInitError) -> Self {
        FlutterApiErrorType::Generic
    }
}

impl From<&WalletRegistrationError> for FlutterApiErrorType {
    fn from(value: &WalletRegistrationError) -> Self {
        match value {
            WalletRegistrationError::ChallengeRequest(e) => Self::from(e),
            WalletRegistrationError::RegistrationRequest(e) => Self::from(e),
            _ => FlutterApiErrorType::Generic,
        }
    }
}

impl From<&WalletUnlockError> for FlutterApiErrorType {
    fn from(_value: &WalletUnlockError) -> Self {
        FlutterApiErrorType::Networking
    }
}

impl From<&AccountServerClientError> for FlutterApiErrorType {
    fn from(_value: &AccountServerClientError) -> Self {
        FlutterApiErrorType::Networking
    }
}
