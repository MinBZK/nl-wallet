use uuid::Uuid;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::credential_payload::CredentialPayload;
use attestation_types::claim_path::ClaimPath;
use crypto::x509::BorrowingCertificateExtension;
use mdoc::holder::Mdoc;
use mdoc::holder::disclosure::MissingAttributesError;
use sd_jwt::sd_jwt::SdJwt;
use sd_jwt::sd_jwt::UnsignedSdJwtPresentation;
use sd_jwt::sd_jwt::VerifiedSdJwt;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use utils::vec_at_least::VecNonEmpty;

use crate::AttestationIdentity;
use crate::AttestationPresentation;

#[derive(Debug, thiserror::Error)]
pub enum StoredAttestationCopyPathError {
    #[error("requested path not present in mdoc attestion: {0}")]
    MsoMdoc(#[from] MissingAttributesError),
    #[error("requested path not present in SD-JWT attestion: {0}")]
    SdJwt(#[from] sd_jwt::error::Error),
}

#[derive(Debug, Clone)]
pub enum StoredAttestation<S> {
    MsoMdoc { mdoc: Box<Mdoc> }, // TODO: Wrap in similar VerifiedMdoc type (PVW-4132)
    SdJwt { sd_jwt: Box<S> },
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(derive_more::Constructor))]
pub struct StoredAttestationCopy<S> {
    pub(super) attestation_id: Uuid,
    pub(super) attestation_copy_id: Uuid,
    pub(super) attestation: StoredAttestation<S>,
    pub(super) normalized_metadata: NormalizedTypeMetadata,
}

/// This is an alias for the type of `StoredAttestation` that is returned by the database and contains all of the
/// attributes issued. Its counterpart is `StoredAttestationCopy<UnsignedSdJwtPresentation>`, which contains a subset of
/// the issued attributes, which can then be used for selective disclosure.
pub type FullStoredAttestationCopy = StoredAttestationCopy<VerifiedSdJwt>;

fn credential_payload_from_sd_jwt(sd_jwt: &SdJwt) -> CredentialPayload {
    CredentialPayload::from_sd_jwt_unvalidated(sd_jwt)
        .expect("a stored SD-JWT attestation should convert to CredentialPayload without errors")
}

impl<S> StoredAttestationCopy<S> {
    pub fn attestation_id(&self) -> Uuid {
        self.attestation_id
    }

    pub fn attestation_copy_id(&self) -> Uuid {
        self.attestation_copy_id
    }

    pub fn attestation(&self) -> &StoredAttestation<S> {
        &self.attestation
    }

