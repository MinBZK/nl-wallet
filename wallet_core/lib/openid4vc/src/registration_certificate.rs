use attestation_data::registration_certificate::RegistrationCertificateAuthorizationError;
use attestation_data::registration_certificate::RegistrationCertificateEnvelopeError;
use attestation_data::registration_certificate::RegistrationCertificateStatusValidationError;
use attestation_data::registration_certificate::RegistrationCertificateValidationError;
use attestation_data::registration_certificate::StatusValidatedRegistrationCertificate;
use attestation_data::registration_certificate::verify_registration_certificate_envelope;
use attestation_data::x509::RelyingParty;
use attestation_data::x509::RelyingPartyError;
use base64::prelude::*;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::BorrowingCertificate;
use crypto::x509::DistinguishedNameError;
use token_status_list::verification::client::StatusListClient;
use token_status_list::verification::verifier::RevocationVerifier;
use utils::generator::Generator;

use crate::openid4vp::REGISTRATION_CERTIFICATE_FORMAT;
use crate::openid4vp::VerifierInfo;

#[derive(Debug, thiserror::Error)]
pub enum RegistrationCertificateError {
    #[error("Authorization Request does not contain a registration certificate")]
    Missing,
    #[error("Authorization Request contains multiple registration certificates")]
    Multiple,
    #[error("registration certificate is not valid base64url: {0}")]
    Base64(#[source] base64::DecodeError),
    #[error("registration certificate has an invalid envelope: {0}")]
    Envelope(#[source] RegistrationCertificateEnvelopeError),
    #[error("could not parse access-certificate subject: {0}")]
    AccessCertificateSubject(#[source] DistinguishedNameError),
    #[error("access certificate does not contain a relying-party subject: {0}")]
    RelyingParty(#[source] RelyingPartyError),
    #[error("registration certificate has invalid contents: {0}")]
    Structure(#[source] RegistrationCertificateValidationError),
    #[error("registration certificate has invalid status: {0}")]
    Status(#[source] RegistrationCertificateStatusValidationError),
    #[error("registration certificate does not authorize the request: {0}")]
    Authorization(#[source] RegistrationCertificateAuthorizationError),
}

pub async fn validate_registration_certificate<C>(
    verifier_info: Option<&[VerifierInfo]>,
    access_certificate: &BorrowingCertificate,
    registration_certificate_trust_anchors: &TrustAnchors,
    revocation_verifier: &RevocationVerifier<C>,
    time: &impl Generator<chrono::DateTime<chrono::Utc>>,
) -> Result<StatusValidatedRegistrationCertificate, RegistrationCertificateError>
where
    C: StatusListClient,
{
    let registration_certificates = verifier_info
        .unwrap_or_default()
        .iter()
        .filter(|info| info.format == REGISTRATION_CERTIFICATE_FORMAT)
        .collect::<Vec<_>>();
    let registration_certificate = match registration_certificates.as_slice() {
        [] => return Err(RegistrationCertificateError::Missing),
        [registration_certificate] => registration_certificate,
        _ => return Err(RegistrationCertificateError::Multiple),
    };

    let certificate_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(&registration_certificate.data)
        .map_err(RegistrationCertificateError::Base64)?;
    let envelope =
        verify_registration_certificate_envelope(&certificate_bytes, registration_certificate_trust_anchors, time)
            .map_err(RegistrationCertificateError::Envelope)?;
    let (payload, signing_certificate_dn) = envelope.into_parts();
    let access_certificate_subject = access_certificate
        .to_distinguished_name()
        .map_err(RegistrationCertificateError::AccessCertificateSubject)?;
    let access_subject =
        RelyingParty::try_from(access_certificate_subject).map_err(RegistrationCertificateError::RelyingParty)?;

    let certificate = payload
        .validate_structure(&access_subject, time.generate())
        .map_err(RegistrationCertificateError::Structure)?;
    certificate
        .validate_status(
            revocation_verifier,
            registration_certificate_trust_anchors,
            signing_certificate_dn,
            time,
        )
        .await
        .map_err(RegistrationCertificateError::Status)
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::sync::Arc;

    use attestation_data::registration_certificate::RegistrationCertificateAuthorizationError;
    use attestation_data::registration_certificate::RegistrationCertificateStatusValidationError;
    use attestation_data::registration_certificate::RegistrationCertificateValidationError;
    use attestation_data::registration_certificate::mock::MockRegistrationCertificateAuthority;
    use attestation_data::registration_certificate::mock::StaticStatusListClient;
    use base64::prelude::*;
    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use crypto::x509::BorrowingCertificate;
    use crypto::x509::DistinguishedName;
    use crypto::x509::NO_SAN;
    use dcql::Query;
    use rstest::rstest;
    use token_status_list::status_list::StatusType;
    use token_status_list::verification::verifier::RevocationVerifier;
    use utils::generator::TimeGenerator;

    use super::RegistrationCertificateError;
    use super::validate_registration_certificate;
    use crate::openid4vp::VerifierInfo;

    enum EnvelopeFormat {
        Jwt,
        Cwt,
    }

    enum MissingRegistrationCertificate {
        NoVerifierInfo,
        OtherVerifierInfo,
    }

    fn setup(status: StatusType) -> (KeyPair, Query, MockRegistrationCertificateAuthority) {
        let access_certificate_ca = Ca::generate_wrpac_mock_ca().unwrap();
        let access_key_pair = access_certificate_ca.generate_wrpac_verifier_mock().unwrap();
        let query = Query::new_mock_mdoc_pid_example();
        let authority = MockRegistrationCertificateAuthority::new_with_status(status);

        (access_key_pair, query, authority)
    }

    fn registration_certificate_info(certificate: Vec<u8>) -> VerifierInfo {
        VerifierInfo::registration_certificate(BASE64_URL_SAFE_NO_PAD.encode(certificate))
    }

    async fn validate(
        verifier_info: Option<&[VerifierInfo]>,
        query: &Query,
        access_certificate: &BorrowingCertificate,
        trust_anchors: &TrustAnchors,
        status_list_client: StaticStatusListClient,
    ) -> Result<(), RegistrationCertificateError> {
        let revocation_verifier = RevocationVerifier::new_without_caching(Arc::new(status_list_client));

        let certificate = validate_registration_certificate(
            verifier_info,
            access_certificate,
            trust_anchors,
            &revocation_verifier,
            &TimeGenerator,
        )
        .await?;

        certificate
            .validate_query_authorization(query)
            .map_err(RegistrationCertificateError::Authorization)
    }

    #[rstest]
    #[case::jwt(EnvelopeFormat::Jwt)]
    #[case::cwt(EnvelopeFormat::Cwt)]
    #[tokio::test]
    async fn validate_registration_certificate_envelope(#[case] format: EnvelopeFormat) {
        let (access_key_pair, query, authority) = setup(StatusType::Valid);
        let certificate = match format {
            EnvelopeFormat::Jwt => authority.issue_jwt(access_key_pair.certificate(), query.clone()),
            EnvelopeFormat::Cwt => authority.issue_cwt(access_key_pair.certificate(), query.clone()),
        };
        let verifier_info = [registration_certificate_info(certificate)];

        validate(
            Some(&verifier_info),
            &query,
            access_key_pair.certificate(),
            &authority.trust_anchors,
            authority.status_list_client.clone(),
        )
        .await
        .unwrap();
    }

    #[rstest]
    #[case::no_verifier_info(MissingRegistrationCertificate::NoVerifierInfo)]
    #[case::other_verifier_info(MissingRegistrationCertificate::OtherVerifierInfo)]
    #[tokio::test]
    async fn reject_missing_registration_certificate(#[case] missing: MissingRegistrationCertificate) {
        let (access_key_pair, query, authority) = setup(StatusType::Valid);
        let verifier_info = match missing {
            MissingRegistrationCertificate::NoVerifierInfo => None,
            MissingRegistrationCertificate::OtherVerifierInfo => Some(vec![VerifierInfo {
                format: "other_format".to_string(),
                data: "other_data".to_string(),
            }]),
        };

        let error = validate(
            verifier_info.as_deref(),
            &query,
            access_key_pair.certificate(),
            &authority.trust_anchors,
            authority.status_list_client.clone(),
        )
        .await
        .unwrap_err();

        assert_matches!(error, RegistrationCertificateError::Missing);
    }

    #[tokio::test]
    async fn reject_multiple_registration_certificates() {
        let (access_key_pair, query, authority) = setup(StatusType::Valid);
        let certificate = authority.issue_jwt(access_key_pair.certificate(), query.clone());
        let verifier_info = [
            registration_certificate_info(certificate.clone()),
            registration_certificate_info(certificate),
        ];

        let error = validate(
            Some(&verifier_info),
            &query,
            access_key_pair.certificate(),
            &authority.trust_anchors,
            authority.status_list_client.clone(),
        )
        .await
        .unwrap_err();

        assert_matches!(error, RegistrationCertificateError::Multiple);
    }

    #[tokio::test]
    async fn accept_single_registration_certificate_among_other_verifier_info() {
        let (access_key_pair, query, authority) = setup(StatusType::Valid);
        let certificate = authority.issue_jwt(access_key_pair.certificate(), query.clone());
        let verifier_info = [
            VerifierInfo {
                format: "first_other_format".to_string(),
                data: "first_other_data".to_string(),
            },
            registration_certificate_info(certificate),
            VerifierInfo {
                format: "second_other_format".to_string(),
                data: "second_other_data".to_string(),
            },
        ];

        validate(
            Some(&verifier_info),
            &query,
            access_key_pair.certificate(),
            &authority.trust_anchors,
            authority.status_list_client.clone(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn reject_invalid_base64url_registration_certificate() {
        let (access_key_pair, query, authority) = setup(StatusType::Valid);
        let verifier_info = [VerifierInfo::registration_certificate("***".to_string())];

        let error = validate(
            Some(&verifier_info),
            &query,
            access_key_pair.certificate(),
            &authority.trust_anchors,
            authority.status_list_client.clone(),
        )
        .await
        .unwrap_err();

        assert_matches!(error, RegistrationCertificateError::Base64(_));
    }

    #[tokio::test]
    async fn reject_registration_certificate_from_untrusted_signer() {
        let (access_key_pair, query, authority) = setup(StatusType::Valid);
        let certificate = authority.issue_jwt(access_key_pair.certificate(), query.clone());
        let verifier_info = [registration_certificate_info(certificate)];
        let untrusted_ca = Ca::generate_mock();

        let error = validate(
            Some(&verifier_info),
            &query,
            access_key_pair.certificate(),
            &TrustAnchors::from(&untrusted_ca),
            authority.status_list_client.clone(),
        )
        .await
        .unwrap_err();

        assert_matches!(error, RegistrationCertificateError::Envelope(_));
    }

    #[tokio::test]
    async fn reject_registration_certificate_with_mismatching_access_certificate_subject() {
        let (access_key_pair, query, authority) = setup(StatusType::Valid);
        let certificate = authority.issue_jwt(access_key_pair.certificate(), query.clone());
        let verifier_info = [registration_certificate_info(certificate)];
        let mismatching_access_certificate = Ca::generate_wrpac_mock_ca()
            .unwrap()
            .generate_key_pair(
                DistinguishedName::new_legal_person(
                    "Other relying party".to_string(),
                    "NL".to_string(),
                    "Other relying party".to_string(),
                    "other-organization-identifier".to_string(),
                ),
                Default::default(),
                NO_SAN,
            )
            .unwrap();

        let error = validate(
            Some(&verifier_info),
            &query,
            mismatching_access_certificate.certificate(),
            &authority.trust_anchors,
            authority.status_list_client.clone(),
        )
        .await
        .unwrap_err();

        assert_matches!(
            error,
            RegistrationCertificateError::Structure(
                RegistrationCertificateValidationError::SubjectIdentifierMismatch {
                    field: "organizationIdentifier"
                }
            )
        );
    }

    #[tokio::test]
    async fn reject_revoked_registration_certificate() {
        let (access_key_pair, query, authority) = setup(StatusType::Invalid);
        let certificate = authority.issue_jwt(access_key_pair.certificate(), query.clone());
        let verifier_info = [registration_certificate_info(certificate)];

        let error = validate(
            Some(&verifier_info),
            &query,
            access_key_pair.certificate(),
            &authority.trust_anchors,
            authority.status_list_client.clone(),
        )
        .await
        .unwrap_err();

        assert_matches!(
            error,
            RegistrationCertificateError::Status(RegistrationCertificateStatusValidationError::NotValid)
        );
    }

    #[tokio::test]
    async fn reject_query_not_authorized_by_registration_certificate() {
        let (access_key_pair, authorized_query, authority) = setup(StatusType::Valid);
        let certificate = authority.issue_jwt(access_key_pair.certificate(), authorized_query);
        let verifier_info = [registration_certificate_info(certificate)];
        let unauthorized_query = Query::new_mock_sd_jwt_pid_example();

        let error = validate(
            Some(&verifier_info),
            &unauthorized_query,
            access_key_pair.certificate(),
            &authority.trust_anchors,
            authority.status_list_client.clone(),
        )
        .await
        .unwrap_err();

        assert_matches!(
            error,
            RegistrationCertificateError::Authorization(
                RegistrationCertificateAuthorizationError::UnauthorizedCredential(_)
            )
        );
    }
}
