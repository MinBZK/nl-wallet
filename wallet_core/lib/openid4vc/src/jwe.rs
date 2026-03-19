use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use strum::EnumString;

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
        self.preference_rank().is_some()
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

    use super::JweCompressionAlgorithm;
    use super::JweEncryptionAlgorithm;

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
