use attestation_data::auth::Organization;
use attestation_data::registration_certificate::RegistrationCertificateEnvelope;
use attestation_data::registration_certificate::StatusValidatedRegistrationCertificate;
use crypto::x509::BorrowingCertificate;
use derive_more::Debug;
use serde::Deserialize;
use serde::Serialize;
use serde_with::TryFromIntoRef;
use serde_with::base64::Base64;
use serde_with::serde_as;

/// Issuer information captured after validating signed issuer metadata and its bound WRPRC.
///
/// Serialization is for trusted wallet storage, including an authorization session interrupted by an app restart.
/// This is a snapshot of validation at discovery time, not proof of current validity or issuer authorization.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IssuerRegistration {
    #[debug(skip)]
    #[serde_as(as = "TryFromIntoRef<String>")]
    registration_certificate: RegistrationCertificateEnvelope,
    #[debug(skip)]
    #[serde_as(as = "Base64")]
    access_certificate: BorrowingCertificate,
    organization: Box<Organization>,
}

impl IssuerRegistration {
    pub(super) fn new(
        registration_certificate: RegistrationCertificateEnvelope,
        access_certificate: BorrowingCertificate,
        validated: &StatusValidatedRegistrationCertificate,
    ) -> Self {
        Self {
            registration_certificate,
            access_certificate,
            organization: Box::new(Organization::from(validated)),
        }
    }

    pub fn organization(&self) -> &Organization {
        &self.organization
    }

    pub fn registration_certificate(&self) -> &RegistrationCertificateEnvelope {
        &self.registration_certificate
    }

    pub fn access_certificate(&self) -> &BorrowingCertificate {
        &self.access_certificate
    }
}

#[cfg(any(test, feature = "mock"))]
impl IssuerRegistration {
    pub fn new_mock() -> Self {
        use std::sync::Arc;

        use attestation_data::registration_certificate::mock::MockRegistrationCertificate;
        use attestation_types::credential_format::Format;
        use attestation_types::credential_kind::CredentialKind;
        use crypto::server_keys::generate::Ca;
        use futures::FutureExt;
        use token_status_list::verification::verifier::RevocationVerifier;
        use utils::generator::TimeGenerator;

        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let access_certificate = ca.generate_wrpac_issuer_mock().unwrap().certificate().clone();
        let certificate = MockRegistrationCertificate::new_issuer(
            &access_certificate,
            [CredentialKind::new(Format::SdJwt, "urn:example:credential".to_string())],
        );
        let envelope = RegistrationCertificateEnvelope::try_from(certificate.certificate.as_slice()).unwrap();
        let verifier = RevocationVerifier::new_with_defaults(Arc::new(certificate.status_list_client), TimeGenerator);
        let validated = crate::registration_certificate::validate_registration_certificate(
            &envelope,
            &access_certificate,
            &certificate.trust_anchors,
            &verifier,
            &TimeGenerator,
        )
        .now_or_never()
        .unwrap()
        .unwrap();
        Self::new(envelope, access_certificate, &validated)
    }
}
