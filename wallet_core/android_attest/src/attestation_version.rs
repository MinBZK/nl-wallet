use num_traits::ToPrimitive;
use rasn::ber::de::DecoderOptions;
use rasn::ber::{self};
use rasn::types::Integer;
use rasn::types::Tag;
use rasn::AsnType;
use rasn::Decode;
use rasn::Decoder;

#[derive(Debug, Clone, Default, PartialEq, Eq, AsnType)]
pub struct AttestationVersion {
    pub attestation_version: Integer,
}

#[derive(Debug, thiserror::Error)]
pub enum AttestationVersionError {
    #[error("decoding error: {0}")]
    Decoding(#[from] rasn::error::DecodeError),
    #[error("expected Integer as first field in a sequence")]
    InvalidInput,
}

impl AttestationVersion {
    pub fn try_parse_from(bytes: &[u8]) -> Result<Self, AttestationVersionError> {
        let mut attestation_version = Err(AttestationVersionError::InvalidInput);

        let mut decoder = ber::de::Decoder::new(bytes, DecoderOptions::der());
        decoder
            .decode_sequence(Tag::SEQUENCE, None::<fn() -> AttestationVersion>, |decoder| {
                attestation_version = match Integer::decode(decoder) {
                    Ok(value) => Ok(value.into()),
                    Err(error) => Err(error.into()),
                };
                // Return error here, as we're only interested in the attestation_version
                Err(rasn::error::DecodeError::unexpected_extra_data(
                    decoder.decoded_len(),
                    decoder.codec(),
                ))
            })
            .expect_err("this should fail because this doesn't consume the whole sequence");

        attestation_version
    }
}

impl From<Integer> for AttestationVersion {
    fn from(attestation_version: Integer) -> Self {
        AttestationVersion { attestation_version }
    }
}

impl AttestationVersion {
    pub fn as_u16(&self) -> Option<u16> {
        self.attestation_version.to_u16()
    }
}
