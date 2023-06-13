use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use serde::Serialize;

use wallet::wallet::{AccountServerClientError, WalletInitError, WalletRegistrationError};

#[derive(Debug, Serialize)]
pub struct FlutterApiError {
    #[serde(rename = "type")]
    typ: FlutterErrorType,
    description: String,
}

#[derive(Debug, Serialize)]
pub enum FlutterErrorType {
    Generic,
    Networking, // TODO: have different networking error types
}

impl Display for FlutterApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl Error for FlutterApiError {}

impl<E> From<E> for FlutterApiError
where
    E: Error + Into<FlutterErrorType>,
{
    fn from(value: E) -> Self {
        let description = value.to_string();

        FlutterApiError {
            typ: value.into(),
            description,
        }
    }
}

impl From<WalletInitError> for FlutterErrorType {
    fn from(_value: WalletInitError) -> Self {
        FlutterErrorType::Generic
    }
}

impl From<WalletRegistrationError> for FlutterErrorType {
    fn from(value: WalletRegistrationError) -> Self {
        match value {
            WalletRegistrationError::ChallengeRequest(e) => e.into(),
            WalletRegistrationError::RegistrationRequest(e) => e.into(),
            _ => FlutterErrorType::Generic,
        }
    }
}

impl From<AccountServerClientError> for FlutterErrorType {
    fn from(_value: AccountServerClientError) -> Self {
        FlutterErrorType::Networking
    }
}
