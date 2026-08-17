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

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::sync::Arc;
    use std::time::Duration;

    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use crypto::x509::CanonicalDistinguishedName;
    use jwt::error::JwtParseError;
    use serde_json::json;
    use token_status_list::status_list::StatusList;
    use token_status_list::status_list::StatusType;
    use token_status_list::status_list_token::StatusListToken;
    use token_status_list::verification::client::StatusListClient;
    use token_status_list::verification::client::StatusListClientError;
    use token_status_list::verification::verifier::RevocationVerifier;
    use url::Url;
    use utils::generator::mock::MockTimeGenerator;

    use super::super::RegistrationCertificateValidationError;
    use super::super::UncheckedRegistrationCertificate;
    use super::super::test::STATUS_LIST_URI;
    use super::super::test::legal_person_access_certificate_subject;
    use super::super::test::valid_payload;
    use super::super::test::valid_payload_json;
    use super::super::test::validation_time;
    use super::RegistrationCertificateStatus;
    use super::RegistrationCertificateStatusValidationError;

    #[derive(Debug)]
    struct StaticStatusListClient(StatusListToken);

    impl StatusListClient for StaticStatusListClient {
        async fn fetch(&self, _url: Url) -> Result<StatusListToken, StatusListClientError> {
            Ok(self.0.clone())
        }
    }

    #[derive(Debug)]
    struct FailingStatusListClient;

    impl StatusListClient for FailingStatusListClient {
        async fn fetch(&self, _url: Url) -> Result<StatusListToken, StatusListClientError> {
            Err(StatusListClientError::JwtParsing(JwtParseError::MissingKid))
        }
    }

    struct StatusValidationContext {
        verifier: RevocationVerifier<StaticStatusListClient>,
        trust_anchors: TrustAnchors,
        signing_certificate_dn: CanonicalDistinguishedName,
        time: MockTimeGenerator,
    }

    async fn status_validation_context(status: StatusType, status_list_length: usize) -> StatusValidationContext {
        let ca = Ca::generate_mock();
        let status_list_signer = ca.generate_issuer_status_list_mock().unwrap();
        let registration_certificate_signer = ca.generate_issuer_mock().unwrap();
        let mut status_list = StatusList::new(status_list_length);
        if status != StatusType::Valid {
            assert_eq!(status_list.insert(0, status), None);
        }
        let status_list_token = StatusListToken::builder(STATUS_LIST_URI.parse().unwrap(), status_list.pack())
            .sign(&status_list_signer)
            .await
            .unwrap();
        let time = MockTimeGenerator::default();

        StatusValidationContext {
            verifier: RevocationVerifier::new(
                Arc::new(StaticStatusListClient(status_list_token)),
                0,
                Duration::ZERO,
                Duration::ZERO,
                time.clone(),
            ),
            trust_anchors: TrustAnchors::from(&ca),
            signing_certificate_dn: registration_certificate_signer
                .certificate()
                .to_canonical_distinguished_name()
                .unwrap(),
            time,
        }
    }

    #[test]
    fn reject_non_numeric_status_list_index() {
        for index in ["", "+42", "-42", "42.0", "forty-two"] {
            let result = serde_json::from_value::<RegistrationCertificateStatus>(json!({
                "idx": index,
                "uri": STATUS_LIST_URI,
            }));
            assert!(result.is_err(), "`{index}` should not be a numeric string");
        }
    }

    #[test]
    fn reject_non_normative_status_shapes_during_structural_validation() {
        let mut nested = valid_payload_json();
        nested["status"] = json!({
            "status_list": {
                "idx": 0,
                "uri": STATUS_LIST_URI,
            }
        });
        let nested: UncheckedRegistrationCertificate = serde_json::from_value(nested).unwrap();
        assert_matches!(
            nested.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
            Err(RegistrationCertificateValidationError::InvalidStatusWireFormat)
        );

        let mut integer = valid_payload_json();
        integer["status"] = json!({
            "idx": 0,
            "uri": STATUS_LIST_URI,
        });
        let integer: UncheckedRegistrationCertificate = serde_json::from_value(integer).unwrap();
        assert_matches!(
            integer.validate_structure(&legal_person_access_certificate_subject(), validation_time()),
            Err(RegistrationCertificateValidationError::InvalidStatusWireFormat)
        );
    }

    #[tokio::test]
    async fn accept_registration_certificate_with_valid_referenced_status() {
        let context = status_validation_context(StatusType::Valid, 8).await;
        let certificate = valid_payload()
            .validate_structure(&legal_person_access_certificate_subject(), validation_time())
            .unwrap()
            .validate_status(
                &context.verifier,
                &context.trust_anchors,
                context.signing_certificate_dn,
                &context.time,
            )
            .await
            .unwrap();

        assert_eq!(certificate.payload().id.as_deref(), Some("wrprc-example-1"));
    }

    #[tokio::test]
    async fn reject_registration_certificate_with_revoked_status() {
        let context = status_validation_context(StatusType::Invalid, 8).await;
        let result = valid_payload()
            .validate_structure(&legal_person_access_certificate_subject(), validation_time())
            .unwrap()
            .validate_status(
                &context.verifier,
                &context.trust_anchors,
                context.signing_certificate_dn,
                &context.time,
            )
            .await;

        assert_matches!(result, Err(RegistrationCertificateStatusValidationError::NotValid));
    }

    #[tokio::test]
    async fn reject_registration_certificate_with_out_of_bounds_status_index() {
        let context = status_validation_context(StatusType::Valid, 8).await;
        let mut payload = valid_payload_json();
        payload["status"]["idx"] = json!("8");
        let payload: UncheckedRegistrationCertificate = serde_json::from_value(payload).unwrap();
        let result = payload
            .validate_structure(&legal_person_access_certificate_subject(), validation_time())
            .unwrap()
            .validate_status(
                &context.verifier,
                &context.trust_anchors,
                context.signing_certificate_dn,
                &context.time,
            )
            .await;

        assert_matches!(
            result,
            Err(RegistrationCertificateStatusValidationError::InvalidStatusListReference)
        );
    }

    #[tokio::test]
    async fn reject_status_list_not_signed_under_registration_certificate_trust_anchor() {
        let context = status_validation_context(StatusType::Valid, 8).await;
        let untrusted_ca = Ca::generate_mock();
        let result = valid_payload()
            .validate_structure(&legal_person_access_certificate_subject(), validation_time())
            .unwrap()
            .validate_status(
                &context.verifier,
                &TrustAnchors::from(&untrusted_ca),
                context.signing_certificate_dn,
                &context.time,
            )
            .await;

        assert_matches!(
            result,
            Err(RegistrationCertificateStatusValidationError::InvalidStatusListReference)
        );
    }

    #[tokio::test]
    async fn reject_registration_certificate_when_status_cannot_be_determined() {
        let context = status_validation_context(StatusType::Valid, 8).await;
        let verifier = RevocationVerifier::new(
            Arc::new(FailingStatusListClient),
            0,
            Duration::ZERO,
            Duration::ZERO,
            context.time.clone(),
        );
        let result = valid_payload()
            .validate_structure(&legal_person_access_certificate_subject(), validation_time())
            .unwrap()
            .validate_status(
                &verifier,
                &context.trust_anchors,
                context.signing_certificate_dn,
                &context.time,
            )
            .await;

        assert_matches!(result, Err(RegistrationCertificateStatusValidationError::Undetermined));
    }
}
