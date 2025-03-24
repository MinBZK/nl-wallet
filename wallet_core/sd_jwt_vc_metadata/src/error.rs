// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Alias for a `Result` with the error type [`Error`].
pub type Result<T> = ::core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, strum::IntoStaticStr, PartialEq)]
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

    #[error("digest {0} appears multiple times")]
    DuplicateDigest(String),

    #[error("array disclosure object contains keys other than `...`")]
    InvalidArrayDisclosureObject,

    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("invalid input: {0}")]
    Deserialization(String),

    #[error("{0}")]
    Unspecified(String),

    #[error("salt size must be greater than or equal to 16")]
    InvalidSaltSize,

    #[error("the validation ended with {0} unused disclosure(s)")]
    UnusedDisclosures(usize),

    #[error("JWS creation failure: {0}")]
    JwsSignerFailure(String),

    #[error("Missing required KB-JWT")]
    MissingKeyBindingJwt,
}
