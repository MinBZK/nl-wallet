#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CloseProximitySessionCryptoError {
    #[error("CBOR decoding error: {reason}")]
    CborDecoding { reason: String },
    #[error("session encryption error: {reason}")]
    SessionEncryption { reason: String },
    #[error("other close proximity session crypto error: {reason}")]
    Other { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseProximityReaderKey {
    pub encoded_cose_key: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseProximityDecryptedMessage {
    pub data: Option<Vec<u8>>,
    pub status: Option<i64>,
}

#[derive(Debug)]
pub struct CloseProximitySessionCrypto {
    e_device_private_key: Vec<u8>,
    encoded_reader_key: Vec<u8>,
    encoded_session_transcript: Vec<u8>,
}

impl CloseProximitySessionCrypto {
    pub fn new(
        e_device_private_key: Vec<u8>,
        encoded_reader_key: Vec<u8>,
        encoded_session_transcript: Vec<u8>,
    ) -> Result<Self, CloseProximitySessionCryptoError> {
        Ok(Self {
            e_device_private_key,
            encoded_reader_key,
            encoded_session_transcript,
        })
    }

    pub fn decrypt(
        &self,
        _message: Vec<u8>,
    ) -> Result<CloseProximityDecryptedMessage, CloseProximitySessionCryptoError> {
        let _ = (
            &self.e_device_private_key,
            &self.encoded_reader_key,
            &self.encoded_session_transcript,
        );

        Err(CloseProximitySessionCryptoError::Other {
            reason: "close proximity session crypto decrypt is not implemented yet".to_string(),
        })
    }

    pub fn encrypt(&self, _plaintext: Vec<u8>, _status_code: i64) -> Result<Vec<u8>, CloseProximitySessionCryptoError> {
        let _ = (
            &self.e_device_private_key,
            &self.encoded_reader_key,
            &self.encoded_session_transcript,
        );

        Err(CloseProximitySessionCryptoError::Other {
            reason: "close proximity session crypto encrypt is not implemented yet".to_string(),
        })
    }
}

pub fn close_proximity_get_e_reader_key(
    _session_establishment_message: Vec<u8>,
) -> Result<CloseProximityReaderKey, CloseProximitySessionCryptoError> {
    Err(CloseProximitySessionCryptoError::Other {
        reason: "close proximity session crypto get_e_reader_key is not implemented yet".to_string(),
    })
}
