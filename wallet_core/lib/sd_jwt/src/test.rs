use crate::disclosure::Disclosure;
use crate::disclosure::DisclosureContent;
use crate::hasher::Hasher;
use crate::sd_alg::SdAlg;

pub const DIGESTS_KEY: &str = "_sd";
pub const ARRAY_DIGEST_KEY: &str = "...";

pub fn prepare_disclosure(content: DisclosureContent) -> (String, Disclosure) {
    let disclosure = Disclosure::try_new(content).unwrap();
    let hasher = SdAlg::Sha256.hasher().unwrap();
    let digest = hasher.encoded_digest(disclosure.as_str());
    (digest, disclosure)
}

pub fn object_disclosure(key: &'static str, value: serde_json::Value) -> (String, Disclosure) {
    prepare_disclosure(DisclosureContent::ObjectProperty(
        crypto::utils::random_string(16),
        key.parse().unwrap(),
        serde_json::from_value(value).unwrap(),
    ))
}

pub fn array_disclosure(value: serde_json::Value) -> (String, Disclosure) {
    prepare_disclosure(DisclosureContent::ArrayElement(
        crypto::utils::random_string(16),
        serde_json::from_value(value).unwrap(),
    ))
}
