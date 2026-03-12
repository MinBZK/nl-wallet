use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use strum::EnumString;

/// A type representing the "alg" header parameter value for JWE, i.e. the JWE algorithm.
/// See: <https://www.rfc-editor.org/rfc/rfc7518.html#section-4>
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString, SerializeDisplay, DeserializeFromStr)]
#[strum(serialize_all = "SCREAMING-KEBAB-CASE")]
pub enum JweAlgorithm {
    EcdhEs,
    #[strum(default)]
    Other(String),
}

/// A type representing the "alg" header parameter value for JWE, i.e. the JWE encryption algorithm.
/// See: <https://www.rfc-editor.org/rfc/rfc7518.html#section-5>
#[derive(Debug, Clone, Default, PartialEq, Eq, strum::Display, EnumString, SerializeDisplay, DeserializeFromStr)]
#[strum(serialize_all = "UPPERCASE")]
pub enum JweEncryptionAlgorithm {
    A256Gcm,
    A192Gcm,
    #[default]
    A128Gcm,

    #[strum(default)]
    Other(String),
}

impl JweEncryptionAlgorithm {
    pub fn is_supported(&self) -> bool {
        matches!(self, Self::A128Gcm | Self::A192Gcm | Self::A256Gcm)
    }

    // This is explicitly ranking the algorithms to prevent mis-interpretation of the Ord order
    pub fn preference_rank(&self) -> Option<u8> {
        match self {
            Self::A128Gcm => Some(1),
            Self::A192Gcm => Some(2),
            Self::A256Gcm => Some(3),
            Self::Other(_) => None,
        }
    }
}

/// A type representing the "zip" header parameter value for JWE, i.e. the JWE encryption compression algorithm.
/// See: <https://www.rfc-editor.org/rfc/rfc7518.html#section-7.3>
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString, SerializeDisplay, DeserializeFromStr)]
pub enum JweCompressionAlgorithm {
    #[strum(serialize = "DEF")]
    Deflate,
    #[strum(default)]
    Other(String),
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::JweAlgorithm;
    use super::JweCompressionAlgorithm;
    use super::JweEncryptionAlgorithm;

    #[rstest]
    #[case::ecdh_es("ECDH-ES", JweAlgorithm::EcdhEs)]
    #[case::ecdh_es("dir", JweAlgorithm::Other("dir".to_string()))]
    fn test_jwe_algorithm_parse(#[case] input: &str, #[case] expected_jwe_alg: JweAlgorithm) {
        let jwe_alg = input
            .parse::<JweAlgorithm>()
            .expect("parsing JweAlgorithm from string should succeed");

        assert_eq!(jwe_alg, expected_jwe_alg);
        assert_eq!(jwe_alg.to_string(), *input);
    }

    #[rstest]
    #[case::a128gcm("A128GCM", JweEncryptionAlgorithm::A128Gcm)]
    #[case::a192gcm("A192GCM", JweEncryptionAlgorithm::A192Gcm)]
    #[case::a256gcm("A256GCM", JweEncryptionAlgorithm::A256Gcm)]
    #[case::a128cbc_hs256("A128CBC-HS256", JweEncryptionAlgorithm::Other("A128CBC-HS256".to_string()))]
    fn test_jwe_encryption_algorithm_parse(#[case] input: &str, #[case] expected_jwe_enc: JweEncryptionAlgorithm) {
        let jwe_enc = input
            .parse::<JweEncryptionAlgorithm>()
            .expect("parsing JweEncryptionAlgorithm from string should succeed");

        assert_eq!(jwe_enc, expected_jwe_enc);
        assert_eq!(jwe_enc.to_string(), *input);
    }

    #[rstest]
    #[case::a128gcm(JweEncryptionAlgorithm::A128Gcm, Some(1))]
    #[case::a192gcm(JweEncryptionAlgorithm::A192Gcm, Some(2))]
    #[case::a256gcm(JweEncryptionAlgorithm::A256Gcm, Some(3))]
    #[case::other(
        JweEncryptionAlgorithm::Other("A512GCM".to_string()),
        None
    )]
    fn test_jwe_encryption_algorithm_preference_rank(
        #[case] input: JweEncryptionAlgorithm,
        #[case] expected_rank: Option<u8>,
    ) {
        assert_eq!(input.preference_rank(), expected_rank);
    }

    #[rstest]
    #[case::ecdh_es("DEF", JweCompressionAlgorithm::Deflate)]
    #[case::ecdh_gzip("gzip", JweCompressionAlgorithm::Other("gzip".to_string()))]
    fn test_jwe_compression_algorithm_parse(#[case] input: &str, #[case] expected_jwe_zip: JweCompressionAlgorithm) {
        let jwe_zip = input
            .parse::<JweCompressionAlgorithm>()
            .expect("parsing JweCompressionAlgorithm from string should succeed");

        assert_eq!(jwe_zip, expected_jwe_zip);
        assert_eq!(jwe_zip.to_string(), *input);
    }
}
