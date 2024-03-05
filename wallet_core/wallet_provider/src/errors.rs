use std::error::Error;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use http::{header, HeaderValue};
use mime::Mime;
use once_cell::sync::Lazy;
use tracing::debug;

use wallet_common::account::messages::errors::{ErrorData, ErrorType};
use wallet_provider_service::{
    account_server::{ChallengeError, InstructionError, RegistrationError, WalletCertificateError},
    hsm,
};

pub static APPLICATION_PROBLEM_JSON: Lazy<Mime> =
    Lazy::new(|| "application/problem+json".parse().expect("Could not parse MIME type"));

/// This type wraps a [`StatusCode`] and [`ErrorData`] instance,
/// which forms the JSON body of the error reponses.
#[derive(Debug, Clone)]
pub struct WalletProviderError {
    pub status_code: StatusCode,
    pub body: ErrorData,
}

/// Any top-level error should implement this trait in order to be
/// convertible to a [`WalletProviderError`].
pub trait ConvertibleError: Error {
    fn error_type(&self) -> ErrorType;
    fn error_title(&self) -> String {
        self.to_string()
    }
}

/// This allows `axum` to interpret the [`WalletProviderError`] and
/// turn it into a response. We make use of the [`IntoResponse`] implementation
/// of the `(StatusCode, X: IntoResponseParts, Y: IntoResponse)` tuple.
impl IntoResponse for WalletProviderError {
    fn into_response(self) -> Response {
        // Panic because the JSON encoding should always succeed.
        debug!("error result: {:?}", self);

        let bytes = serde_json::to_vec(&self.body).expect("Could not encode ErrorData to JSON.");

        (
            self.status_code,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static(APPLICATION_PROBLEM_JSON.as_ref()),
            )],
            bytes,
        )
            .into_response()
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
            status_code: (&error_type).into(),
            body: ErrorData {
                typ: error_type,
                title: value.error_title(),
            },
        }
    }
}

// Implementations of ConvertibleError for all top-level errors.

impl ConvertibleError for ChallengeError {
    fn error_type(&self) -> ErrorType {
        match self {
            ChallengeError::WalletCertificate(WalletCertificateError::UserBlocked) => ErrorType::AccountBlocked,
            ChallengeError::WalletCertificate(_) => ErrorType::ChallengeValidation,
            _ => ErrorType::ChallengeValidation,
        }
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
            RegistrationError::PinPubKeyEncoding(_) => ErrorType::Unexpected,
            RegistrationError::JwtSigning(_) => ErrorType::Unexpected,
            RegistrationError::CertificateStorage(_) => ErrorType::Unexpected,
            RegistrationError::WalletCertificate(_) => ErrorType::Unexpected,
            RegistrationError::HsmError(_) => ErrorType::Unexpected,
        }
    }
}

impl ConvertibleError for InstructionError {
    fn error_type(&self) -> ErrorType {
        match self {
            InstructionError::IncorrectPin(data) => ErrorType::IncorrectPin(*data),
            InstructionError::PinTimeout(data) => ErrorType::PinTimeout(*data),
            InstructionError::AccountBlocked => ErrorType::AccountBlocked,
            InstructionError::Validation(_) => ErrorType::InstructionValidation,
            InstructionError::KeyNotFound(data) => ErrorType::KeyNotFound(data.to_string()),
            InstructionError::Signing(_)
            | InstructionError::Storage(_)
            | InstructionError::WalletCertificate(_)
            | InstructionError::HsmError(_) => ErrorType::Unexpected,
        }
    }
}

impl ConvertibleError for hsm::HsmError {
    fn error_type(&self) -> ErrorType {
        ErrorType::Unexpected
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use wallet_common::account::messages::errors::IncorrectPinData;

    use super::*;

    #[test]
    fn test_json_encoding() {
        let error = WalletProviderError {
            status_code: StatusCode::OK,
            body: ErrorData {
                typ: ErrorType::IncorrectPin(IncorrectPinData {
                    attempts_left: 8,
                    is_final_attempt: false,
                }),
                title: "Error title".to_string(),
            },
        };
        let error_body = serde_json::to_value(error.body).expect("Could not encode error to JSON");

        let expected_body = json!({
            "type": "IncorrectPin",
            "title": "Error title",
            "data": {
                "attempts_left": 8,
                "is_final_attempt": false,
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
