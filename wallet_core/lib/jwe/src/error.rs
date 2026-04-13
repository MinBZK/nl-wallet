use josekit::JoseError;
use jwk_simple::Algorithm;
use jwk_simple::EcCurve;
use jwk_simple::KeyType;
use jwk_simple::KeyUse;

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(strum::EnumDiscriminants))]
pub enum JwkError {
    #[error("JWK is not valid: {0}")]
    Invalid(#[source] jwk_simple::Error),

    #[error("JWK specifies key use \"{0}\", not encryption")]
    InvalidKeyUse(KeyUse),

    #[error("JWK algorithm \"{0}\" is not supported")]
    UnsupportedAlgorithm(Algorithm),

    #[error("JWK key type \"{0}\" is not consistent with algorithm \"{1}\"")]
    InconsistentKeyType(KeyType, Algorithm),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(strum::EnumDiscriminants))]
pub enum EcdhPublicJwkError {
    #[error("{0}")]
    Jwk(#[from] JwkError),

    #[error("JWK does not contain an algorithm")]
    MissingAlgorithm,

    #[error("JWK EC curve is \"{0}\", not P-256")]
    UnsupportedJwkEcCurve(EcCurve),
}

#[derive(Debug, thiserror::Error)]
pub enum JweEncryptionError {
    #[error("could not serialize data: {0}")]
    Serialization(#[source] serde_json::Error),

    #[error("could not encrypt data: {0}")]
    Encryption(#[source] JoseError),
}

#[derive(Debug, thiserror::Error)]
pub enum JweDecryptionError {
    #[error("could not decrypt data: {0}")]
    Decryption(#[source] JoseError),

    #[error("kid does not match \"{}\": \"{}\"", .0, .1.as_deref().unwrap_or("<NONE>"))]
    IdMismatch(String, Option<String>),

    #[error("could not deserialize data: {0}")]
    Deserialization(#[source] serde_json::Error),
}
