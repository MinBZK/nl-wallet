use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use derive_more::Debug;
use etag::EntityTag;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crate::account::serialization::DerVerifyingKey;
use crate::config::digid::DigidApp2AppConfiguration;
use crate::config::http::TlsPinningConfig;
use crate::trust_anchor::BorrowingTrustAnchor;
use crate::urls::BaseUrl;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct WalletConfiguration {
    pub lock_timeouts: LockTimeoutConfiguration,
    pub account_server: AccountServerConfiguration,
    pub pid_issuance: PidIssuanceConfiguration,
    pub disclosure: DisclosureConfiguration,
    #[debug(skip)]
    #[serde_as(as = "Vec<Base64>")]
    pub mdoc_trust_anchors: Vec<BorrowingTrustAnchor>,
    pub version: u64,
}

impl WalletConfiguration {
    pub fn mdoc_trust_anchors(&self) -> Vec<TrustAnchor> {
        self.mdoc_trust_anchors
            .iter()
            .map(Into::<TrustAnchor<'_>>::into)
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct AccountServerConfiguration {
    pub http_config: TlsPinningConfig,
    #[debug(skip)]
    pub certificate_public_key: DerVerifyingKey,
    #[debug(skip)]
    pub instruction_result_public_key: DerVerifyingKey,
    #[debug(skip)]
    pub wte_public_key: DerVerifyingKey,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct DigidConfiguration {
    pub client_id: String,
    #[serde(default)]
    pub app2app: Option<DigidApp2AppConfiguration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct PidIssuanceConfiguration {
    pub pid_issuer_url: BaseUrl,
    pub digid: DigidConfiguration,
    pub digid_http_config: TlsPinningConfig,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct DisclosureConfiguration {
    #[debug(skip)]
    #[serde_as(as = "Vec<Base64>")]
    pub rp_trust_anchors: Vec<BorrowingTrustAnchor>,
}

impl DisclosureConfiguration {
    pub fn rp_trust_anchors(&self) -> Vec<TrustAnchor> {
        self.rp_trust_anchors
            .iter()
            .map(Into::<TrustAnchor<'_>>::into)
            .collect()
    }
}
