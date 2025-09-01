use uuid::Uuid;

use attestation_data::auth::Organization;
use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::credential_payload::CredentialPayload;
use attestation_types::claim_path::ClaimPath;
use crypto::x509::BorrowingCertificateExtension;
use mdoc::IssuerSigned;
use mdoc::holder::Mdoc;
use mdoc::holder::disclosure::MissingAttributesError;
use mdoc::holder::disclosure::PartialMdoc;
use sd_jwt::sd_jwt::SdJwt;
use sd_jwt::sd_jwt::UnsignedSdJwtPresentation;
use sd_jwt::sd_jwt::VerifiedSdJwt;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use utils::vec_at_least::VecNonEmpty;

use crate::AttestationIdentity;
use crate::AttestationPresentation;

#[derive(Debug, thiserror::Error)]
pub enum PartialAttestationError {
    #[error("requested path not present in mdoc attestation: {0}")]
    MsoMdoc(#[from] MissingAttributesError),
    #[error("requested path not present in SD-JWT attestation: {0}")]
    SdJwt(#[from] sd_jwt::error::Error),
}

/// An attestation that is present in the wallet database, part of [`StoredAttestationCopy`].
#[derive(Debug, Clone)]
pub enum StoredAttestation {
    MsoMdoc { mdoc: Box<Mdoc> }, // TODO: Wrap in similar VerifiedMdoc type (PVW-4132)
    SdJwt { sd_jwt: Box<VerifiedSdJwt> },
}

/// An instance of an attestation copy as it is contained in the wallet database, which contains both the column id for
/// that particular copy and the foreign key id for its attestation parent.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(derive_more::Constructor))]
pub struct StoredAttestationCopy {
    pub(super) attestation_id: Uuid,
    pub(super) attestation_copy_id: Uuid,
    pub(super) attestation: StoredAttestation,
    pub(super) normalized_metadata: NormalizedTypeMetadata,
}

/// An attestation that is present in the wallet database and contains a subset of its original attributes, part of
/// [`DisclosableAttestation`].
#[derive(Debug, Clone)]
pub enum PartialAttestation {
    MsoMdoc { partial_mdoc: Box<PartialMdoc> },
    SdJwt { sd_jwt: Box<UnsignedSdJwtPresentation> },
}

/// A version of an attestation in the wallet database which contains a subset of its original attributes and whose
/// intended purpose is disclosure. It contains the column id for the copy in the database that is its source, the
/// partial attestation itself and an [`AttestationPresentation`] of the partial attestation that can be shown to the
/// user for approval. This type is always derived from [`StoredAttestationCopy`].
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(derive_more::Constructor))]
pub struct DisclosableAttestation {
    attestation_copy_id: Uuid,
    partial_attestation: PartialAttestation,
    presentation: AttestationPresentation,
}

fn credential_payload_from_sd_jwt(sd_jwt: &impl AsRef<SdJwt>) -> CredentialPayload {
    CredentialPayload::from_sd_jwt_unvalidated(sd_jwt.as_ref())
        .expect("a stored SD-JWT attestation should convert to CredentialPayload without errors")
}

fn attestation_presentation_from_issuer_signed(
    issuer_signed: IssuerSigned,
    attestation_id: Uuid,
    normalized_metadata: NormalizedTypeMetadata,
    issuer_organization: Organization,
) -> AttestationPresentation {
    AttestationPresentation::create_from_mdoc(
        AttestationIdentity::Fixed { id: attestation_id },
        normalized_metadata,
        issuer_organization,
        issuer_signed.into_entries_by_namespace(),
    )
    .expect("a stored mdoc attestation should convert to AttestationPresentation without errors")
}

fn attestation_presentation_from_sd_jwt(
    sd_jwt: &impl AsRef<SdJwt>,
    attestation_id: Uuid,
    normalized_metadata: NormalizedTypeMetadata,
    issuer_organization: Organization,
) -> AttestationPresentation {
    let credential_payload = credential_payload_from_sd_jwt(sd_jwt);

    AttestationPresentation::create_from_attributes(
        AttestationIdentity::Fixed { id: attestation_id },
        normalized_metadata,
        issuer_organization,
        &credential_payload.previewable_payload.attributes,
    )
    .expect("a stored SD-JWT attestation should convert to AttestationPresentation without errors")
}

impl StoredAttestation {
    /// Extract the [`IssuerRegistration`] from a stored attestation by parsing it from the issuer certificate.
    fn issuer_registration(&self) -> IssuerRegistration {
        let issuer_certificate = match self {
            Self::MsoMdoc { mdoc } => &mdoc
                .issuer_certificate()
                .expect("a stored mdoc attestation should always contain an issuer certificate"),
            Self::SdJwt { sd_jwt } => sd_jwt
                .as_ref()
                .as_ref()
                .issuer_certificate()
                .expect("a stored SD-JWT attestation should always contain an issuer certificate"),
        };

        // Note that this means that an `IssuerRegistration` should ALWAYS be backwards compatible.
        IssuerRegistration::from_certificate(issuer_certificate)
            .expect("a stored attestation should always contain a valid IssuerRegistration")
            .expect("a stored attestation should always contain an IssuerRegistration")
    }
}

