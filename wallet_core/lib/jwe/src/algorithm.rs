use josekit::jwe::alg::ecdh_es::EcdhEsJweAlgorithm;
use jwk_simple::Algorithm;
use strum::EnumString;

/// A type representing possible values of the "alg" header parameter value for JWE, when using an elliptic curve
/// secret and public key. It contains only those algorithms supported by this crate.
/// See: <https://www.rfc-editor.org/rfc/rfc7518.html#section-4>
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "SCREAMING-KEBAB-CASE")]
pub enum EcdhAlgorithm {
    EcdhEs,
    #[strum(serialize = "ECDH-ES+A128KW")]
    EcdhEsA128kw,
    #[strum(serialize = "ECDH-ES+A192KW")]
    EcdhEsA192kw,
    #[strum(serialize = "ECDH-ES+A256KW")]
    EcdhEsA256kw,
}

impl EcdhAlgorithm {
    pub fn try_from_jwk_simple_algorithm(value: &Algorithm) -> Option<Self> {
        match &value {
            Algorithm::EcdhEs => Some(Self::EcdhEs),
            Algorithm::EcdhEsA128kw => Some(Self::EcdhEsA128kw),
            Algorithm::EcdhEsA192kw => Some(Self::EcdhEsA192kw),
            Algorithm::EcdhEsA256kw => Some(Self::EcdhEsA256kw),
            _ => None,
        }
    }
}

impl From<EcdhAlgorithm> for Algorithm {
    fn from(value: EcdhAlgorithm) -> Self {
        match value {
            EcdhAlgorithm::EcdhEs => Self::EcdhEs,
            EcdhAlgorithm::EcdhEsA128kw => Self::EcdhEsA128kw,
            EcdhAlgorithm::EcdhEsA192kw => Self::EcdhEsA192kw,
            EcdhAlgorithm::EcdhEsA256kw => Self::EcdhEsA256kw,
        }
    }
}

impl From<EcdhAlgorithm> for EcdhEsJweAlgorithm {
    fn from(value: EcdhAlgorithm) -> Self {
        match value {
            EcdhAlgorithm::EcdhEs => Self::EcdhEs,
            EcdhAlgorithm::EcdhEsA128kw => Self::EcdhEsA128kw,
            EcdhAlgorithm::EcdhEsA192kw => Self::EcdhEsA192kw,
            EcdhAlgorithm::EcdhEsA256kw => Self::EcdhEsA256kw,
        }
    }
}

/// A type representing the "enc" header parameter value for JWE, i.e. the JWE encryption algorithm. It contains only
/// those algorithms supported by this crate. See: <https://www.rfc-editor.org/rfc/rfc7518.html#section-5>
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "UPPERCASE")]
pub enum JweEncryptionAlgorithm {
    #[strum(serialize = "A128CBC-HS256")]
    A128CbcHs256,
    #[strum(serialize = "A192CBC-HS384")]
    A192CbcHs384,
    #[strum(serialize = "A256CBC-HS512")]
    A256CbcHs512,
    A128Gcm,
    A192Gcm,
    A256Gcm,
}
