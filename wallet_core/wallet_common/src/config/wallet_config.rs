use std::{
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
};

use etag::EntityTag;
use nutype::nutype;
use serde::{Deserialize, Serialize};
use url::Url;
use webpki::TrustAnchor;

use crate::{
    account::serialization::DerVerifyingKey, config::digid::DigidApp2AppConfiguration, trust_anchor::DerTrustAnchor,
};

#[nutype(
    validate(predicate = |u| !u.cannot_be_a_base()),
    derive(FromStr, Debug, Clone, Deserialize, Serialize, Display, AsRef, TryFrom, PartialEq, Eq, Hash),
)]
pub struct BaseUrl(Url);

impl BaseUrl {
    // removes leading forward slashes, calls `Url::join` and unwraps the result
    // the idea behind this is that a BaseURL is intended to be joined with a relative path and not an absolute path
    pub fn join(&self, input: &str) -> Url {
        let mut ret = self.as_ref().clone();
        // both safe to unwrap because we know the URL is a valid base URL
        if !ret.path().ends_with('/') {
            ret.path_segments_mut().unwrap().push("/");
        }
        ret.join(input.trim_start_matches('/')).unwrap()
    }

    // call .join, but converted into a BaseUrl
    pub fn join_base_url(&self, input: &str) -> Self {
        self.join(input).try_into().unwrap()
    }
}

pub const DEFAULT_UNIVERSAL_LINK_BASE: &str = "walletdebuginteraction://wallet.edi.rijksoverheid.nl/";
const ISSUANCE_BASE_PATH: &str = "return-from-digid";
const DISCLOSURE_BASE_PATH: &str = "disclosure";

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct WalletConfiguration {
    pub lock_timeouts: LockTimeoutConfiguration,
    pub account_server: AccountServerConfiguration,
    pub pid_issuance: PidIssuanceConfiguration,
    pub disclosure: DisclosureConfiguration,
    pub mdoc_trust_anchors: Vec<DerTrustAnchor>,
    pub version: u64,
}

impl WalletConfiguration {
    pub fn mdoc_trust_anchors(&self) -> Vec<TrustAnchor> {
        self.mdoc_trust_anchors
            .iter()
            .map(|anchor| (&anchor.owned_trust_anchor).into())
            .collect()
    }

    pub fn to_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    #[inline]
    pub fn issuance_base_uri(universal_link_base: &BaseUrl) -> BaseUrl {
        universal_link_base.join_base_url(ISSUANCE_BASE_PATH)
    }

    #[inline]
    pub fn disclosure_base_uri(universal_link_base: &BaseUrl) -> BaseUrl {
        universal_link_base.join_base_url(DISCLOSURE_BASE_PATH)
    }
}

