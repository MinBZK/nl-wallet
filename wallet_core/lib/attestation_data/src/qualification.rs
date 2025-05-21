use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, SerializeDisplay, DeserializeFromStr, strum::EnumString, strum::Display,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AttestationQualification {
    /// Qualified Electronic Attestation of Attributes
    QEAA,

    /// Electronic attestation of attributes issued by or on behalf of a public sector body
    #[strum(to_string = "PuB-EAA")]
    PubEAA,

    /// Electronic attestation of attributes (non-qualified)
    #[default]
    EAA,
}
