use super::errors::Result;

pub use self::payload::ChallengeRequest;
pub use self::payload::ChallengeRequestPayload;
pub use self::payload::ChallengeResponse;
pub use self::payload::ChallengeResponsePayload;
pub use self::payload::SequenceNumberComparison;

mod payload;
mod raw_value;
mod signed_message;

/// Used internally within this submodule to represent a payload that contains a challenge.
pub trait ContainsChallenge {
    fn challenge(&self) -> Result<impl AsRef<[u8]>>;
}

/// The types of signature a message can be signed with, which
/// is either an ECDSA signature or an Apple assertion. The
/// former has a subtype in the form of [`EcdsaSignatureType`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureType {
    Ecdsa(EcdsaSignatureType),
    AppleAssertion,
}

/// An ECDSA signature can either originate from a derived
/// PIN key or a key stored in hardware on the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcdsaSignatureType {
    Pin,
    Google,
}
