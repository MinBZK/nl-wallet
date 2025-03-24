use itertools::Itertools;
use nutype::nutype;
use rustls_pki_types::TrustAnchor;

use sd_jwt::metadata_chain::TypeMetadataDocuments;
use wallet_common::vec_at_least::VecNonEmpty;

use crate::token::CredentialPreview;
use crate::token::CredentialPreviewError;
use crate::Format;

pub trait CredentialFormat {
    fn format(&self) -> Format;
}

pub trait CredentialType: CredentialFormat {
    fn credential_type(&self) -> &str;
}

pub trait Credential: CredentialType {
    fn metadata_type(&self) -> &TypeMetadataDocuments;
}

impl CredentialFormat for CredentialPreview {
    fn format(&self) -> Format {
        match self {
            CredentialPreview::MsoMdoc { .. } => Format::MsoMdoc,
        }
    }
}

impl CredentialType for CredentialPreview {
    fn credential_type(&self) -> &str {
        match self {
            CredentialPreview::MsoMdoc { unsigned_mdoc, .. } => &unsigned_mdoc.doc_type,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum CredentialFormatsError {
    #[error("duplicate credential formats not allowed")]
    DuplicateFormats,
    #[error("all credential formats must have the same attestation type")]
    DifferentCredentialTypes,
}

/// This struct is intended to be used as a wrapper for credentials and other structures that are issued or used in
/// multiple formats. This type offers the following guarantees:
/// - It always contains at least one format.
/// - Each format can only be present once.
/// - All formats must have the same attestation type.
#[nutype(
    derive(Clone, Debug, Serialize, Deserialize, AsRef),
    validate(with = CredentialFormats::validate, error = CredentialFormatsError),
)]
pub struct CredentialFormats<T: CredentialType>(VecNonEmpty<T>);

impl<T: CredentialType> IntoIterator for CredentialFormats<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

impl<T: CredentialType> CredentialFormats<T> {
    fn validate(formats: &VecNonEmpty<T>) -> Result<(), CredentialFormatsError> {
        formats
            .iter()
            .counts_by(T::format)
            .values()
            .all(|v| *v <= 1)
            .then_some(())
            .ok_or(CredentialFormatsError::DuplicateFormats)?;

        formats
            .iter()
            .map(T::credential_type)
            .all_equal()
            .then_some(())
            .ok_or(CredentialFormatsError::DifferentCredentialTypes)?;

        Ok(())
    }
}

impl CredentialFormats<CredentialPreview> {
    pub fn verify(&self, trust_anchors: &[TrustAnchor<'_>]) -> Result<(), CredentialPreviewError> {
        self.as_ref()
            .iter()
            .map(|preview| preview.verify(trust_anchors))
            .collect::<Result<Vec<()>, CredentialPreviewError>>()?;

        Ok(())
    }

    pub fn flatten_copies(&self) -> VecNonEmpty<CredentialPreview> {
        self.as_ref()
            .iter()
            .flat_map(|preview| itertools::repeat_n(preview.clone(), preview.copy_count().into()))
            .collect_vec()
            .try_into()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::Format;

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestCredential {
        format: Format,
        credential_type: String,
    }

    impl CredentialFormat for TestCredential {
        fn format(&self) -> Format {
            self.format
        }
    }

    impl CredentialType for TestCredential {
        fn credential_type(&self) -> &str {
            &self.credential_type
        }
    }

    #[rstest]
    #[case(vec![TestCredential {
        format: Format::MsoMdoc,
        credential_type: "type1".to_string(),
    }, TestCredential {
        format: Format::Jwt,
        credential_type: "type1".to_string(),
    }].try_into().unwrap(), Ok(()))]
    #[case(vec![TestCredential {
        format: Format::MsoMdoc,
        credential_type: "type1".to_string(),
    }, TestCredential {
        format: Format::Jwt,
        credential_type: "type1".to_string(),
    }, TestCredential {
        format: Format::MsoMdoc,
        credential_type: "type1".to_string(),
    }].try_into().unwrap(), Err(CredentialFormatsError::DuplicateFormats))]
    #[case(vec![TestCredential {
        format: Format::MsoMdoc,
        credential_type: "credential_type".to_string(),
    }, TestCredential {
        format: Format::Jwt,
        credential_type: "different_credential_type".to_string(),
    }].try_into().unwrap(), Err(CredentialFormatsError::DifferentCredentialTypes))]
    fn test_credential_formats(
        #[case] credentials: VecNonEmpty<TestCredential>,
        #[case] expected: Result<(), CredentialFormatsError>,
    ) {
        let formats = CredentialFormats::validate(&credentials);
        assert_eq!(formats, expected);
    }
}
