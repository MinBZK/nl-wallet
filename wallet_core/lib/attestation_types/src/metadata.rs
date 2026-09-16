use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::claim_path::ClaimPath;
use crate::data_uri::DataUriError;
use crate::image::ImageError;

#[derive(Debug, thiserror::Error)]
pub enum AttestationMetadataError {
    #[error("display information is missing a name for locale {}", .0.as_deref().unwrap_or("<none>"))]
    NoDisplayName(Option<String>),

    #[error("display information is missing a locale")]
    NoDisplayLocale,

    #[error("could not read image as a data URI: {0}")]
    ImageDataUri(#[source] DataUriError),

    #[error("could not convert image: {0}")]
    Image(#[source] ImageError),
}

/// Describes the constraints of a single attestation claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimConstraint<'a> {
    /// The path to the claim within the attestation.
    pub path: &'a VecNonEmpty<ClaimPath>,

    /// Whether the issuer is required to include this claim in the attestation.
    pub mandatory: bool,
}

impl<'a> ClaimConstraint<'a> {
    /// The `SelectByKey` path components of this claim as keys.
    pub fn key_path(&self) -> Option<VecNonEmpty<&'a str>> {
        self.path
            .nonempty_iter()
            .map(|path| path.try_key_path().ok_or(()))
            .collect::<Result<VecNonEmpty<_>, _>>()
            .ok()
    }
}

/// Implemented by every kind of metadata that describes the claims an attestation may contain.
pub trait AttestationClaims {
    /// The claims this metadata describes, which together determine the attributes that an attestation is permitted
    /// and required to contain.
    fn claim_constraints(&self) -> impl Iterator<Item = ClaimConstraint<'_>>;
}
