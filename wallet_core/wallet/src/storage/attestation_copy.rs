use std::borrow::Cow;

use uuid::Uuid;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::credential_payload::CredentialPayload;
use crypto::x509::BorrowingCertificateExtension;
use mdoc::holder::Mdoc;
use sd_jwt::sd_jwt::VerifiedSdJwt;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;

use crate::AttestationIdentity;
use crate::AttestationPresentation;

#[derive(Debug, Clone)]
pub enum StoredAttestationFormat {
    MsoMdoc { mdoc: Box<Mdoc> }, // TODO: Wrap in similar VerifiedMdoc type (PVW-4132)
    SdJwt { sd_jwt: Box<VerifiedSdJwt> },
}

#[derive(Debug, Clone)]
pub struct StoredAttestationCopy {
    pub attestation_id: Uuid,
    pub attestation_copy_id: Uuid,
    pub attestation: StoredAttestationFormat,
    pub normalized_metadata: NormalizedTypeMetadata,
}

impl StoredAttestationFormat {
    fn issuer_registration(&self) -> IssuerRegistration {
        let issuer_certificate = match self {
            Self::MsoMdoc { mdoc } => Cow::Owned(
                mdoc.issuer_certificate()
                    .expect("a stored mdoc attestation should always contain an issuer certificate"),
            ),
            Self::SdJwt { sd_jwt } => Cow::Borrowed(sd_jwt.issuer_certificate()),
        };

        IssuerRegistration::from_certificate(issuer_certificate.as_ref())
            .expect("a stored attestation should always contain a valid IssuerRegistration")
            .expect("a stored attestation should always contain an IssuerRegistration")
    }
}

impl StoredAttestationCopy {
    pub fn into_credential_payload(self) -> CredentialPayload {
        match self.attestation {
            StoredAttestationFormat::MsoMdoc { mdoc } => {
                CredentialPayload::from_mdoc_unvalidated(*mdoc, &self.normalized_metadata)
                    .expect("conversion from stored mdoc attestation to CredentialPayload has been done before")
            }
            StoredAttestationFormat::SdJwt { sd_jwt } => CredentialPayload::from_verified_sd_jwt_unvalidated(*sd_jwt)
                .expect("conversion from stored SD-JWT attestation to CredentialPayload has been done before"),
        }
    }

    pub fn into_attestation_presentation(self) -> AttestationPresentation {
        let issuer_registration = self.attestation.issuer_registration();

        match self.attestation {
            StoredAttestationFormat::MsoMdoc { mdoc } => AttestationPresentation::create_from_mdoc(
                AttestationIdentity::Fixed {
                    id: self.attestation_id,
                },
                self.normalized_metadata,
                issuer_registration.organization,
                mdoc.issuer_signed.into_entries_by_namespace(),
            ),
            StoredAttestationFormat::SdJwt { sd_jwt } => {
                let credential_payload = CredentialPayload::from_verified_sd_jwt_unvalidated(*sd_jwt)
                    .expect("conversion from stored SD-JWT attestation to CredentialPayload has been done before");

                AttestationPresentation::create_from_attributes(
                    AttestationIdentity::Fixed {
                        id: self.attestation_id,
                    },
                    self.normalized_metadata,
                    issuer_registration.organization,
                    &credential_payload.previewable_payload.attributes,
                )
            }
        }
        .expect("conversion from stored SD-JWT attestation to AttestationPresentation has been done before")
    }
}