impl From<&WalletConfiguration> for EntityTag {
    fn from(value: &WalletConfiguration) -> Self {
        EntityTag::new(false, &value.to_hash().to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct LockTimeoutConfiguration {
    /// App inactivity lock timeout in seconds
    pub inactive_timeout: u16,
    /// App background lock timeout in seconds
    pub background_timeout: u16,
}

impl Default for LockTimeoutConfiguration {
    fn default() -> Self {
        Self {
            inactive_timeout: 5 * 60,
            background_timeout: 5 * 60,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct AccountServerConfiguration {
    // The base URL for the Account Server API
    pub base_url: BaseUrl,
    // The known public key for the Wallet Provider
    pub certificate_public_key: DerVerifyingKey,
    pub instruction_result_public_key: DerVerifyingKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct PidIssuanceConfiguration {
    pub pid_issuer_url: BaseUrl,
    pub digid_url: BaseUrl,
    pub digid_client_id: String,
    #[serde(default)]
    pub digid_trust_anchors: Vec<DerTrustAnchor>,
    #[serde(default)]
    pub digid_app2app: Option<DigidApp2AppConfiguration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct DisclosureConfiguration {
    pub rp_trust_anchors: Vec<DerTrustAnchor>,
}

impl Debug for AccountServerConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountServerConfiguration")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

impl PidIssuanceConfiguration {
    pub fn digid_trust_anchors(&self) -> Vec<reqwest::Certificate> {
        self.digid_trust_anchors
            .iter()
            .map(|anchor| {
                reqwest::Certificate::from_der(&anchor.der_bytes)
                    .expect("DerTrustAnchor should be able to be converted to reqwest::Certificate")
            })
            .collect()
    }
}

impl DisclosureConfiguration {
    pub fn rp_trust_anchors(&self) -> Vec<TrustAnchor> {
        self.rp_trust_anchors
            .iter()
            .map(|anchor| (&anchor.owned_trust_anchor).into())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case("https://example.com/", Ok(()))]
    #[case("https://example.com/", Ok(()))]
    #[case("https://example.com/path/", Ok(()))]
    #[case("https://example.com/path", Ok(()))] // this is okay, since the `.join` method will add a trailing slash
    #[case("data:image/jpeg;base64,/9j/4AAQSkZJRgABAgAAZABkAAD", Err(()))]
    #[tokio::test]
    async fn base_url(#[case] value: &str, #[case] expected_err: Result<(), ()>) {
        // The `BaseUrlParseError` that `nutype` returns does not implement `PartialEq`
        assert_eq!(value.parse::<BaseUrl>().map(|_| ()).map_err(|_| ()), expected_err);
    }

    #[rstest]
    #[case("https://example.com/", "to", "https://example.com/to")]
    #[case("https://example.com/", "/to", "https://example.com/to")]
    #[case("https://example.com/", "to/", "https://example.com/to/")]
    #[case("https://example.com/", "/to/", "https://example.com/to/")]
    #[case("https://example.com/", "path/to", "https://example.com/path/to")]
    #[case("https://example.com/", "/path/to", "https://example.com/path/to")]
    #[case("https://example.com/", "path/to/", "https://example.com/path/to/")]
    #[case("https://example.com/", "/path/to/", "https://example.com/path/to/")]
    #[case("https://example.com/path/", "to", "https://example.com/path/to")]
    #[case("https://example.com/path/", "to/", "https://example.com/path/to/")]
    #[case("https://example.com/path/", "to/success", "https://example.com/path/to/success")]
    #[case("https://example.com/path/", "to/success/", "https://example.com/path/to/success/")]
    // if path is absolute, remove leading '/'
    #[case("https://example.com/path/", "/to", "https://example.com/path/to")]
    #[case("https://example.com/path/", "/to/", "https://example.com/path/to/")]
    #[case("https://example.com/path/", "/to/success", "https://example.com/path/to/success")]
    #[case("https://example.com/path/", "/to/success/", "https://example.com/path/to/success/")]
    #[tokio::test]
    async fn base_url_join(#[case] value: BaseUrl, #[case] path: &str, #[case] expected: &str) {
        assert_eq!(value.join(path).as_str(), expected);
    }

    #[rstest]
    #[case("https://example.com/", "to", "https://example.com/to")]
    #[case("https://example.com/", "/to", "https://example.com/to")]
    #[case("https://example.com/", "to/", "https://example.com/to/")]
    #[case("https://example.com/", "/to/", "https://example.com/to/")]
    #[case("https://example.com/", "path/to", "https://example.com/path/to")]
    #[case("https://example.com/", "/path/to", "https://example.com/path/to")]
    #[case("https://example.com/", "path/to/", "https://example.com/path/to/")]
    #[case("https://example.com/", "/path/to/", "https://example.com/path/to/")]
    #[case("https://example.com/path/", "to", "https://example.com/path/to")]
    #[case("https://example.com/path/", "to/", "https://example.com/path/to/")]
    #[case("https://example.com/path/", "to/success", "https://example.com/path/to/success")]
    #[case("https://example.com/path/", "to/success/", "https://example.com/path/to/success/")]
    // if path is absolute, remove leading '/'
    #[case("https://example.com/path/", "/to", "https://example.com/path/to")]
    #[case("https://example.com/path/", "/to/", "https://example.com/path/to/")]
    #[case("https://example.com/path/", "/to/success", "https://example.com/path/to/success")]
    #[case("https://example.com/path/", "/to/success/", "https://example.com/path/to/success/")]
    #[tokio::test]
    async fn base_url_join_base_url(#[case] value: BaseUrl, #[case] path: &str, #[case] expected: &str) {
        assert_eq!(value.join_base_url(path).as_ref().as_str(), expected);
    }
}
