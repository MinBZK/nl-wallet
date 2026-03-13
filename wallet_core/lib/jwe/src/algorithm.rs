use jwk_simple::Algorithm;

/// A type representing the "alg" header parameter value for JWE, i.e. the JWE algorithm. It contains only those
/// algorithms supported by this crate. See: <https://www.rfc-editor.org/rfc/rfc7518.html#section-4>
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "SCREAMING-KEBAB-CASE")]
pub enum JweAlgorithm {
    EcdhEs,
    #[strum(serialize = "ECDH-ES+A128KW")]
    EcdhEsA128kw,
    #[strum(serialize = "ECDH-ES+A192KW")]
    EcdhEsA192kw,
    #[strum(serialize = "ECDH-ES+A256KW")]
    EcdhEsA256kw,
}

impl JweAlgorithm {
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

impl From<JweAlgorithm> for Algorithm {
    fn from(value: JweAlgorithm) -> Self {
        match value {
            JweAlgorithm::EcdhEs => Self::EcdhEs,
            JweAlgorithm::EcdhEsA128kw => Self::EcdhEsA128kw,
            JweAlgorithm::EcdhEsA192kw => Self::EcdhEsA192kw,
            JweAlgorithm::EcdhEsA256kw => Self::EcdhEsA256kw,
        }
    }
}
