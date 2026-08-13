use attestation_types::status_claim::StatusClaim;
use attestation_types::status_claim::StatusListClaim;
use chrono::DateTime;
use chrono::Utc;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::CanonicalDistinguishedName;
use serde::Deserialize;
use serde::Deserializer;
use serde::de;
use token_status_list::verification::client::StatusListClient;
use token_status_list::verification::verifier::RevocationStatus;
use token_status_list::verification::verifier::RevocationVerifier;
use url::Url;
use utils::generator::Generator;

use super::payload::UncheckedRegistrationCertificate;
use super::validation::StructurallyValidatedRegistrationCertificate;

/// Reference to the status list entry for this registration certificate.
///
/// Deserialization accepts both the direct shape required by ETSI TS 119 475 clause 6.2.6.1 and the nested OAuth
/// Status List shape used by the informative Annex C example, but structural validation only accepts the normative
/// direct shape with a numeric-string index.
pub struct RegistrationCertificateStatus {
    status_list_claim: StatusListClaim,
    input_format: RegistrationCertificateStatusInputFormat,
}

enum RegistrationCertificateStatusInputFormat {
    DirectNumericString,
    DirectInteger,
    NestedStatusList,
}

impl RegistrationCertificateStatus {
    pub(super) fn has_normative_input_format(&self) -> bool {
        matches!(
            self.input_format,
            RegistrationCertificateStatusInputFormat::DirectNumericString
        )
    }

    fn status_list_claim(&self) -> &StatusListClaim {
        &self.status_list_claim
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RegistrationCertificateStatusWireFormat {
    NestedStatusList(StatusClaim),
    Direct { idx: StatusListIndex, uri: Url },
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StatusListIndex {
    String(String),
    Integer(u32),
}

fn parse_status_list_index<E>(value: &str) -> Result<u32, E>
where
    E: de::Error,
{
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(E::custom(format!(
            "status list index `{value}` is not a numeric string"
        )));
    }

    value.parse().map_err(|_| {
        E::custom(format!(
            "status list index `{value}` does not fit in a 32-bit unsigned integer"
        ))
    })
}

impl<'de> Deserialize<'de> for RegistrationCertificateStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let status = RegistrationCertificateStatusWireFormat::deserialize(deserializer)?;
        let (status_list_claim, input_format) = match status {
            RegistrationCertificateStatusWireFormat::NestedStatusList(status_claim) => {
                let StatusClaim::StatusList(status_list_claim) = status_claim;
                (
                    status_list_claim,
                    RegistrationCertificateStatusInputFormat::NestedStatusList,
                )
            }
            RegistrationCertificateStatusWireFormat::Direct { idx, uri } => {
                let (idx, input_format) = match idx {
                    StatusListIndex::String(value) => (
                        parse_status_list_index(&value)?,
                        RegistrationCertificateStatusInputFormat::DirectNumericString,
                    ),
                    StatusListIndex::Integer(value) => (value, RegistrationCertificateStatusInputFormat::DirectInteger),
                };
                (StatusListClaim { idx, uri }, input_format)
            }
        };
        Ok(Self {
            status_list_claim,
            input_format,
        })
    }
}

impl StructurallyValidatedRegistrationCertificate {
    /// Verifies the referenced status-list token using the registration-certificate trust anchors.
    ///
    /// The signing-certificate distinguished name must come from the already verified registration-certificate
    /// envelope. The status-list signer is required to have the same distinguished name.
    pub async fn validate_status<C>(
        self,
        revocation_verifier: &RevocationVerifier<C>,
        registration_certificate_trust_anchors: &TrustAnchors,
        registration_certificate_signing_certificate_dn: CanonicalDistinguishedName,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<StatusValidatedRegistrationCertificate, RegistrationCertificateStatusValidationError>
    where
        C: StatusListClient,
    {
        let status = revocation_verifier
            .verify(
                registration_certificate_trust_anchors,
                registration_certificate_signing_certificate_dn,
                StatusClaim::StatusList(self.payload().status.status_list_claim().clone()),
                time,
            )
            .await;

        match status {
            RevocationStatus::Valid => Ok(StatusValidatedRegistrationCertificate(self)),
            RevocationStatus::Revoked => Err(RegistrationCertificateStatusValidationError::NotValid),
            RevocationStatus::Undetermined => Err(RegistrationCertificateStatusValidationError::Undetermined),
            RevocationStatus::Corrupted => {
                Err(RegistrationCertificateStatusValidationError::InvalidStatusListReference)
            }
        }
    }
}

/// A structurally validated registration-certificate payload whose referenced status-list entry is valid.
///
/// Registration-certificate header and signature validation remain the responsibility of PVW-5898 and PVW-5899.
pub struct StatusValidatedRegistrationCertificate(StructurallyValidatedRegistrationCertificate);

impl StatusValidatedRegistrationCertificate {
    pub fn payload(&self) -> &UncheckedRegistrationCertificate {
        self.0.payload()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistrationCertificateStatusValidationError {
    #[error("registration certificate status is not valid")]
    NotValid,
    #[error("registration certificate status could not be determined")]
    Undetermined,
    #[error("registration-certificate status-list token or reference is invalid")]
    InvalidStatusListReference,
}