impl StoredAttestationCopy {
    pub fn attestation_id(&self) -> Uuid {
        self.attestation_id
    }

    /// Checks if the stored attestation matches a list of claim paths.
    pub fn matches_requested_attributes<'a, 'b>(
        &'a self,
        claim_paths: impl IntoIterator<Item = &'b VecNonEmpty<ClaimPath>>,
    ) -> bool {
        match &self.attestation {
            StoredAttestation::MsoMdoc { mdoc } => mdoc.issuer_signed.matches_requested_attributes(claim_paths).is_ok(),
            StoredAttestation::SdJwt { sd_jwt } => {
                // Create a temporary CredentialPayload to check if the paths are all present.
                let credential_payload = credential_payload_from_sd_jwt(sd_jwt.as_ref());

                credential_payload
                    .previewable_payload
                    .attributes
                    .has_claim_paths(claim_paths)
            }
        }
    }

    /// Convert the stored attestation into a [`CredentialPayload`], skipping JSON schema validation.
    pub fn into_credential_payload(self) -> CredentialPayload {
        match self.attestation {
            StoredAttestation::MsoMdoc { mdoc } => {
                CredentialPayload::from_mdoc_unvalidated(*mdoc, &self.normalized_metadata)
                    .expect("a stored mdoc attestation should convert to CredentialPayload without errors")
            }
            StoredAttestation::SdJwt { sd_jwt } => credential_payload_from_sd_jwt(sd_jwt.as_ref()),
        }
    }

    /// Convert the stored attestation (which may contain a subset of the attributes)
    /// to an [`AttestationPresentation`] that can be displayed to the user.
    pub fn into_attestation_presentation(self) -> AttestationPresentation {
        let issuer_registration = self.attestation.issuer_registration();

        match self.attestation {
            StoredAttestation::MsoMdoc { mdoc } => attestation_presentation_from_issuer_signed(
                mdoc.issuer_signed,
                self.attestation_id,
                self.normalized_metadata,
                issuer_registration.organization,
            ),
            StoredAttestation::SdJwt { sd_jwt } => attestation_presentation_from_sd_jwt(
                sd_jwt.as_ref(),
                self.attestation_id,
                self.normalized_metadata,
                issuer_registration.organization,
            ),
        }
    }
}

impl PartialAttestation {
    fn try_new<'a>(
        attestation: StoredAttestation,
        claim_paths: impl IntoIterator<Item = &'a VecNonEmpty<ClaimPath>>,
    ) -> Result<Self, PartialAttestationError> {
        let partial_attestation = match attestation {
            StoredAttestation::MsoMdoc { mdoc } => {
                let partial_mdoc = Box::new(PartialMdoc::try_new(*mdoc, claim_paths)?);

                PartialAttestation::MsoMdoc { partial_mdoc }
            }
            StoredAttestation::SdJwt { sd_jwt } => {
                let unsigned_presentation = claim_paths
                    .into_iter()
                    .try_fold(sd_jwt.into_presentation_builder(), |builder, claim_path| {
                        builder.disclose(claim_path)
                    })?
                    .finish();

                PartialAttestation::SdJwt {
                    sd_jwt: Box::new(unsigned_presentation),
                }
            }
        };

        Ok(partial_attestation)
    }
}