    pub fn normalized_metadata(&self) -> &NormalizedTypeMetadata {
        &self.normalized_metadata
    }
}

impl<S> StoredAttestation<S>
where
    S: AsRef<SdJwt>,
{
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

impl<S> StoredAttestationCopy<S>
where
    S: AsRef<SdJwt>,
{
    /// Convert the stored attestation (which may contain a subset of the attributes)
    /// to an [`AttestationPresentation`] that can be displayed to the user.
    pub fn into_attestation_presentation(self) -> AttestationPresentation {
        let issuer_registration = self.attestation.issuer_registration();

        match self.attestation {
            StoredAttestation::MsoMdoc { mdoc } => AttestationPresentation::create_from_mdoc(
                AttestationIdentity::Fixed {
                    id: self.attestation_id,
                },
                self.normalized_metadata,
                issuer_registration.organization,
                mdoc.issuer_signed.into_entries_by_namespace(),
            ),
            StoredAttestation::SdJwt { sd_jwt } => {
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
        .expect("a stored attestation should convert to AttestationPresentation without errors")
    }
}

impl StoredAttestationCopy<VerifiedSdJwt> {
    /// Convert the stored attestation into a [`CredentialPayload`], skipping JSON schema validation.
    pub fn into_credential_payload(self) -> CredentialPayload {
        match self.attestation {
            StoredAttestation::MsoMdoc { mdoc } => {
                CredentialPayload::from_mdoc_unvalidated(*mdoc, &self.normalized_metadata)
                    .expect("a stored mdoc attestation should convert to CredentialPayload without errors")
            }
            StoredAttestation::SdJwt { sd_jwt } => credential_payload_from_sd_jwt(sd_jwt.as_ref().as_ref()),
        }
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
    ) -> Result<StoredAttestationCopy<UnsignedSdJwtPresentation>, StoredAttestationCopyPathError> {
        let Self {
            attestation_id,
            attestation_copy_id,
            attestation,
            normalized_metadata,
        } = self;

        let attestation = match attestation {
            StoredAttestation::MsoMdoc { mut mdoc } => {
                mdoc.issuer_signed = mdoc.issuer_signed.into_attribute_subset(claim_paths)?;

                StoredAttestation::MsoMdoc { mdoc }
            }
            StoredAttestation::SdJwt { sd_jwt } => {
                let presentation = claim_paths
                    .into_iter()
                    .try_fold(sd_jwt.into_presentation_builder(), |builder, claim_path| {
                        builder.disclose(claim_path)
                    })?
                    .finish();

                StoredAttestation::SdJwt {
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
    use attestation_data::credential_payload::CredentialPayload;
    use attestation_data::x509::generate::mock::generate_issuer_mock;
    use attestation_types::claim_path::ClaimPath;
    use crypto::keys::WithIdentifier;
    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use sd_jwt::sd_jwt::VerifiedSdJwt;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_at_least::VecNonEmpty;
    use wscd::mock_remote::MockRemoteEcdsaKey;

    use crate::attestation::BSN_ATTR_NAME;
    use crate::attestation::PID_DOCTYPE;

    use super::StoredAttestation;
    use super::StoredAttestationCopy;

    static ATTESTATION_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);

    fn mdoc_stored_attestation_copy(
        issuer_keypair: &KeyPair,
    ) -> (StoredAttestationCopy<VerifiedSdJwt>, VecNonEmpty<ClaimPath>) {
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
            ClaimPath::SelectByKey(PID_DOCTYPE.to_string()),
            ClaimPath::SelectByKey(BSN_ATTR_NAME.to_string()),
        ]
        .try_into()
        .unwrap();

        (copy, bsn_path)
    }

    fn sd_jwt_stored_attestation_copy(
        issuer_keypair: &KeyPair,
    ) -> (StoredAttestationCopy<VerifiedSdJwt>, VecNonEmpty<ClaimPath>) {
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

        let bsn_path = vec![ClaimPath::SelectByKey(BSN_ATTR_NAME.to_string())]
            .try_into()
            .unwrap();

        (copy, bsn_path)
    }

    #[test]
    fn test_stored_attestation_copy() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_registration = IssuerRegistration::new_mock();
        let issuer_keypair = generate_issuer_mock(&ca, issuer_registration.clone().into()).unwrap();

        let (full_presentations, partial_presentations): (Vec<_>, Vec<_>) = [
            mdoc_stored_attestation_copy(&issuer_keypair),
            sd_jwt_stored_attestation_copy(&issuer_keypair),
        ]
        .into_iter()
        .map(|(full_attestation_copy, bsn_path)| {
            // The retrieved `IssuerRegistration` matches the input.
            let full_issuer_registration = full_attestation_copy.attestation.issuer_registration();
            assert_eq!(full_issuer_registration, issuer_registration);

            // The attestation should contain the BSN attribute path.
            assert!(full_attestation_copy.matches_requested_attributes([&bsn_path]));

            // The attestation should not contain some incorrect path.
            let missing_path = vec![ClaimPath::SelectByKey("missing".to_string())].try_into().unwrap();
            assert!(!full_attestation_copy.matches_requested_attributes([&missing_path]));

            // The converted `AttestationPresentation` contains multiple attributes.
            let full_presentation = full_attestation_copy.clone().into_attestation_presentation();
            assert_eq!(full_presentation.attributes.len(), 3);

            // Selecting a particular attribute for disclosure should only succeed if the path exists.
            let partial_attestation_copy = full_attestation_copy
                .clone()
                .try_into_partial([&bsn_path])
                .expect("converting the full attestation copy to on containing just the BSN should succeed");

            let _error = full_attestation_copy
                .try_into_partial([&missing_path])
                .expect_err("converting the full attestation copy to a partial one should not succeed");

            // The retrieved `IssuerRegistration` of the partial attestation copy matches the input.
            let partial_issuer_registration = partial_attestation_copy.attestation.issuer_registration();
            assert_eq!(partial_issuer_registration, issuer_registration);

            // The converted `AttestationPresentation` contains only one attribute.
            let partial_presentation = partial_attestation_copy.into_attestation_presentation();
            assert_eq!(partial_presentation.attributes.len(), 1);

            (full_presentation, partial_presentation)
        })
        .unzip();

        // The full and partial `AttestationPresentation`s should be the same for both formats.
        assert!(full_presentations.iter().all_equal());
        assert!(partial_presentations.iter().all_equal())
    }
}
