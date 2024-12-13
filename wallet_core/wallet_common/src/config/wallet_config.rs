use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use derive_more::Debug;
use etag::EntityTag;
use serde::Deserialize;
use serde::Serialize;
use webpki::types::TrustAnchor;

use crate::account::serialization::DerVerifyingKey;
use crate::config::digid::DigidApp2AppConfiguration;
use crate::config::http::TlsPinningConfig;
use crate::trust_anchor::DerTrustAnchor;
use crate::urls::BaseUrl;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WalletConfiguration {
    pub lock_timeouts: LockTimeoutConfiguration,
    pub account_server: AccountServerConfiguration,
    pub pid_issuance: PidIssuanceConfiguration,
    pub disclosure: DisclosureConfiguration,
    pub mdoc_trust_anchors: Vec<DerTrustAnchor>,
    pub version: u64,
    pub update_policy_server: UpdatePolicyServerConfiguration,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountServerConfiguration {
    pub http_config: TlsPinningConfig,
    #[debug(skip)]
    pub certificate_public_key: DerVerifyingKey,
    #[debug(skip)]
    pub instruction_result_public_key: DerVerifyingKey,
    #[debug(skip)]
    pub wte_public_key: DerVerifyingKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UpdatePolicyServerConfiguration {
    #[serde(flatten)]
    pub http_config: TlsPinningConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct PidIssuanceConfiguration {
    pub pid_issuer_url: BaseUrl,
    pub digid: DigidConfiguration,
    pub digid_http_config: TlsPinningConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct DigidConfiguration {
    pub client_id: String,
    #[serde(default)]
    pub app2app: Option<DigidApp2AppConfiguration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DisclosureConfiguration {
    #[debug(skip)]
    pub rp_trust_anchors: Vec<DerTrustAnchor>,
}

impl DisclosureConfiguration {
    pub fn rp_trust_anchors(&self) -> Vec<TrustAnchor> {
        self.rp_trust_anchors
            .iter()
            .map(|anchor| (&anchor.owned_trust_anchor).into())
            .collect()
    }
}
