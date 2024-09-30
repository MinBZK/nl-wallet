use std::{
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
};

use etag::EntityTag;
use serde::{Deserialize, Serialize};
use webpki::TrustAnchor;

use crate::{
    account::serialization::DerVerifyingKey, config::digid::DigidApp2AppConfiguration, trust_anchor::DerTrustAnchor,
    urls::BaseUrl,
};

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
    pub base_url: BaseUrl,
    // The known public key for the Wallet Provider
    pub certificate_public_key: DerVerifyingKey,
    pub instruction_result_public_key: DerVerifyingKey,
    pub wte_trust_anchors: Vec<DerTrustAnchor>,
}

impl AccountServerConfiguration {
    pub fn wte_trust_anchors(&self) -> Vec<TrustAnchor> {
        self.wte_trust_anchors
            .iter()
            .map(|anchor| (&anchor.owned_trust_anchor).into())
            .collect()
    }
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
