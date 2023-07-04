use std::{collections::HashMap, error::Error};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use wallet_provider_service::account_server::{ChallengeError, RegistrationError};

/// This type wraps a [`StatusCode`] and [`ErrorData`] instance,
/// which forms the JSON body of the error reponses.
#[derive(Debug, Clone)]
pub struct WalletProviderError {
    pub status_code: StatusCode,
    pub body: ErrorData,
}

/// The contents of the error JSON are (loosely) based on
/// [RFC 7807](https://datatracker.ietf.org/doc/html/rfc7807).
/// It has the following fields:
///
/// * A `type` field wich contains a uniquely identifiable string.
///   As opposed to what is suggested in the RFC, this is not a
///   resolvable URL.
/// * A `title`, which contains the string value of the error.
/// * Optionally a `data` field, which can contain some key-value
///   data.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorData {
    #[serde(rename = "type")]
    pub typ: ErrorType,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<HashMap<String, DataValue>>,
}

/// The list of uniquely identifiable error types. A client
/// can use these types to distinguish between different errors.
#[derive(Debug, Copy, Clone, Serialize)]
pub enum ErrorType {
    Unexpected,
    ChallengeValidation,
    RegistrationParsing,
}

/// This enum exists to allow the key-value error data to contain
/// multiple types of values. It will most likely be expanded later.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum DataValue {
    #[allow(dead_code)]
    String(String),
}

/// Any top-level error should implement this trait in order to be
/// convertible to a [`WalletProviderError`].
trait ConvertibleError: Error {
    fn error_type(&self) -> ErrorType;

    fn error_title(&self) -> String {
        self.to_string()
    }
    fn error_extra_data(&self) -> Option<HashMap<String, DataValue>> {
        None
    }
}

/// This allows `axum` to interpret the [`WalletProviderError`] and
/// turn it into a response. We just make use of the [`IntoResponse`]
/// implementation of the `(StatusCode, Json<T>)` tuple.
impl IntoResponse for WalletProviderError {
    fn into_response(self) -> Response {
        (self.status_code, Json(self.body)).into_response()
    }
}

/// Allows conversion from any [`Error`] that implements the
/// [`ConvertibleError`] to [`WalletProviderError`]. This makes
/// automatic conversion through the `?` operator possible.
impl<E> From<E> for WalletProviderError
where
    E: ConvertibleError,
{
    fn from(value: E) -> Self {
        let error_type = value.error_type();

        WalletProviderError {
            status_code: error_type.into(),
            body: ErrorData {
                typ: error_type,
                title: value.error_title(),
                data: value.error_extra_data(),
            },
        }
    }
}

/// For the purposes of predictability, there exist a strict mapping
/// of unique error identifiers to HTTP response codes. In this sense
/// the error type gives addtional information over the HTTP response code.
impl From<ErrorType> for StatusCode {
    fn from(value: ErrorType) -> Self {
        match value {
            ErrorType::Unexpected => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorType::ChallengeValidation => StatusCode::UNAUTHORIZED,
            ErrorType::RegistrationParsing => StatusCode::BAD_REQUEST,
        }
    }
}

// Implementations of ConvertibleError for all top-level errors.

impl ConvertibleError for ChallengeError {
    fn error_type(&self) -> ErrorType {
        ErrorType::Unexpected
    }
}

impl ConvertibleError for RegistrationError {
    fn error_type(&self) -> ErrorType {
        match self {
            RegistrationError::ChallengeDecoding(_) => ErrorType::ChallengeValidation,
            RegistrationError::ChallengeValidation(_) => ErrorType::ChallengeValidation,
            RegistrationError::MessageParsing(_) => ErrorType::RegistrationParsing,
            RegistrationError::MessageValidation(_) => ErrorType::RegistrationParsing,
            RegistrationError::SerialNumberMismatch {
                expected: _,
                received: _,
            } => ErrorType::RegistrationParsing,
            RegistrationError::PinPubKeyDecoding(_) => ErrorType::Unexpected,
            RegistrationError::PinPubKeyEncoding(_) => ErrorType::Unexpected,
            RegistrationError::JwtSigning(_) => ErrorType::Unexpected,
            RegistrationError::CertificateStorage(_) => ErrorType::Unexpected,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_json_encoding() {
        let error = WalletProviderError {
            status_code: StatusCode::OK,
            body: ErrorData {
                typ: ErrorType::ChallengeValidation,
                title: "Error title".to_string(),
                data: Some(HashMap::from([
                    ("foo".to_string(), DataValue::String("bar".to_string())),
                    ("bleh".to_string(), DataValue::String("blah".to_string())),
                ])),
            },
        };
        let error_body = serde_json::to_value(error.body).expect("Could not encode error to JSON");

        let expected_body = json!({
            "type": "ChallengeValidation",
            "title": "Error title",
            "data": {
                "foo": "bar",
                "bleh": "blah",
            }
        });
        assert_eq!(error_body, expected_body);
    }

    #[test]
    fn test_error_conversion() {
        let error = RegistrationError::SerialNumberMismatch {
            expected: 1,
            received: 2,
        };
        let wp_error = WalletProviderError::from(error);

        assert_eq!(wp_error.status_code, StatusCode::BAD_REQUEST);

        let wp_error_body = serde_json::to_value(wp_error.body).expect("Could not encode error to JSON");

        let expected_body = json!({
                "type": "RegistrationParsing",
                "title": "incorrect registration serial number (expected: 1, received: 2)"
        });
        assert_eq!(wp_error_body, expected_body);
    }
}
