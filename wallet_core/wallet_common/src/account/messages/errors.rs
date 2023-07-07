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

/// This enum exists to allow the key-value error data to contain
/// multiple types of values. It will most likely be expanded later.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataValue {
    #[allow(dead_code)]
    String(String),
}

/// The list of uniquely identifiable error types. A client
/// can use these types to distinguish between different errors.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    Unexpected,
    ChallengeValidation,
    RegistrationParsing,
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
        }
    }
}
