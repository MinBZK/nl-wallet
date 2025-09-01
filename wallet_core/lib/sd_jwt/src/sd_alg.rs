use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;

/// <https://www.iana.org/assignments/named-information/named-information.xhtml>
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::Display, SerializeDisplay, DeserializeFromStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SdAlg {
    #[default]
    #[strum(serialize = "sha-256")]
    Sha256,
    #[strum(serialize = "sha-256-128")]
    Sha256_128,
    #[strum(serialize = "sha-256-120")]
    Sha256_120,
    #[strum(serialize = "sha-256-96")]
    Sha256_96,
    #[strum(serialize = "sha-256-64")]
    Sha256_64,
    #[strum(serialize = "sha-256-32")]
    Sha256_32,
    #[strum(serialize = "sha-384")]
    Sha384,
    #[strum(serialize = "sha-512")]
    Sha512,
    Sha3_224,
    Sha3_256,
    Sha3_384,
    Sha3_512,
    #[strum(serialize = "blake2s-256")]
    Blake2s256,
    #[strum(serialize = "blake2b-256")]
    Blake2b256,
    #[strum(serialize = "blake2b-512")]
    Blake2b512,
    K12_256,
    K12_512,
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(SdAlg::default(), "sha-256")]
    #[case(SdAlg::Sha256, "sha-256")]
    #[case(SdAlg::Sha256_128, "sha-256-128")]
    #[case(SdAlg::Sha256_120, "sha-256-120")]
    #[case(SdAlg::Sha256_96, "sha-256-96")]
    #[case(SdAlg::Sha256_64, "sha-256-64")]
    #[case(SdAlg::Sha256_32, "sha-256-32")]
    #[case(SdAlg::Sha384, "sha-384")]
    #[case(SdAlg::Sha512, "sha-512")]
    #[case(SdAlg::Sha3_224, "sha3-224")]
    #[case(SdAlg::Sha3_256, "sha3-256")]
    #[case(SdAlg::Sha3_384, "sha3-384")]
    #[case(SdAlg::Sha3_512, "sha3-512")]
    #[case(SdAlg::Blake2s256, "blake2s-256")]
    #[case(SdAlg::Blake2b256, "blake2b-256")]
    #[case(SdAlg::Blake2b512, "blake2b-512")]
    #[case(SdAlg::K12_256, "k12-256")]
    #[case(SdAlg::K12_512, "k12-512")]
    fn sd_alg_display(#[case] alg: SdAlg, #[case] alg_str: &str) {
        assert_eq!(alg.to_string(), alg_str);
        let parsed: SdAlg = alg_str.parse().unwrap();
        assert_eq!(parsed, alg);
    }
}
