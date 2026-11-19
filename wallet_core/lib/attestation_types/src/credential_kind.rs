use derive_more::Constructor;
use derive_more::Display;
use serde::Deserialize;
use serde::Serialize;

use crate::credential_format::Format;

/// The combination of an attestation type and the format it is expressed in. Note that the same attestation type can be
/// present in multiple formats, which means that neither of these two values identifies an attestation on its own.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Constructor, Serialize, Deserialize)]
#[display("({format}): {attestation_type}")]
pub struct CredentialKind {
    pub format: Format,
    pub attestation_type: String,
}

impl CredentialKind {
    /// Borrows this credential kind as a [`CredentialKindRef`].
    pub fn as_ref(&self) -> CredentialKindRef<'_> {
        CredentialKindRef {
            format: self.format,
            attestation_type: &self.attestation_type,
        }
    }
}

/// The borrowed counterpart of [`CredentialKind`], which allows a credential kind to be passed around, compared and
/// looked up without cloning its attestation type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Display, Constructor)]
#[display("({format}): {attestation_type}")]
pub struct CredentialKindRef<'a> {
    pub format: Format,
    pub attestation_type: &'a str,
}

impl CredentialKindRef<'_> {
    /// Converts this credential kind into an owned [`CredentialKind`], cloning its attestation type.
    pub fn into_owned(self) -> CredentialKind {
        CredentialKind {
            format: self.format,
            attestation_type: self.attestation_type.to_owned(),
        }
    }
}
