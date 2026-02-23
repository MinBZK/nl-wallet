use std::collections::HashMap;
use std::ops::Range;

use chrono::DateTime;
use chrono::Utc;
use derive_more::Debug;
use itertools::Itertools;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use attestation_data::disclosure_type::DisclosureTypeConfig;
use attestation_types::claim_path::ClaimPath;
use crypto::p256_der::DerVerifyingKey;
use crypto::trust_anchor::BorrowingTrustAnchor;
use error_category::ErrorCategory;
use http_utils::tls::pinning::TlsPinningConfig;
use http_utils::urls::BaseUrl;
use jwt::JwtTyp;
use openid4vc::issuer_identifier::CredentialIssuerIdentifier;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::EnvironmentSpecific;
use crate::digid::DigidApp2AppConfiguration;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletConfiguration {
    pub environment: String,
    pub lock_timeouts: LockTimeoutConfiguration,
    pub account_server: AccountServerConfiguration,
    pub pid_attributes: PidAttributesConfiguration,
    pub pid_issuance: PidIssuanceConfiguration,
    pub disclosure: DisclosureConfiguration,
    #[debug(skip)]
    #[serde_as(as = "Vec<Base64>")]
    pub issuer_trust_anchors: Vec<BorrowingTrustAnchor>,
    pub update_policy_server: UpdatePolicyServerConfiguration,
    pub google_cloud_project_number: u64,
    pub static_assets_base_url: BaseUrl,
    // Note that this serializes to a "start" and "end" field.
    pub maintenance_window: Option<Range<DateTime<Utc>>>,
    pub version: u64,
}

impl JwtTyp for WalletConfiguration {}

impl WalletConfiguration {
    pub fn issuer_trust_anchors(&self) -> Vec<TrustAnchor<'_>> {
        self.issuer_trust_anchors
            .iter()
            .map(|anchor| anchor.as_trust_anchor().clone())
            .collect()
    }
}

impl EnvironmentSpecific for WalletConfiguration {
    fn environment(&self) -> &str {
        &self.environment
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockTimeoutConfiguration {
    /// App inactivity warning timeout in seconds
    pub warning_timeout: u16,
    /// App inactivity lock timeout in seconds
    pub inactive_timeout: u16,
    /// App background lock timeout in seconds
    pub background_timeout: u16,
}

impl Default for LockTimeoutConfiguration {
    fn default() -> Self {
        Self {
            warning_timeout: 4 * 60,
            inactive_timeout: 5 * 60,
            background_timeout: 5 * 60,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountServerConfiguration {
    pub http_config: TlsPinningConfig,
    #[debug(skip)]
    #[serde_as(as = "Base64")]
    pub certificate_public_key: DerVerifyingKey,
    #[debug(skip)]
    #[serde_as(as = "Base64")]
    pub instruction_result_public_key: DerVerifyingKey,
    #[debug(skip)]
    #[serde_as(as = "Base64")]
    pub wua_public_key: DerVerifyingKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdatePolicyServerConfiguration {
    pub http_config: TlsPinningConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PidAttributesConfiguration {
    pub mso_mdoc: HashMap<String, PidAttributePaths>,
    pub sd_jwt: HashMap<String, PidAttributePaths>,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum PidAttributesConfigurationError {
    #[category(critical)]
    #[error("attestation type {0} has no PID configuration")]
    NoPidConfiguration(String),
}

impl PidAttributesConfiguration {
    pub fn pid_attestation_types(&self) -> impl Iterator<Item = &str> {
        [&self.mso_mdoc, &self.sd_jwt]
            .into_iter()
            .flat_map(HashMap::keys)
            .unique()
            .map(String::as_str)
    }

    pub fn recovery_code_path(
        &self,
        attestation_type: &str,
    ) -> Result<VecNonEmpty<ClaimPath>, PidAttributesConfigurationError> {
        let path = self
            .sd_jwt
            .get(attestation_type)
            .ok_or(PidAttributesConfigurationError::NoPidConfiguration(
                attestation_type.to_string(),
            ))?
            .recovery_code
            .nonempty_iter()
            .map(|path| ClaimPath::SelectByKey(path.to_string()))
            .collect();

        Ok(path)
    }
}

impl DisclosureTypeConfig for PidAttributesConfiguration {
    fn mdoc_login_path(&self, doctype: &str) -> Option<impl Iterator<Item = &str>> {
        self.mso_mdoc
            .get(doctype)
            .map(|paths| paths.login.iter().map(String::as_str))
    }

    fn sd_jwt_login_path(&self, vct: &str) -> Option<impl Iterator<Item = &str>> {
        self.sd_jwt.get(vct).map(|paths| paths.login.iter().map(String::as_str))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PidAttributePaths {
    pub login: VecNonEmpty<String>,
    pub recovery_code: VecNonEmpty<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PidIssuanceConfiguration {
    pub pid_issuer_url: CredentialIssuerIdentifier,
    pub digid: DigidConfiguration,
    pub digid_http_config: TlsPinningConfig,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DigidConfiguration {
    pub client_id: String,
    #[serde(default)]
    pub app2app: Option<DigidApp2AppConfiguration>,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisclosureConfiguration {
    #[debug(skip)]
    #[serde_as(as = "Vec<Base64>")]
    pub rp_trust_anchors: Vec<BorrowingTrustAnchor>,
}

impl DisclosureConfiguration {
    pub fn rp_trust_anchors(&self) -> Vec<TrustAnchor<'_>> {
        self.rp_trust_anchors
            .iter()
            .map(|anchor| anchor.as_trust_anchor().clone())
            .collect()
    }
}
