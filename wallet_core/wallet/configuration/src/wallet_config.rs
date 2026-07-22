use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Range;

use attestation_data::disclosure_type::DisclosureTypeConfig;
use attestation_types::claim_path::ClaimPath;
use attestation_types::credential_format::Format;
use attestation_types::credential_kind::CredentialKind;
use chrono::DateTime;
use chrono::Utc;
use crypto::p256_der::DerVerifyingKey;
use crypto::trust_anchor::TrustAnchors;
use derive_more::Debug;
use error_category::ErrorCategory;
use http_utils::client::TlsPinningConfig;
use http_utils::urls::BaseUrl;
use jwt::JwtTyp;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;
use url::Url;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::EnvironmentSpecific;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletConfiguration {
    pub environment: String,
    pub lock_timeouts: LockTimeoutConfiguration,
    pub account_server: AccountServerConfiguration,
    pub pid_attributes: PidAttributesConfiguration,
    pub pid_credential_offer: Url,
    #[debug(skip)]
    pub issuer_trust_anchors: TrustAnchors,
    #[debug(skip)]
    pub wrpac_trust_anchors: TrustAnchors,
    #[debug(skip)]
    pub wrprc_trust_anchors: TrustAnchors,
    pub update_policy_server: UpdatePolicyServerConfiguration,
    pub google_cloud_project_number: u64,
    pub static_assets_base_url: BaseUrl,
    // Note that this serializes to a "start" and "end" field.
    pub maintenance_window: Option<Range<DateTime<Utc>>>,
    pub version: u64,
}

impl JwtTyp for WalletConfiguration {}

impl WalletConfiguration {
    pub fn issuer_trust_anchors(&self) -> &TrustAnchors {
        &self.issuer_trust_anchors
    }

    pub fn wrpac_trust_anchors(&self) -> &TrustAnchors {
        &self.wrpac_trust_anchors
    }

    pub fn wrprc_trust_anchors(&self) -> &TrustAnchors {
        &self.wrprc_trust_anchors
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
    pub wia_trust_anchors: TrustAnchors,
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
    /// The [`CredentialKind`]s of all configured PID attestations, in both the mdoc and the SD-JWT format.
    pub fn credential_kinds(&self) -> HashSet<CredentialKind> {
        self.mso_mdoc
            .keys()
            .map(|doc_type| CredentialKind::new(Format::MsoMdoc, doc_type.clone()))
            .chain(self.sd_jwt_credential_kinds())
            .collect()
    }

    /// The [`CredentialKind`]s of the configured PID attestations in the SD-JWT format.
    pub fn sd_jwt_credential_kinds(&self) -> HashSet<CredentialKind> {
        self.sd_jwt
            .keys()
            .map(|vct| CredentialKind::new(Format::SdJwt, vct.clone()))
            .collect()
    }

    pub fn contains_credential_kind(&self, format: Format, attestation_type: &str) -> bool {
        match format {
            Format::MsoMdoc => self.mso_mdoc.contains_key(attestation_type),
            Format::SdJwt => self.sd_jwt.contains_key(attestation_type),
        }
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
