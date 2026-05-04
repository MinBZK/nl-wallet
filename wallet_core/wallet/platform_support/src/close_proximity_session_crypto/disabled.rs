const IOS_SESSION_CRYPTO_UNIMPLEMENTED_MESSAGE: &str =
    "close proximity session crypto is only available with the `ios_session_crypto` feature";

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
pub struct CloseProximityQrSessionSetup {
    pub e_device_private_key: Vec<u8>,
    pub encoded_device_engagement: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseProximityDecryptedMessage {
    pub data: Option<Vec<u8>>,
    pub status: Option<i64>,
}

#[derive(Debug)]
pub struct CloseProximitySessionCrypto;

impl CloseProximitySessionCrypto {
    pub fn new(
        _e_device_private_key: Vec<u8>,
        _encoded_reader_key: Vec<u8>,
        _encoded_session_transcript: Vec<u8>,
    ) -> Result<Self, CloseProximitySessionCryptoError> {
        ios_session_crypto_unimplemented()
    }

    pub fn decrypt(
        &self,
        _message: Vec<u8>,
    ) -> Result<CloseProximityDecryptedMessage, CloseProximitySessionCryptoError> {
        ios_session_crypto_unimplemented()
    }

    pub fn encrypt(
        &self,
        _plaintext: Vec<u8>,
        _status_code: i64,
    ) -> Result<Vec<u8>, CloseProximitySessionCryptoError> {
        ios_session_crypto_unimplemented()
    }
}

pub fn close_proximity_get_e_reader_key(
    _session_establishment_message: Vec<u8>,
) -> Result<CloseProximityReaderKey, CloseProximitySessionCryptoError> {
    ios_session_crypto_unimplemented()
}

pub fn close_proximity_create_qr_session_setup(
    _peripheral_server_uuid: Vec<u8>,
) -> Result<CloseProximityQrSessionSetup, CloseProximitySessionCryptoError> {
    ios_session_crypto_unimplemented()
}

pub fn close_proximity_build_session_transcript(
    _encoded_device_engagement: Vec<u8>,
    _encoded_reader_key: Vec<u8>,
) -> Result<Vec<u8>, CloseProximitySessionCryptoError> {
    ios_session_crypto_unimplemented()
}

pub fn close_proximity_encode_session_status(_status_code: i64) -> Result<Vec<u8>, CloseProximitySessionCryptoError> {
    ios_session_crypto_unimplemented()
}

fn ios_session_crypto_unimplemented() -> ! {
    unimplemented!("{IOS_SESSION_CRYPTO_UNIMPLEMENTED_MESSAGE}")
}
