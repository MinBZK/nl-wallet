use attestation_data::auth::Organization;
use attestation_data::registration_certificate::RegistrationCertificateEnvelope;
use attestation_data::registration_certificate::StatusValidatedRegistrationCertificate;
use crypto::x509::BorrowingCertificate;
use derive_more::Debug;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de::Error;
use serde_with::TryFromIntoRef;
use serde_with::base64::Base64;
use serde_with::serde_as;

/// Issuer information captured after validating signed issuer metadata and its bound WRPRC.
///
/// Serialization is for trusted wallet storage, including an authorization session interrupted by an app restart.
/// This is a snapshot of validation at discovery time, not proof of current validity or issuer authorization.
#[serde_as]
#[derive(Clone, Debug, Serialize)]
pub struct IssuerRegistration {
    #[debug(skip)]
    #[serde_as(as = "TryFromIntoRef<String>")]
    registration_certificate: RegistrationCertificateEnvelope,
    #[debug(skip)]
    #[serde_as(as = "Base64")]
    access_certificate: BorrowingCertificate,
    #[serde(skip_serializing)]
    organization: Box<Organization>,
}

impl<'de> Deserialize<'de> for IssuerRegistration {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[serde_as]
        #[derive(Deserialize)]
        struct Stored {
            #[serde_as(as = "TryFromIntoRef<String>")]
            registration_certificate: RegistrationCertificateEnvelope,
            #[serde_as(as = "Base64")]
            access_certificate: BorrowingCertificate,
        }

        let stored = Stored::deserialize(deserializer)?;
        // Trusted wallet storage: recover the organization without requiring the certificate to still be valid.
        let parsed = match &stored.registration_certificate {
            RegistrationCertificateEnvelope::Jwt(jwt) => jwt
                .dangerous_parse_unverified()
                .map(|(_, payload)| payload)
                .map_err(D::Error::custom)?,
            RegistrationCertificateEnvelope::Cwt(cwt) => cwt.dangerous_parse_unverified().map_err(D::Error::custom)?,
        };

        Ok(Self {
            registration_certificate: stored.registration_certificate,
            access_certificate: stored.access_certificate,
            organization: Box::new(Organization::from(&parsed)),
        })
    }
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

#[cfg(test)]
mod tests {
    use attestation_data::registration_certificate::mock::MockRegistrationCertificateAuthority;
    use attestation_data::registration_certificate::mock::RegistrationCertificateFixture;
    use attestation_data::registration_certificate::mock::issuer_registration_certificate_payload;
    use attestation_types::credential_format::Format;
    use attestation_types::credential_kind::CredentialKind;
    use chrono::Duration;
    use chrono::Utc;
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    #[rstest]
    #[case::jwt(MockRegistrationCertificateAuthority::sign_jwt)]
    #[case::cwt(MockRegistrationCertificateAuthority::sign_cwt)]
    fn test_persist_and_restore(
        #[case] sign: fn(&MockRegistrationCertificateAuthority, &RegistrationCertificateFixture) -> Vec<u8>,
    ) {
        let mut registration = IssuerRegistration::new_mock();
        let authority = MockRegistrationCertificateAuthority::new();
        let mut payload = issuer_registration_certificate_payload(
            registration.access_certificate(),
            [CredentialKind::new(Format::SdJwt, "urn:example:credential".to_string())],
        );
        // Stored issuer information must remain available after its WRPRC expires.
        payload.0["iat"] = json!((Utc::now() - Duration::hours(2)).timestamp());
        payload.0["exp"] = json!((Utc::now() - Duration::hours(1)).timestamp());
        registration.registration_certificate =
            RegistrationCertificateEnvelope::try_from(sign(&authority, &payload).as_slice()).unwrap();

        let mut stored = serde_json::to_value(&registration).unwrap();
        assert_eq!(stored.as_object().unwrap().len(), 2);
        assert!(stored.get("organization").is_none());
        let restored: IssuerRegistration = serde_json::from_value(stored.clone()).unwrap();
        assert_eq!(restored.organization(), registration.organization());
        assert_eq!(serde_json::to_value(restored).unwrap(), stored);

        // Reject malformed stored payloads instead of accepting an independent organization snapshot.
        payload.0["sub"] = json!(null);
        let invalid = RegistrationCertificateEnvelope::try_from(sign(&authority, &payload).as_slice()).unwrap();
        stored["registration_certificate"] = json!(String::try_from(&invalid).unwrap());
        stored["organization"] = serde_json::to_value(registration.organization()).unwrap();
        assert!(serde_json::from_value::<IssuerRegistration>(stored).is_err());
    }
}
