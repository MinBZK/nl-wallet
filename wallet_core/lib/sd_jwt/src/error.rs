use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use itertools::Itertools;

use attestation_types::claim_path::ClaimPath;
use jwt::error::JwkConversionError;
use jwt::error::JwtError;
use jwt::error::JwtX5cError;
use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;

use crate::claims::ArrayClaim;
use crate::claims::ClaimName;
use crate::claims::ClaimNameError;
use crate::claims::ClaimType;
use crate::claims::ClaimValue;
use crate::claims::ObjectClaims;
use crate::sd_alg::SdAlg;

#[derive(Debug, thiserror::Error)]
#[error("no hasher implemented for sd_alg: {0:?}")]
pub struct SdAlgHasherNotImplemented(pub SdAlg);

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ClaimError {
    #[error("claim type mismatch; expected {expected}, found `{actual:?}` at path `{path}`")]
    ClaimTypeMismatch {
        expected: ClaimType,
        actual: ClaimType,
        path: ClaimPath,
    },

    #[error("expected an array element, but found an array hash at index `{0}`")]
    ExpectedArrayElement(ClaimPath),

    #[error("disclosure type mismatch; expected {expected}, found {actual} for digest {digest}")]
    DisclosureTypeMismatch {
        expected: ClaimType,
        actual: ClaimType,
        digest: String,
    },

    #[error("encountered a reserved claim name which cannot be used: {0}")]
    ReservedClaimName(#[from] ClaimNameError),

    #[error("invalid array index: {0}, for array: {1:?}")]
    IndexOutOfBounds(usize, Vec<ArrayClaim>),

    #[error("object field `{0}` not found in: `{1:?}`")]
    ObjectFieldNotFound(ClaimName, Box<ObjectClaims>),

    #[error("couldn't find parent for path: /{}", .0.iter().map(ToString::to_string).join("/"))]
    ParentNotFound(Vec<ClaimPath>),

    #[error("unexpected element: {:?}, for path: /{}", .0, .1.iter().map(ToString::to_string).join("/"))]
    UnexpectedElement(Box<ClaimValue>, Vec<ClaimPath>),

    #[error("the array element for path: '{0}' cannot be found")]
    ElementNotFoundInArray(ClaimPath),

    #[error("cannot disclose empty path")]
    EmptyPath,

    #[error("the referenced intermediate element for path: '{0}' cannot be found")]
    IntermediateElementNotFound(String),

    #[error("the referenced element for path: '{0}' cannot be found")]
    ElementNotFound(String),

    #[error("cannot traverse object for path: {0}")]
    UnsupportedTraversalPath(ClaimPath),

    #[error("no disclosure found with digest: {0}, for path: `{1:?}`")]
    DisclosureNotFound(String, Vec<ClaimPath>),

    #[error("expected selective disclosability for claim `{0:?}` is `{1:?}`, but it was {2}")]
    SelectiveDisclosabilityMismatch(Vec<ClaimPath>, ClaimSelectiveDisclosureMetadata, bool),
}

#[derive(Debug, thiserror::Error)]
pub enum DecoderError {
    #[error("unsupported hash algorithm: {0}")]
    Hasher(#[from] SdAlgHasherNotImplemented),

    #[error("SD-JWT format is invalid, input doesn't end with '~'")]
    MissingFinalTilde,

    #[error("SD-JWT format is invalid, input doesn't contain an issuer signed JWT")]
    MissingIssuerSignedJwt,

    #[error("SD-JWT format is invalid, no segments found")]
    MissingSegments,

    #[error("hash occurs multiple times in SD-JWT: {0}")]
    DuplicateHash(String),

    #[error("SD-JWT contains an unreferenced disclosure with digest {0}")]
    UnreferencedDisclosure(String),

    #[error("SD-JWT contains additional disclosures with digests {0:?}")]
    UnreferencedDisclosures(Vec<String>),

    #[error("claim structure error: {0}")]
    ClaimStructure(#[from] ClaimError),

    #[error("base64 decoding of disclosure failed: {0}")]
    Base64Decoding(#[from] base64::DecodeError),

    #[error("JSON deserialization of disclosure failed: {0}")]
    JsonDeserialization(#[from] serde_json::Error),

    #[error("error creating JWK from verifying key: {0}")]
    Jwk(#[from] JwkConversionError),

    #[error("error parsing JWT: {0}")]
    JwtParsing(#[from] JwtError),

    #[error("failed to verify SD-JWT: {0}")]
    JwtVerification(#[from] JwtX5cError),

    #[error("invalid KB-JWT: {0}")]
    KeyBinding(#[from] KeyBindingError),
}

#[derive(Debug, thiserror::Error)]
pub enum EncoderError {
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("claim structure error: {0}")]
    ClaimStructure(#[from] ClaimError),

    #[error("signing error: {0}")]
    Signing(#[from] JwtError),
}

#[derive(Debug, thiserror::Error)]
pub enum KeyBindingError {
    #[error("unexpected nonce, got `{0}``")]
    NonceMismatch(String),

    #[error("iat ({0}) not in acceptable window with duration `{1:?}`, current time: `{2}`")]
    InvalidSignatureTimestamp(DateTime<Utc>, Duration, DateTime<Utc>),

    #[error("jwt error: {0}")]
    Jwt(#[from] JwtError),

    #[error("unsupported hashing algorithm: {0}")]
    Hasher(#[from] SdAlgHasherNotImplemented),
}

#[derive(Debug, thiserror::Error)]
pub enum SigningError {
    #[error("error creating JWK from verifying key: {0}")]
    Jwk(#[from] JwkConversionError),

    #[error("error signing: {0}")]
    Jwt(#[from] JwtError),

    #[error("invalid KB-JWT: {0}")]
    KeyBinding(#[from] KeyBindingError),
}
