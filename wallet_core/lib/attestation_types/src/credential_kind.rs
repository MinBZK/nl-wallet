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
