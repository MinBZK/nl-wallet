// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crypto::x509::CertificateError;
use jwt::error::JwkConversionError;
use jwt::error::JwtError;

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

    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("invalid input: {0}")]
    Deserialization(String),

    #[error("error serializing to JSON: {0}")]
    Serialization(#[from] serde_json::error::Error),

    #[error("{0}")]
    Unspecified(String),

    #[error("the validation ended with {0} unused disclosure(s)")]
    UnusedDisclosures(usize),

    #[error("error parsing JWT: {0}")]
    JwtParsing(#[from] JwtError),

    #[error("error creating JWK from verifying key: {0}")]
    Jwk(#[from] JwkConversionError),

    #[error("missing required property: {0}")]
    MissingRequiredProperty(String),

    #[error("missing required JWK key binding")]
    MissingJwkKeybinding,

    #[error("error decoding base64: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("missing issuer certificate")]
    MissingIssuerCertificate,

    #[error("error constructing issuer certificate: {0}")]
    IssuerCertificate(#[from] CertificateError),
}
