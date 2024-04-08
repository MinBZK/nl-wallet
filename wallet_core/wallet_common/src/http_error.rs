use std::fmt::Debug;

use mime::Mime;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub static APPLICATION_PROBLEM_JSON: Lazy<Mime> =
    Lazy::new(|| "application/problem+json".parse().expect("Could not parse MIME type"));

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
pub struct ErrorData<T> {
    #[serde(flatten)]
    pub typ: T,
    pub title: String,
}

// /// This type wraps a [`StatusCode`] and [`ErrorData`] instance,
// /// which forms the JSON body of the error reponses.
// #[derive(Debug, Clone)]
// pub struct ServerError {
//     pub status_code: StatusCode,
//     pub body: ErrorData<ErrorType>,
// }
