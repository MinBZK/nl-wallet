use std::fmt::{Display, Formatter};

use http::StatusCode;
use serde::{Deserialize, Serialize};

/// The contents of the error JSON are (loosely) based on
/// [RFC 7807](https://datatracker.ietf.org/doc/html/rfc7807).
/// It serializes having the following fields:
///
/// * A `type` field wich contains a uniquely identifiable string.
///   As opposed to what is suggested in the RFC, this is not a
///   resolvable URL.
/// * A `title`, which contains the string value of the error.
/// * Optionally a `data` field, which can contain some key-value
///   data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    #[serde(flatten)]
    pub typ: ErrorType,
    pub title: String,
}

/// The list of uniquely identifiable error types. A client
/// can use these types to distinguish between different errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ErrorType {
    Unexpected,
    ChallengeValidation,
    RegistrationParsing,
    IncorrectPin(IncorrectPinData),
    PinTimeout(PinTimeoutData),
    AccountBlocked,
    InstructionValidation,
    KeyNotFound(String),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct IncorrectPinData {
    pub attempts_left_in_round: u8,
    pub is_final_round: bool,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct PinTimeoutData {
    pub time_left_in_ms: u64,
}

impl Display for ErrorData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

/// For the purposes of predictability, there exist a strict mapping
/// of unique error identifiers to HTTP response codes. In this sense
/// the error type gives addtional information over the HTTP response code.
impl From<&ErrorType> for StatusCode {
    fn from(value: &ErrorType) -> Self {
        match value {
            ErrorType::Unexpected => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorType::ChallengeValidation => StatusCode::UNAUTHORIZED,
            ErrorType::RegistrationParsing => StatusCode::BAD_REQUEST,
            ErrorType::IncorrectPin(_) => StatusCode::FORBIDDEN,
            ErrorType::PinTimeout(_) => StatusCode::FORBIDDEN,
            ErrorType::AccountBlocked => StatusCode::UNAUTHORIZED,
            ErrorType::InstructionValidation => StatusCode::FORBIDDEN,
            ErrorType::KeyNotFound(_) => StatusCode::NOT_FOUND,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_status_code_to_error_type() {
        assert_eq!(StatusCode::from(&ErrorType::ChallengeValidation).as_u16(), 401);
    }

    #[test]
    fn error_data_should_serialize_with_data() {
        let error_data = ErrorData {
            typ: ErrorType::PinTimeout(PinTimeoutData { time_left_in_ms: 1234 }),
            title: "title123".to_string(),
        };
        assert_eq!(
            json!({"type":"PinTimeout","data":{"time_left_in_ms":1234},"title":"title123"}),
            serde_json::to_value(error_data).unwrap()
        );
    }

    #[test]
    fn error_data_should_serialize_without_data() {
        let error_data = ErrorData {
            typ: ErrorType::ChallengeValidation,
            title: "title123".to_string(),
        };
        assert_eq!(
            json!({"type":"ChallengeValidation","title":"title123"}),
            serde_json::to_value(error_data).unwrap()
        );
    }
}
