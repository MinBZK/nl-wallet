// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use itertools::Itertools;

use attestation_types::claim_path::ClaimPath;
use jwt::error::JwkConversionError;
use jwt::error::JwtError;
use jwt::error::JwtX5cError;

/// Alias for a `Result` with the error type [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("invalid input: {0}")]
    InvalidDisclosure(String),

    #[error("invalid hasher: {0}")]
    InvalidHasher(String),

    #[error("data type is not expected: {0}")]
    DataTypeMismatch(String),

    #[error("claim {0} of disclosure already exists")]
    ClaimCollision(String),

    #[error("claim {0} is a reserved claim name and cannot be used")]
    ReservedClaimNameUsed(String),

    #[error("digest {0} appears multiple times")]
    DuplicateDigest(String),

    #[error("array disclosure object contains keys other than `...`")]
    InvalidArrayDisclosureObject,

    #[error("invalid array index: {0}, for array: {1:?}")]
    IndexOutOfBounds(usize, Vec<serde_json::Value>),

    #[error("disclosure not found for key: {0} in map: {1:?}")]
    DisclosureNotFound(String, serde_json::Map<String, serde_json::Value>),

    #[error("couldn't find parent for path: /{}", .0.iter().map(ToString::to_string).join("/"))]
    ParentNotFound(Vec<ClaimPath>),

    #[error("unexpected element: {}, for path: /{}", .0, .1.iter().map(ToString::to_string).join("/"))]
    UnexpectedElement(serde_json::Value, Vec<ClaimPath>),

    #[error("the array element for path: '{path}' cannot be found")]
    ElementNotFoundInArray { path: String },

    #[error("cannot disclose empty path")]
    EmptyPath,

    #[error("the referenced intermediate element for path: '{path}' cannot be found")]
    IntermediateElementNotFound { path: String },

    #[error("the referenced element for path: '{path}' cannot be found")]
    ElementNotFound { path: String },

    #[error("invalid input: {0}")]
    Deserialization(String),

    #[error("error serializing to JSON: {0}")]
    Serialization(#[from] serde_json::error::Error),

    #[error("the validation ended with {0} unused disclosure(s)")]
    UnusedDisclosures(usize),

    #[error("error parsing JWT: {0}")]
    JwtParsing(#[from] JwtError),

    #[error("failed to verify SD-JWT: {0}")]
    JwtVerification(#[from] JwtX5cError),

    #[error("error creating JWK from verifying key: {0}")]
    Jwk(#[from] JwkConversionError),

    #[error("missing required property: {0}")]
    MissingRequiredProperty(String),

    #[error("missing required JWK key binding")]
    MissingJwkKeybinding,

    #[error("cannot traverse object for path: {0}")]
    UnsupportedTraversalPath(ClaimPath),
}
