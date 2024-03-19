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

use crate::{account::serialization::DerVerifyingKey, trust_anchor::DerTrustAnchor};

#[nutype(
    validate(predicate = |u| !u.cannot_be_a_base() && u.path().ends_with('/')),
    derive(FromStr, Clone, Deserialize, Display, AsRef),
)]
pub struct BaseUrl(Url);

impl BaseUrl {
    pub fn join(&self, input: &str) -> Url {
        // safe to unwrap because we know the URL is a valid base URL
        self.as_ref().join(input).unwrap()
    }
}

pub const DEFAULT_UNIVERSAL_LINK_BASE: &str = "walletdebuginteraction://wallet.edi.rijksoverheid.nl/";
const DIGID_REDIRECT_PATH: &str = "authentication/";
const DISCLOSURE_BASE_PATH: &str = "disclosure/";

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
    pub fn issuance_redirect_uri(universal_link_base: &BaseUrl) -> Url {
        universal_link_base.join(DIGID_REDIRECT_PATH)
    }

    #[inline]
    pub fn disclosure_base_uri(universal_link_base: &BaseUrl) -> Url {
        universal_link_base.join(DISCLOSURE_BASE_PATH)
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
    pub base_url: Url,
    // The known public key for the Wallet Provider
    pub certificate_public_key: DerVerifyingKey,
    pub instruction_result_public_key: DerVerifyingKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct PidIssuanceConfiguration {
    pub pid_issuer_url: Url,
    pub digid_url: Url,
    pub digid_client_id: String,
    #[serde(default)]
    pub digid_trust_anchors: Vec<DerTrustAnchor>,
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
