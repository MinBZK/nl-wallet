use itertools::Itertools;

use attestation_types::claim_path::ClaimPath;
use jwt::error::JwkConversionError;
use jwt::error::JwtError;
use jwt::error::JwtX5cError;

use crate::claims::ArrayClaim;
use crate::claims::ClaimName;
use crate::claims::ClaimNameError;
use crate::claims::ClaimValue;
use crate::claims::ObjectClaims;
use crate::sd_alg::SdAlg;

/// Alias for a `Result` with the error type [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("invalid input: {0}")]
    InvalidDisclosure(String),

    #[error("data type is not expected: {0}")]
    DataTypeMismatch(String),

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

    #[error("invalid input: {0}")]
    Deserialization(String),

    #[error("error serializing to JSON: {0}")]
    Serialization(#[from] serde_json::error::Error),

    #[error("error parsing JWT: {0}")]
    JwtParsing(#[from] JwtError),

    #[error("failed to verify SD-JWT: {0}")]
    JwtVerification(#[from] JwtX5cError),

    #[error("error creating JWK from verifying key: {0}")]
    Jwk(#[from] JwkConversionError),

    #[error("missing required JWK key binding")]
    MissingJwkKeybinding,

    #[error("cannot traverse object for path: {0}")]
    UnsupportedTraversalPath(ClaimPath),

    #[error("no hasher implemented for sd_alg: {0:?}")]
    SdAlgHasherNotImplemented(SdAlg),

    #[error("hash occurs multiple times in SD-JWT: {0}")]
    DuplicateHash(String),

    #[error("SD-JWT contains an unreferenced disclosure with digest {0}")]
    UnreferencedDisclosure(String),

    #[error("SD-JWT contains additional disclosures with digests {0:?}")]
    UnreferencedDisclosures(Vec<String>),
}
