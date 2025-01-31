use itertools::Itertools;
use nl_wallet_mdoc::utils::x509::CertificateError;
use nutype::nutype;
use rustls_pki_types::TrustAnchor;
use wallet_common::vec_at_least::VecNonEmpty;

use crate::token::CredentialPreview;
use crate::Format;

pub trait CredentialFormat {
    fn format(&self) -> Format;
}

pub trait CredentialType: CredentialFormat {
    fn credential_type(&self) -> String;
}

impl CredentialFormat for CredentialPreview {
    fn format(&self) -> Format {
        match self {
            CredentialPreview::MsoMdoc { .. } => Format::MsoMdoc,
        }
    }
}

impl CredentialType for CredentialPreview {
    fn credential_type(&self) -> String {
        match self {
            CredentialPreview::MsoMdoc { unsigned_mdoc, .. } => unsigned_mdoc.doc_type.clone(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
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
            .as_slice()
            .iter()
            .counts_by(T::format)
            .values()
            .all(|v| *v <= 1)
            .then_some(())
            .ok_or(CredentialFormatsError::DuplicateFormats)?;

        formats
            .as_slice()
            .iter()
            .map(T::credential_type)
            .all_equal()
            .then_some(())
            .ok_or(CredentialFormatsError::DifferentCredentialTypes)?;

        Ok(())
    }

    pub fn credential_type(&self) -> String {
        assert!(self.as_ref().as_slice().iter().map(T::credential_type).all_equal());

        self.as_ref().first().credential_type()
    }
}

impl CredentialFormats<CredentialPreview> {
    pub fn verify(&self, trust_anchors: &[TrustAnchor<'_>]) -> Result<(), CertificateError> {
        self.as_ref()
            .as_slice()
            .iter()
            .map(|preview| preview.verify(trust_anchors))
            .collect::<Result<Vec<()>, CertificateError>>()?;

        Ok(())
    }

    pub fn flatten_copies(&self) -> VecNonEmpty<CredentialPreview> {
        self.as_ref()
            .as_slice()
            .iter()
            .flat_map(|preview| itertools::repeat_n(preview.clone(), preview.copy_count().into()))
            .collect_vec()
            .try_into()
            .unwrap()
    }
}
