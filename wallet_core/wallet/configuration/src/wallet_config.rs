use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Range;
use std::time::Duration;

use attestation_data::disclosure_type::DisclosureTypeConfig;
use attestation_types::credential_format::Format;
use attestation_types::credential_kind::CredentialKind;
use chrono::DateTime;
use chrono::Utc;
use crypto::keys::AsPublicKeyRef;
use crypto::keys::PublicKeyRef;
use crypto::kid::Kid;
use crypto::p256_der::DerVerifyingKey;
use crypto::trust_anchor::TrustAnchors;
use derive_more::Debug;
use error_category::ErrorCategory;
use http_utils::client::TlsPinningConfig;
use http_utils::urls::BaseUrl;
use itertools::Either;
use jwt::JwtTyp;
use openid4vc::wallet_issuance::AcceptIssuanceTrustAnchors;
use sd_jwt::claims::ClaimName;
use serde::Deserialize;
use serde_with::base64::Base64;
use serde_with::serde_as;
use url::Url;
use utils::date_time_seconds::DateTimeSeconds;
use utils::duration::DurationDays;

use crate::EnvironmentSpecific;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(any(test, feature = "test"), derive(serde::Serialize))]
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

    /// The moment after which this configuration should no longer be used, ensuring the key material and trust anchors
    /// contained in this configuration can't be trusted indefinitely.
    ///
    /// Note that no `nbf` counterpart is present, as the `version` field above already prevents an older
    /// configuration from being presented to the wallet.
    #[serde(rename = "exp")]
    pub expires: DateTimeSeconds,
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

    pub fn accept_issuance_anchors(&self) -> AcceptIssuanceTrustAnchors<'_> {
        AcceptIssuanceTrustAnchors::new(&self.issuer_trust_anchors, &self.account_server.wptl_trust_anchors)
    }
}

impl EnvironmentSpecific for WalletConfiguration {
    fn environment(&self) -> &str {
        &self.environment
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(any(test, feature = "test"), derive(serde::Serialize))]
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(any(test, feature = "test"), derive(serde::Serialize))]
pub struct AccountServerConfiguration {
    pub http_config: TlsPinningConfig,
    #[debug(skip)]
    pub certificate_public_keys: HashMap<Kid, CertificatePublicKey>,
    #[debug(skip)]
    #[serde_as(as = "HashMap<_, Base64>")]
    pub instruction_result_public_keys: HashMap<Kid, DerVerifyingKey>,
    #[debug(skip)]
    pub wptl_trust_anchors: TrustAnchors,
    #[serde(rename = "certificate_refresh_threshold_in_days")]
    #[serde_as(as = "DurationDays")]
    pub certificate_refresh_threshold: Duration,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(any(test, feature = "test"), derive(serde::Serialize))]
pub struct CertificatePublicKey {
    #[serde_as(as = "Base64")]
    pub key: DerVerifyingKey,

    /// The moment from which the wallet provider will actually sign with this key. Keys may be present in the
    /// wallet configuration before this moment, so that wallets have a chance to trust them ahead of the wallet
    /// provider's rollover. Used to determine whether the wallet should actively update its certificate to the
    /// newest key that is already in use.
    pub used_from: DateTimeSeconds,
}

impl AsPublicKeyRef for CertificatePublicKey {
    fn as_public_key_ref<'a>(&'a self) -> PublicKeyRef<'a> {
        PublicKeyRef::from(self.key.as_inner())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(any(test, feature = "test"), derive(serde::Serialize))]
pub struct UpdatePolicyServerConfiguration {
    pub http_config: TlsPinningConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(any(test, feature = "test"), derive(serde::Serialize))]
pub struct PidAttributesConfiguration {
    pub mso_mdoc: HashMap<String, MsoMdocPidClaims>,
    pub sd_jwt: HashMap<String, SdJwtPidClaims>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(any(test, feature = "test"), derive(serde::Serialize))]
