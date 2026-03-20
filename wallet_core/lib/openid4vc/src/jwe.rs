use std::str::FromStr;

use derive_more::Display;
use derive_more::From;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use strum::EnumString;

/// A type representing the "enc" header parameter value for JWE, i.e. the JWE encryption algorithm.
/// See: <https://www.rfc-editor.org/rfc/rfc7518.html#section-5>
#[derive(Debug, Clone, PartialEq, Eq, From, Display, SerializeDisplay, DeserializeFromStr)]
pub enum JweEncryptionAlgorithm {
    #[from]
    Known(jwe::algorithm::JweEncryptionAlgorithm),
    Unknown(String),
}

impl From<&str> for JweEncryptionAlgorithm {
    fn from(value: &str) -> Self {
        match value.parse::<jwe::algorithm::JweEncryptionAlgorithm>() {
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

impl Default for JweEncryptionAlgorithm {
    fn default() -> Self {
        // This is the default value for OpenID4VP.
        // See: https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-5.1-2.4.2.2
        Self::Known(jwe::algorithm::JweEncryptionAlgorithm::A128Gcm)
    }
}

impl JweEncryptionAlgorithm {
    /// Explicitly rank the supported algorithms in order of preference.
    pub fn preference_rank(&self) -> Option<u8> {
        match self {
            Self::Known(algorithm) => Some(match algorithm {
                jwe::algorithm::JweEncryptionAlgorithm::A128Gcm => 1,
                jwe::algorithm::JweEncryptionAlgorithm::A128CbcHs256 => 2,
                jwe::algorithm::JweEncryptionAlgorithm::A192Gcm => 3,
                jwe::algorithm::JweEncryptionAlgorithm::A192CbcHs384 => 4,
                jwe::algorithm::JweEncryptionAlgorithm::A256Gcm => 5,
                jwe::algorithm::JweEncryptionAlgorithm::A256CbcHs512 => 6,
            }),
            Self::Unknown(_) => None,
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
    #[case::a128gcm(
        "A128GCM",
        JweEncryptionAlgorithm::Known(jwe::algorithm::JweEncryptionAlgorithm::A128Gcm)
    )]
    #[case::a256gcm(
        "A256GCM",
        JweEncryptionAlgorithm::Known(jwe::algorithm::JweEncryptionAlgorithm::A256Gcm)
    )]
    #[case::a128cbc_hs256(
        "A128CBC-HS256",
        JweEncryptionAlgorithm::Known(jwe::algorithm::JweEncryptionAlgorithm::A128CbcHs256)
    )]
    #[case::a256cbc_hs512(
        "A256CBC-HS512",
        JweEncryptionAlgorithm::Known(jwe::algorithm::JweEncryptionAlgorithm::A256CbcHs512)
    )]
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
