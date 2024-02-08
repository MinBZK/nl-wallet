use std::{
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
};

use etag::EntityTag;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use url::Url;
use webpki::TrustAnchor;

use crate::{account::serialization::DerVerifyingKey, trust_anchor::DerTrustAnchor};

// This should always equal the deep/universal link configured for the app.
static UNIVERSAL_LINK_BASE: Lazy<Url> = Lazy::new(|| {
    Url::parse("https://example.com/deeplink/")
        .expect("hardcoded values should always result in a valid URL")
});
const DIGID_REDIRECT_PATH: &str = "authentication/";
const DISCLOSURE_BASE_PATH: &str = "disclosure/";

pub static ISSUANCE_REDIRECT_URI: Lazy<Url> = Lazy::new(|| {
    UNIVERSAL_LINK_BASE
        .join(DIGID_REDIRECT_PATH)
        .expect("hardcoded values should always result in a valid URL")
});
pub static DISCLOSURE_BASE_URI: Lazy<Url> = Lazy::new(|| {
    UNIVERSAL_LINK_BASE
        .join(DISCLOSURE_BASE_PATH)
        .expect("hardcoded values should always result in a valid URL")
});

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

impl DisclosureConfiguration {
    pub fn rp_trust_anchors(&self) -> Vec<TrustAnchor> {
        self.rp_trust_anchors
            .iter()
            .map(|anchor| (&anchor.owned_trust_anchor).into())
            .collect()
    }
}
