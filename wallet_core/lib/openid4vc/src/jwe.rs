use std::str::FromStr;

use derive_more::Display;
use derive_more::From;
use jwe::algorithm::EncryptionAlgorithm;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use strum::EnumString;

/// A type representing the "enc" header parameter value for JWE, i.e. the JWE encryption algorithm.
/// See: <https://www.rfc-editor.org/rfc/rfc7518.html#section-5>
#[derive(Debug, Clone, PartialEq, Eq, From, Display, SerializeDisplay, DeserializeFromStr)]
pub enum JweEncryptionAlgorithm {
    #[from]
    Known(EncryptionAlgorithm),
    Unknown(String),
}

impl From<&str> for JweEncryptionAlgorithm {
    fn from(value: &str) -> Self {
        match value.parse::<EncryptionAlgorithm>() {
            Ok(algorithm) => Self::Known(algorithm),
            Err(_) => Self::Unknown(value.to_string()),
        }
    }
}

impl FromStr for JweEncryptionAlgorithm {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}

impl JweEncryptionAlgorithm {
    /// Explicitly rank the supported algorithms in order of preference.
    fn preference_rank(algorithm: EncryptionAlgorithm) -> u8 {
        match algorithm {
            EncryptionAlgorithm::A128CbcHs256 => 1,
            EncryptionAlgorithm::A192CbcHs384 => 2,
            EncryptionAlgorithm::A256CbcHs512 => 3,
            EncryptionAlgorithm::A128Gcm => 4,
            EncryptionAlgorithm::A192Gcm => 5,
            EncryptionAlgorithm::A256Gcm => 6,
        }
    }

    pub fn find_preferred_known<'a>(algorithms: impl IntoIterator<Item = &'a Self>) -> Option<EncryptionAlgorithm> {
        algorithms
            .into_iter()
            .filter_map(|algorithm| match algorithm {
                Self::Known(algorithm) => Some(algorithm),
                Self::Unknown(_) => None,
            })
            .copied()
            .max_by_key(|algorithm| Self::preference_rank(*algorithm))
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
    use jwe::algorithm::EncryptionAlgorithm;
    use rstest::rstest;

    use super::JweCompressionAlgorithm;
    use super::JweEncryptionAlgorithm;

    #[rstest]
    #[case::a128gcm("A128GCM", JweEncryptionAlgorithm::Known(EncryptionAlgorithm::A128Gcm))]
    #[case::a256gcm("A256GCM", JweEncryptionAlgorithm::Known(EncryptionAlgorithm::A256Gcm))]
    #[case::a128cbc_hs256("A128CBC-HS256", JweEncryptionAlgorithm::Known(EncryptionAlgorithm::A128CbcHs256))]
    #[case::a256cbc_hs512("A256CBC-HS512", JweEncryptionAlgorithm::Known(EncryptionAlgorithm::A256CbcHs512))]
    #[case::a512gcm("A512GCM", JweEncryptionAlgorithm::Unknown("A512GCM".to_string()))]
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
