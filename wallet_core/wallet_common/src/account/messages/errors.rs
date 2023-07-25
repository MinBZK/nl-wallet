use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use http::StatusCode;
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    #[serde(rename = "type")]
    pub typ: ErrorType,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, DataValue>>,
}

impl ErrorData {
    pub fn try_get<T>(&self, key: &str) -> Option<T>
    where
        T: From<DataValue>,
    {
        self.clone().data.and_then(|d| d.get(key).cloned()).map(|d| d.into())
    }

    pub fn unwrap_get<T>(&self, key: &str) -> T
    where
        T: From<DataValue>,
    {
        self.try_get(key)
            .unwrap_or_else(|| panic!("data should contain key {}", key))
    }
}

/// This enum exists to allow the key-value error data to contain
/// multiple types of values. It will most likely be expanded later.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataValue {
    String(String),
    Number(u64),
    Boolean(bool),
}

impl From<String> for DataValue {
    fn from(value: String) -> Self {
        DataValue::String(value)
    }
}

impl From<DataValue> for String {
    fn from(value: DataValue) -> Self {
        match value {
            DataValue::String(s) => s,
            _ => panic!("cannot be converted to String: {:?}", value),
        }
    }
}

impl From<u64> for DataValue {
    fn from(value: u64) -> Self {
        DataValue::Number(value)
    }
}

impl From<DataValue> for u64 {
    fn from(value: DataValue) -> Self {
        match value {
            DataValue::Number(n) => n,
            _ => panic!("cannot be converted to u64: {:?}", value),
        }
    }
}

impl From<bool> for DataValue {
    fn from(value: bool) -> Self {
        DataValue::Boolean(value)
    }
}

impl From<DataValue> for bool {
    fn from(value: DataValue) -> Self {
        match value {
            DataValue::Boolean(b) => b,
            _ => panic!("cannot be converted to bool: {:?}", value),
        }
    }
}

/// The list of uniquely identifiable error types. A client
/// can use these types to distinguish between different errors.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    Unexpected,
    ChallengeValidation,
    RegistrationParsing,
    IncorrectPin,
    PinTimeout,
    AccountBlocked,
    InstructionValidation,
}

impl Display for ErrorData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
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
            ErrorType::IncorrectPin => StatusCode::FORBIDDEN,
            ErrorType::PinTimeout => StatusCode::FORBIDDEN,
            ErrorType::AccountBlocked => StatusCode::UNAUTHORIZED,
            ErrorType::InstructionValidation => StatusCode::FORBIDDEN,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_code_to_error_type() {
        assert_eq!(StatusCode::from(ErrorType::ChallengeValidation).as_u16(), 401);
    }
}