pub struct MsoMdocPidClaims {
    pub login: MdocPath,
    pub recovery_code: MdocPath,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(any(test, feature = "test"), derive(serde::Serialize))]
pub struct MdocPath {
    pub namespace: String,
    pub element_name: String,
}

impl MdocPath {
    /// The path to this element as a sequence of claim path keys, i.e. the namespace followed by the element name.
    pub fn key_path(&self) -> [&str; 2] {
        [self.namespace.as_str(), self.element_name.as_str()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(any(test, feature = "test"), derive(serde::Serialize))]
pub struct SdJwtPidClaims {
    pub login: ClaimName,
    pub recovery_code: ClaimName,
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

    /// The name of the claim containing the recovery code in the SD-JWT PID of the specified attestation type.
    pub fn recovery_code_claim(&self, attestation_type: &str) -> Result<&ClaimName, PidAttributesConfigurationError> {
        self.sd_jwt
            .get(attestation_type)
            .map(|paths| &paths.recovery_code)
            .ok_or_else(|| PidAttributesConfigurationError::NoPidConfiguration(attestation_type.to_string()))
    }

    /// The key path of the recovery code in the PID of the specified format and attestation type, if configured.
    pub fn recovery_code_key_path(
        &self,
        format: Format,
        attestation_type: &str,
    ) -> Option<impl Iterator<Item = &str> + Clone> {
        match format {
            Format::MsoMdoc => self
                .mso_mdoc
                .get(attestation_type)
                .map(|paths| Either::Left(paths.recovery_code.key_path().into_iter())),
            Format::SdJwt => self
                .sd_jwt
                .get(attestation_type)
                .map(|paths| Either::Right(std::iter::once(paths.recovery_code.as_str()))),
        }
    }
}

impl DisclosureTypeConfig for PidAttributesConfiguration {
    fn mdoc_login_path(&self, doctype: &str) -> Option<impl Iterator<Item = &str>> {
        self.mso_mdoc
            .get(doctype)
            .map(|paths| paths.login.key_path().into_iter())
    }

    fn sd_jwt_login_path(&self, vct: &str) -> Option<impl Iterator<Item = &str>> {
        self.sd_jwt.get(vct).map(|paths| std::iter::once(paths.login.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use attestation_types::credential_format::Format;
    use itertools::Itertools;
    use rstest::rstest;
    use serde_json::json;

    use super::PidAttributesConfiguration;

    #[test]
    fn test_pid_attributes_configuration_deserialize() {
        let config: PidAttributesConfiguration = serde_json::from_value(json!({
            "mso_mdoc": {
                "pid_doctype": {
                    "login": { "namespace": "pid_namespace", "element_name": "bsn" },
                    "recovery_code": { "namespace": "pid_namespace", "element_name": "recovery_code" }
                }
            },
            "sd_jwt": {
                "pid_vct": { "login": "bsn", "recovery_code": "recovery_code" }
            }
        }))
        .unwrap();

        assert_eq!(
            config
                .recovery_code_key_path(Format::MsoMdoc, "pid_doctype")
                .unwrap()
                .collect_vec(),
            ["pid_namespace", "recovery_code"]
        );
        assert_eq!(config.recovery_code_claim("pid_vct").unwrap().as_str(), "recovery_code");
    }

    #[rstest]
    #[case::login_sd(json!({ "login": "_sd", "recovery_code": "recovery_code" }))]
    #[case::login_array_element(json!({ "login": "...", "recovery_code": "recovery_code" }))]
    #[case::recovery_code_sd(json!({ "login": "bsn", "recovery_code": "_sd" }))]
    #[case::recovery_code_array_element(json!({ "login": "bsn", "recovery_code": "..." }))]
    fn test_pid_attributes_configuration_deserialize_reserved_claim_name(#[case] sd_jwt_claims: serde_json::Value) {
        let result = serde_json::from_value::<PidAttributesConfiguration>(json!({
            "mso_mdoc": {},
            "sd_jwt": { "pid_vct": sd_jwt_claims }
        }));

        assert!(result.is_err());
    }
}
