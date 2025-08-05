use std::borrow::Cow;

use uuid::Uuid;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::credential_payload::CredentialPayload;
use attestation_types::claim_path::ClaimPath;
use crypto::x509::BorrowingCertificateExtension;
use mdoc::holder::Mdoc;
use sd_jwt::sd_jwt::SdJwt;
use sd_jwt::sd_jwt::UnsignedSdJwtPresentation;
use sd_jwt::sd_jwt::VerifiedSdJwt;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use utils::vec_at_least::VecNonEmpty;

use crate::AttestationIdentity;
use crate::AttestationPresentation;

#[derive(Debug, Clone)]
pub enum StoredAttestationFormat<S> {
    MsoMdoc { mdoc: Box<Mdoc> }, // TODO: Wrap in similar VerifiedMdoc type (PVW-4132)
    SdJwt { sd_jwt: Box<S> },
}

#[derive(Debug, Clone)]
pub struct StoredAttestationCopy<S> {
    pub attestation_id: Uuid,
    pub attestation_copy_id: Uuid,
    pub attestation: StoredAttestationFormat<S>,
    pub normalized_metadata: NormalizedTypeMetadata,
}

/// This is an alias for the type of `StoredAttestation` that is returned by the database and contains all of the
/// attributes issued. Its counterpart is `StoredAttestationCopy<UnsignedSdJwtPresentation>`, which contains a subset of
/// the issued attributes, which can then be used for selective disclosure.
pub type FullStoredAttestationCopy = StoredAttestationCopy<VerifiedSdJwt>;

fn credential_payload_from_sd_jwt(sd_jwt: &SdJwt) -> CredentialPayload {
    CredentialPayload::from_sd_jwt_unvalidated(sd_jwt)
        .expect("conversion from stored SD-JWT attestation to CredentialPayload has been done before")
}

impl<S> StoredAttestationFormat<S>
where
    S: AsRef<SdJwt>,
{
    /// Extract the [`IssuerRegistration`] from a stored attestation by parsing it from the issuer certificate.
    fn issuer_registration(&self) -> IssuerRegistration {
        let issuer_certificate = match self {
            Self::MsoMdoc { mdoc } => Cow::Owned(
                mdoc.issuer_certificate()
                    .expect("a stored mdoc attestation should always contain an issuer certificate"),
            ),
            Self::SdJwt { sd_jwt } => Cow::Borrowed(
                sd_jwt
                    .as_ref()
                    .as_ref()
                    .issuer_certificate()
                    .expect("a stored SD-JWT attestation should always contain an issuer certificate"),
            ),
        };

        // Note that this means that an `IssuerRegistration` should ALWAYS be backwards compatible.
        IssuerRegistration::from_certificate(issuer_certificate.as_ref())
            .expect("a stored attestation should always contain a valid IssuerRegistration")
            .expect("a stored attestation should always contain an IssuerRegistration")
    }
}

impl<S> StoredAttestationCopy<S>
where
    S: AsRef<SdJwt>,
{
    /// Convert the stored attestation (which may contain a subset of the attributes)
    /// to an [`AttestationPresentation`] that can be displayed to the user.
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
                let credential_payload = credential_payload_from_sd_jwt(sd_jwt.as_ref().as_ref());

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

impl StoredAttestationCopy<VerifiedSdJwt> {
    /// Convert the stored attestation into a [`CredentialPayload`], skipping JSON schema validation.
    pub fn into_credential_payload(self) -> CredentialPayload {
        match self.attestation {
            StoredAttestationFormat::MsoMdoc { mdoc } => {
                CredentialPayload::from_mdoc_unvalidated(*mdoc, &self.normalized_metadata)
                    .expect("conversion from stored mdoc attestation to CredentialPayload has been done before")
            }
            StoredAttestationFormat::SdJwt { sd_jwt } => credential_payload_from_sd_jwt(sd_jwt.as_ref().as_ref()),
        }
    }

    /// Checks if the stored attestation matches a list of claim paths.
    pub fn matches_requested_attributes<'a, 'b>(
        &'a self,
        claim_paths: impl IntoIterator<Item = &'b VecNonEmpty<ClaimPath>>,
    ) -> bool {
        match &self.attestation {
            StoredAttestationFormat::MsoMdoc { mdoc } => {
                mdoc.issuer_signed.matches_requested_attributes(claim_paths).is_ok()
            }
            StoredAttestationFormat::SdJwt { sd_jwt } => {
                // Create a temporary CredentialPayload to check if the paths are all present.
                let credential_payload = credential_payload_from_sd_jwt(sd_jwt.as_ref().as_ref());

                credential_payload
                    .previewable_payload
                    .attributes
                    .has_claim_paths(claim_paths)
            }
        }
    }

    /// Convert the stored attestation into one that is ready for disclosure by
    /// selecting a subset of its attributes, based on a list of claim paths.
    pub fn try_into_partial<'a>(
        self,
        claim_paths: impl IntoIterator<Item = &'a VecNonEmpty<ClaimPath>>,
    ) -> Result<StoredAttestationCopy<UnsignedSdJwtPresentation>, sd_jwt::error::Error> {
        let Self {
            attestation_id,
            attestation_copy_id,
            attestation,
            normalized_metadata,
        } = self;

        let attestation = match attestation {
            StoredAttestationFormat::MsoMdoc { mut mdoc } => {
                mdoc.issuer_signed = mdoc.issuer_signed.into_attribute_subset(claim_paths);

                StoredAttestationFormat::MsoMdoc { mdoc }
            }
            StoredAttestationFormat::SdJwt { sd_jwt } => {
                let presentation = claim_paths
                    .into_iter()
                    .try_fold(sd_jwt.into_presentation_builder(), |builder, claim_path| {
                        builder.disclose(claim_path)
                    })?
                    .finish();

                StoredAttestationFormat::SdJwt {
                    sd_jwt: Box::new(presentation),
                }
            }
        };

        let partial_copy = StoredAttestationCopy {
            attestation_id,
            attestation_copy_id,
            attestation,
            normalized_metadata,
        };

        Ok(partial_copy)
    }
}
