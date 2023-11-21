use std::fmt::Debug;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use url::{ParseError, Url};
use webpki::TrustAnchor;

use crate::{account::serialization::DerVerifyingKey, trust_anchor::DerTrustAnchor};

// This should always equal the deep/universal link configured for the app.
static UNIVERSAL_LINK_BASE: Lazy<Url> =
    Lazy::new(|| Url::parse("walletdebuginteraction://wallet.edi.rijksoverheid.nl/").unwrap());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfiguration {
    pub lock_timeouts: LockTimeoutConfiguration,
    pub account_server: AccountServerConfiguration,
    pub pid_issuance: PidIssuanceConfiguration,
    pub disclosure: DisclosureConfiguration,
    pub mdoc_trust_anchors: Vec<DerTrustAnchor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockTimeoutConfiguration {
    /// App inactivity lock timeout in seconds
    pub inactive_timeout: u16,
    /// App background lock timeout in seconds
    pub background_timeout: u16,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AccountServerConfiguration {
    // The base URL for the Account Server API
    pub base_url: Url,
    // The known public key for the Wallet Provider
    pub certificate_public_key: DerVerifyingKey,
    pub instruction_result_public_key: DerVerifyingKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidIssuanceConfiguration {
    pub pid_issuer_url: Url,
    pub digid_url: Url,
    pub digid_client_id: String,
    pub digid_redirect_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosureConfiguration {
    pub uri_base_path: String,
    pub rp_trust_anchors: Vec<DerTrustAnchor>,
}

impl WalletConfiguration {
    pub fn mdoc_trust_anchors(&self) -> Vec<TrustAnchor> {
        self.mdoc_trust_anchors
            .iter()
            .map(|anchor| (&anchor.owned_trust_anchor).into())
            .collect()
    }
}

impl Debug for AccountServerConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountServerConfiguration")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

impl PidIssuanceConfiguration {
    pub fn digid_redirect_uri(&self) -> Result<Url, ParseError> {
        UNIVERSAL_LINK_BASE.join(&self.digid_redirect_path)
    }
}

impl DisclosureConfiguration {
    pub fn uri_base(&self) -> Result<Url, ParseError> {
        UNIVERSAL_LINK_BASE.join(&self.uri_base_path)
    }

    pub fn rp_trust_anchors(&self) -> Vec<TrustAnchor> {
        self.rp_trust_anchors
            .iter()
            .map(|anchor| (&anchor.owned_trust_anchor).into())
            .collect()
    }
}
