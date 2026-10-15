use attestation_types::status_claim::StatusClaim;
use attestation_types::status_claim::StatusListClaim;
use chrono::DateTime;
use chrono::Utc;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::CanonicalDistinguishedName;
use serde::Deserialize;
use token_status_list::verification::client::StatusListClient;
use token_status_list::verification::verifier::RevocationStatus;
use token_status_list::verification::verifier::RevocationVerifier;
use url::Url;
use utils::generator::Generator;

use super::payload::Subject;
use super::payload::UncheckedRegistrationCertificate;
use super::validation::BoundRegistrationCertificate;

#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum StatusWireFormat {
    NestedStatusList(StatusClaim),
    Direct { idx: StatusListIndex, uri: Url },
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum StatusListIndex {
    String(String),
    Integer(u32),
}

#[derive(Debug, thiserror::Error)]
pub enum StatusListIndexError {
    #[error("status list index `{0}` is not a numeric string")]
    NotNumeric(String),

    #[error("status list index `{0}` does not fit in a 32-bit unsigned integer")]
    TooLarge(String),
}

/// ETSI TS 119 475 is inconsistent about the shape of `status`. GEN-6.2.6.1-04 requires an object with `idx` as
/// a numeric string, while REV-6.2.6.2-03 refers to the IETF Token Status List, which nests the reference in
/// `status_list` with an integer `idx`. The Annex C example and the EUDI reference implementation use the latter.
/// All of these shapes are accepted.
impl TryFrom<StatusWireFormat> for StatusListClaim {
    type Error = StatusListIndexError;

    fn try_from(status: StatusWireFormat) -> Result<Self, Self::Error> {
        match status {
            StatusWireFormat::NestedStatusList(StatusClaim::StatusList(claim)) => Ok(claim),
            StatusWireFormat::Direct {
                idx: StatusListIndex::Integer(idx),
                uri,
            } => Ok(Self { idx, uri }),
            StatusWireFormat::Direct {
                idx: StatusListIndex::String(idx),
                uri,
            } => {
                if idx.is_empty() || !idx.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(StatusListIndexError::NotNumeric(idx));
                }
                let idx = idx.parse().map_err(|_| StatusListIndexError::TooLarge(idx))?;
                Ok(Self { idx, uri })
            }
        }
    }
}

impl BoundRegistrationCertificate {
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
                StatusClaim::StatusList(self.payload().status.clone()),
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

/// A WRPAC-bound, time-checked registration-certificate payload whose referenced status-list entry is valid.
///
/// Registration-certificate header and signature validation are performed by
/// [`super::verify_registration_certificate_envelope`].
pub struct StatusValidatedRegistrationCertificate(BoundRegistrationCertificate);

impl StatusValidatedRegistrationCertificate {
    pub fn payload(&self) -> &UncheckedRegistrationCertificate {
        self.0.payload()
    }

    pub fn subject(&self) -> &Subject {
        self.0.subject()
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

    use attestation_types::status_claim::StatusListClaim;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use crypto::x509::CanonicalDistinguishedName;
    use jwt::error::JwtParseError;
    use rstest::rstest;
    use serde_json::Value;
    use serde_json::json;
    use token_status_list::status_list::StatusList;
    use token_status_list::status_list::StatusType;
    use token_status_list::status_list_token::StatusListToken;
    use token_status_list::verification::client::StatusListClient;
    use token_status_list::verification::client::StatusListClientError;
    use token_status_list::verification::verifier::RevocationVerifier;
    use url::Url;
    use utils::generator::mock::MockTimeGenerator;

    use super::super::ParsedRegistrationCertificate;
    use super::super::mock::STATUS_LIST_URI;
    use super::super::mock::StaticStatusListClient;
    use super::super::test::legal_person_access_certificate_subject;
    use super::super::test::valid_parsed_payload;
    use super::super::test::valid_payload_json;
    use super::super::test::validation_time;
    use super::RegistrationCertificateStatusValidationError;
    use super::StatusListIndexError;
    use super::StatusWireFormat;

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
            trust_anchors: ca.to_trust_anchors(),
            signing_certificate_dn: registration_certificate_signer
                .certificate()
                .to_canonical_distinguished_name()
                .unwrap(),
            time,
        }
    }

    #[test]
    fn reject_invalid_status_list_index() {
        let status_with_index = |index: &str| -> StatusWireFormat {
            serde_json::from_value(json!({ "idx": index, "uri": STATUS_LIST_URI })).unwrap()
        };

        for index in ["", "+42", "-42", "42.0", "forty-two"] {
            assert_matches!(
                StatusListClaim::try_from(status_with_index(index)),
                Err(StatusListIndexError::NotNumeric(_)),
                "`{index}` should not be a numeric string"
            );
        }
        assert_matches!(
            StatusListClaim::try_from(status_with_index("4294967296")),
            Err(StatusListIndexError::TooLarge(_))
        );
    }

    #[rstest]
    #[case::direct_numeric_string(json!({ "idx": "0", "uri": STATUS_LIST_URI }))]
    #[case::direct_integer(json!({ "idx": 0, "uri": STATUS_LIST_URI }))]
    #[case::nested_token_status_list(json!({ "status_list": { "idx": 0, "uri": STATUS_LIST_URI } }))]
    #[tokio::test]
    async fn check_referenced_status_for_each_status_shape(#[case] status: Value) {
        let context = status_validation_context(StatusType::Invalid, 8).await;
        let mut payload = valid_payload_json();
        payload["status"] = status;
        let payload: ParsedRegistrationCertificate = serde_json::from_value(payload).unwrap();
        let result = payload
            .validate_binding_and_time(&legal_person_access_certificate_subject(), validation_time())
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
    async fn accept_registration_certificate_with_valid_referenced_status() {
        let context = status_validation_context(StatusType::Valid, 8).await;
        let certificate = valid_parsed_payload()
            .validate_binding_and_time(&legal_person_access_certificate_subject(), validation_time())
            .unwrap()
            .validate_status(
                &context.verifier,
                &context.trust_anchors,
                context.signing_certificate_dn,
                &context.time,
            )
            .await
            .unwrap();

        assert_eq!(certificate.payload().sub, "LEIXG-529900T8BM49AURSDO55");
    }

    #[tokio::test]
    async fn reject_registration_certificate_with_revoked_status() {
        let context = status_validation_context(StatusType::Invalid, 8).await;
        let result = valid_parsed_payload()
            .validate_binding_and_time(&legal_person_access_certificate_subject(), validation_time())
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
        let payload: ParsedRegistrationCertificate = serde_json::from_value(payload).unwrap();
        let result = payload
            .validate_binding_and_time(&legal_person_access_certificate_subject(), validation_time())
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
        let result = valid_parsed_payload()
            .validate_binding_and_time(&legal_person_access_certificate_subject(), validation_time())
            .unwrap()
            .validate_status(
                &context.verifier,
                &untrusted_ca.to_trust_anchors(),
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
        let result = valid_parsed_payload()
            .validate_binding_and_time(&legal_person_access_certificate_subject(), validation_time())
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
