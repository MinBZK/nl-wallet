use attestation_types::metadata::AttestationMetadataError;
use attestation_types::metadata::ClaimDescription;
use attestation_types::metadata::DisplayMetadata;
use openid4vc::metadata::issuer_metadata::CredentialMetadata;
use openid4vc::wallet_issuance::issuance_session::OfferedCredentialMetadata;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;

use crate::storage::StoredAttestationMetadata;

pub trait AttestationDisplay {
    fn into_presentation_components(
        self,
    ) -> Result<(Vec<DisplayMetadata>, Vec<ClaimDescription>), AttestationMetadataError>;
}

impl AttestationDisplay for CredentialMetadata {
    fn into_presentation_components(
        self,
    ) -> Result<(Vec<DisplayMetadata>, Vec<ClaimDescription>), AttestationMetadataError> {
        // Note that metadata without any display properties or claims is deliberately not an error here, but rather a
        // UI concern.
        let display = self
            .display
            .map(|display| {
                display
                    .into_inner()
                    .into_iter()
                    .map(DisplayMetadata::try_from)
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        let claims = self
            .claims
            .map(|claims| claims.into_inner().into_iter().map(ClaimDescription::from).collect())
            .unwrap_or_default();

        Ok((display, claims))
    }
}

// Note that this conversion is infallible. It also always yields display metadata, as a type metadata chain is
// validated to contain it when it is normalized.
impl AttestationDisplay for NormalizedTypeMetadata {
    fn into_presentation_components(
        self,
    ) -> Result<(Vec<DisplayMetadata>, Vec<ClaimDescription>), AttestationMetadataError> {
        let (display, claims) = self.into_display_and_claims();

        let claims = claims
            .into_iter()
            .map(|claim| ClaimDescription {
                path: claim.path,
                display: claim.display,
                svg_id: claim.svg_id.map(String::from),
            })
            .collect();

        Ok((display.into_inner(), claims))
    }
}

impl AttestationDisplay for StoredAttestationMetadata {
    fn into_presentation_components(
        self,
    ) -> Result<(Vec<DisplayMetadata>, Vec<ClaimDescription>), AttestationMetadataError> {
        match self {
            StoredAttestationMetadata::TypeMetadata(type_metadata) => type_metadata.into_presentation_components(),
            StoredAttestationMetadata::CredentialMetadata(credential_metadata) => {
                credential_metadata.into_presentation_components()
            }
        }
    }
}

impl AttestationDisplay for OfferedCredentialMetadata {
    fn into_presentation_components(
        self,
    ) -> Result<(Vec<DisplayMetadata>, Vec<ClaimDescription>), AttestationMetadataError> {
        match self {
            OfferedCredentialMetadata::TypeMetadata { normalized, .. } => normalized.into_presentation_components(),
            OfferedCredentialMetadata::CredentialMetadata(credential_metadata) => {
                credential_metadata.into_presentation_components()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use attestation_types::claim_path::ClaimPath;
    use attestation_types::image::Image;
    use attestation_types::metadata::ClaimDisplayMetadata;
    use attestation_types::metadata::RenderingMetadata;
    use openid4vc::metadata::issuer_metadata::CredentialClaim;
    use openid4vc::metadata::issuer_metadata::CredentialDisplay;
    use openid4vc::metadata::issuer_metadata::Logo;
    use openid4vc::metadata::issuer_metadata::NameLocale;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use super::*;

    fn key_path(keys: &[&str]) -> VecNonEmpty<ClaimPath> {
        keys.iter()
            .map(|key| ClaimPath::SelectByKey(String::from(*key)))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    #[test]
    fn test_credential_metadata_presentation_components() {
        let (display, claims) = CredentialMetadata::new_full_example()
            .into_presentation_components()
            .expect("credential metadata should convert to presentation components");

        let display = display.into_iter().next().expect("display should contain one entry");
        assert_eq!(display.locale, "en");
        assert_eq!(display.name, "Example credential");
        assert_eq!(display.description.as_deref(), Some("An example"));
        // The Credential Issuer metadata has no equivalent of a templated summary.
        assert!(display.summary.is_none());

        let rendering = display.rendering.expect("display should contain rendering metadata");
        assert_matches!(
            rendering,
            RenderingMetadata::Simple {
                logo: Some(logo),
                background_image: Some(background_image),
                background_color: Some(background_color),
                text_color: Some(text_color),
            } if matches!(logo.image, Image::Png(_))
                && logo.alt_text.as_ref() == "a single pixel"
                && matches!(background_image.image, Image::Png(_))
                && background_color == "#FFFFFF"
                && text_color == "#000000"
        );

        assert_eq!(
            claims.iter().map(|claim| claim.path.clone()).collect::<Vec<_>>(),
            vec![key_path(&["birth_date"]), key_path(&["place_of_birth", "locality"])]
        );
        assert_eq!(
            claims.first().unwrap().display,
            vec![ClaimDisplayMetadata {
                locale: String::from("en"),
                label: String::from("label for birth_date"),
                description: None,
            }]
        );
        // Only SD-JWT VC Type Metadata provides SVG template identifiers.
        assert!(claims.iter().all(|claim| claim.svg_id.is_none()));
    }

    /// Metadata that describes no display properties is not an error, as it is up to the UI to decide what to render.
    #[test]
    fn test_credential_metadata_without_display() {
        let metadata = CredentialMetadata {
            display: None,
            claims: Some(vec_nonempty![CredentialClaim {
                path: key_path(&["birth_date"]),
                mandatory: false,
                display: None,
            }]),
        };

        let (display, claims) = metadata
            .into_presentation_components()
            .expect("credential metadata without display should convert");

        assert!(display.is_empty());
        assert_eq!(claims.len(), 1);
    }

    #[test]
    fn test_credential_metadata_presentation_components_error_external_logo() {
        let metadata = CredentialMetadata {
            display: Some(vec_nonempty![CredentialDisplay {
                name_locale: NameLocale {
                    name: Some(String::from("Example credential")),
                    locale: Some(String::from("en")),
                },
                // Only images that are embedded in the URI are accepted.
                logo: Some(Logo {
                    uri: "https://example.com/logo.png".parse().unwrap(),
                    alt_text: None,
                }),
                description: None,
                background_color: None,
                background_image: None,
                text_color: None,
            }]),
            claims: None,
        };

        let error = metadata
            .into_presentation_components()
            .expect_err("credential metadata with an externally hosted logo should not convert");

        assert_matches!(error, AttestationMetadataError::ImageDataUri(_));
    }

    #[test]
    fn test_credential_metadata_without_claims() {
        let metadata = CredentialMetadata {
            display: Some(vec_nonempty![CredentialDisplay {
                name_locale: NameLocale {
                    name: Some(String::from("Example credential")),
                    locale: Some(String::from("en")),
                },
                logo: None,
                description: None,
                background_color: None,
                background_image: None,
                text_color: None,
            }]),
            claims: None,
        };

        let (display, claims) = metadata
            .into_presentation_components()
            .expect("credential metadata without claims should convert");

        // Rendering metadata is only present if any of its properties is.
        assert!(
            display
                .into_iter()
                .next()
                .expect("display should contain one entry")
                .rendering
                .is_none()
        );
        assert!(claims.is_empty());
    }

    /// Type metadata always yields display properties, as a chain is validated to contain them when normalized.
    #[test]
    fn test_type_metadata_presentation_components() {
        let (display, claims) = NormalizedTypeMetadata::nl_pid_example()
            .into_presentation_components()
            .expect("type metadata should convert to presentation components");

        assert!(!display.is_empty());
        assert!(!claims.is_empty());
    }
}