impl DisclosableAttestation {
    pub fn try_new<'a>(
        attestation_copy: StoredAttestationCopy,
        claim_paths: impl IntoIterator<Item = &'a VecNonEmpty<ClaimPath>>,
    ) -> Result<Self, PartialAttestationError> {
        let StoredAttestationCopy {
            attestation_id,
            attestation_copy_id,
            attestation,
            normalized_metadata,
        } = attestation_copy;

        let issuer_registration = attestation.issuer_registration();
        let partial_attestation = PartialAttestation::try_new(attestation, claim_paths)?;

        let presentation = match &partial_attestation {
            PartialAttestation::MsoMdoc { partial_mdoc } => attestation_presentation_from_issuer_signed(
                partial_mdoc.issuer_signed.clone(),
                attestation_id,
                normalized_metadata,
                issuer_registration.organization,
            ),
            PartialAttestation::SdJwt { sd_jwt } => attestation_presentation_from_sd_jwt(
                sd_jwt.as_ref(),
                attestation_id,
                normalized_metadata,
                issuer_registration.organization,
            ),
        };

        let disclosable_attestation = Self {
            attestation_copy_id,
            partial_attestation,
            presentation,
        };

        Ok(disclosable_attestation)
    }

    pub fn attestation_copy_id(&self) -> Uuid {
        self.attestation_copy_id
    }

    pub fn partial_attestation(&self) -> &PartialAttestation {
        &self.partial_attestation
    }

    pub fn presentation(&self) -> &AttestationPresentation {
        &self.presentation
    }

    pub fn into_presentation(self) -> AttestationPresentation {
        self.presentation
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use futures::FutureExt;
    use itertools::Itertools;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use ssri::Integrity;
    use uuid::Uuid;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::constants::PID_ATTESTATION_TYPE;
    use attestation_data::constants::PID_BSN;
    use attestation_data::credential_payload::CredentialPayload;
    use attestation_data::x509::generate::mock::generate_issuer_mock;
    use attestation_types::claim_path::ClaimPath;
    use crypto::keys::WithIdentifier;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use sd_jwt::sd_jwt::VerifiedSdJwt;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_at_least::VecNonEmpty;

    use super::DisclosableAttestation;
    use super::StoredAttestation;
    use super::StoredAttestationCopy;

    static ATTESTATION_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);

    fn mdoc_stored_attestation_copy(issuer_keypair: &KeyPair) -> (StoredAttestationCopy, VecNonEmpty<ClaimPath>) {
        let credential_payload = CredentialPayload::nl_pid_example(&MockTimeGenerator::default());

        let mdoc_remote_key = MockRemoteEcdsaKey::new_random("identifier".to_string());
        let mdoc = credential_payload
            .previewable_payload
            .into_signed_mdoc_unverified::<MockRemoteEcdsaKey>(
                Integrity::from(""),
                mdoc_remote_key.identifier().to_string(),
                mdoc_remote_key.verifying_key(),
                issuer_keypair,
            )
            .now_or_never()
            .unwrap()
            .unwrap();

        let copy = StoredAttestationCopy {
            attestation_id: *ATTESTATION_ID,
            attestation_copy_id: Uuid::new_v4(),
            attestation: StoredAttestation::MsoMdoc { mdoc: Box::new(mdoc) },
            normalized_metadata: NormalizedTypeMetadata::nl_pid_example(),
        };

        let bsn_path = vec![
            ClaimPath::SelectByKey(PID_ATTESTATION_TYPE.to_string()),
            ClaimPath::SelectByKey(PID_BSN.to_string()),
        ]
        .try_into()
        .unwrap();

        (copy, bsn_path)
    }

    fn sd_jwt_stored_attestation_copy(issuer_keypair: &KeyPair) -> (StoredAttestationCopy, VecNonEmpty<ClaimPath>) {
        let credential_payload = CredentialPayload::nl_pid_example(&MockTimeGenerator::default());

        let holder_privkey = SigningKey::random(&mut OsRng);
        let sd_jwt = credential_payload
            .into_sd_jwt(
                &NormalizedTypeMetadata::nl_pid_example(),
                holder_privkey.verifying_key(),
                issuer_keypair,
            )
            .now_or_never()
            .unwrap()
            .unwrap();

        let copy = StoredAttestationCopy {
            attestation_id: *ATTESTATION_ID,
            attestation_copy_id: Uuid::new_v4(),
            attestation: StoredAttestation::SdJwt {
                sd_jwt: Box::new(VerifiedSdJwt::new_mock(sd_jwt)),
            },
            normalized_metadata: NormalizedTypeMetadata::nl_pid_example(),
        };

        let bsn_path = vec![ClaimPath::SelectByKey(PID_BSN.to_string())].try_into().unwrap();

        (copy, bsn_path)
    }

    #[test]
    fn test_stored_attestation_copy() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_registration = IssuerRegistration::new_mock();
        let issuer_keypair = generate_issuer_mock(&ca, issuer_registration.clone().into()).unwrap();

        let (full_presentations, disclosable_presentations): (Vec<_>, Vec<_>) = [
            mdoc_stored_attestation_copy(&issuer_keypair),
            sd_jwt_stored_attestation_copy(&issuer_keypair),
        ]
        .into_iter()
        .map(|(attestation_copy, bsn_path)| {
            // The retrieved `IssuerRegistration` matches the input.
            let full_issuer_registration = attestation_copy.attestation.issuer_registration();
            assert_eq!(full_issuer_registration, issuer_registration);

            // The attestation should contain the BSN attribute path.
            assert!(attestation_copy.matches_requested_attributes([&bsn_path]));

            // The attestation should not contain some incorrect path.
            let missing_path = vec![ClaimPath::SelectByKey("missing".to_string())].try_into().unwrap();
            assert!(!attestation_copy.matches_requested_attributes([&missing_path]));

            // The converted `AttestationPresentation` contains multiple attributes.
            let full_presentation = attestation_copy.clone().into_attestation_presentation();
            assert_eq!(full_presentation.attributes.len(), 4);

            // Selecting a particular attribute for disclosure should only succeed if the path exists.
            let disclosable_attestation = DisclosableAttestation::try_new(attestation_copy.clone(), [&bsn_path])
                .expect("converting the full attestation copy to on containing just the BSN should succeed");

            let _error = DisclosableAttestation::try_new(attestation_copy, [&missing_path])
                .expect_err("converting the full attestation copy to a partial one should not succeed");

            // The `DisclosableAttestation` contains only one attribute.
            assert_eq!(disclosable_attestation.presentation().attributes.len(), 1);

            (full_presentation, disclosable_attestation.into_presentation())
        })
        .unzip();

        // The full and partial `AttestationPresentation`s should be the same for both formats.
        assert!(full_presentations.iter().all_equal());
        assert!(disclosable_presentations.iter().all_equal())
    }
}
