//! UniFFI-facing wrapper around `mdoc` close-proximity session encryption.
//!
//! The protocol model and cryptography live in `mdoc`, where they can use typed
//! `SessionTranscript`/`EReaderKeyBytes` values and stay close to the ISO domain model.
//! This module intentionally stays thin:
//! - accept and return raw `Vec<u8>` values for UniFFI
//! - translate between raw bytes and typed `mdoc` values
//! - map `mdoc` errors onto the stable FFI error surface expected by native code

use std::fmt::Display;

use mdoc::holder::disclosure::SessionEncryption;
use mdoc::holder::disclosure::SessionEncryptionError;
use mdoc::holder::disclosure::SessionRole;
use mdoc::holder::disclosure::SessionStatus;
use mdoc::holder::disclosure::encode_status;
use mdoc::holder::disclosure::extract_e_reader_key;
use mdoc::iso::engagement::BleOptions;
use mdoc::iso::engagement::CipherSuiteIdentifier;
use mdoc::iso::engagement::DeviceEngagement;
use mdoc::iso::engagement::DeviceRetrievalMethodKeyed;
use mdoc::iso::engagement::DeviceRetrievalMethodVersion;
use mdoc::iso::engagement::Engagement;
use mdoc::iso::engagement::EngagementVersion;
use mdoc::iso::engagement::RetrievalOptions;
use mdoc::iso::engagement::SecurityKeyed;
use mdoc::iso::engagement::SessionTranscript;
use mdoc::utils::cose::CoseKey;
use mdoc::utils::serialization::CborIntMap;
use mdoc::utils::serialization::TaggedBytes;
use mdoc::utils::serialization::cbor_deserialize;
use mdoc::utils::serialization::cbor_serialize;
use p256::SecretKey;
use p256::ecdsa::VerifyingKey;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use rand::rngs::OsRng;
use serde_bytes::ByteBuf;

const BLE_RETRIEVAL_METHOD_TYPE: u64 = 2;
const BLE_UUID_BYTE_LEN: usize = 16;

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
pub struct CloseProximitySessionCrypto {
    session_crypto: SessionEncryption,
}

impl CloseProximitySessionCrypto {
    #[expect(
        clippy::needless_pass_by_value,
        reason = "UniFFI exports byte arrays as owned Vec<u8> values"
    )]
    pub fn new(
        e_device_private_key: Vec<u8>,
        encoded_reader_key: Vec<u8>,
        encoded_session_transcript: Vec<u8>,
    ) -> Result<Self, CloseProximitySessionCryptoError> {
        // Keep the FFI surface byte-oriented, then convert immediately into the typed `mdoc`
        // representation so the actual protocol logic remains in the protocol crate.
        let e_device_private_key =
            SecretKey::from_slice(&e_device_private_key).map_err(|error| session_encryption_error(&error))?;
        let reader_key: CoseKey =
            cbor_deserialize(encoded_reader_key.as_slice()).map_err(|error| cbor_decoding_error(&error))?;
        let reader_verifying_key = VerifyingKey::try_from(&reader_key).map_err(|error| cbor_decoding_error(&error))?;
        let session_transcript = SessionTranscript::try_from_bytes(&encoded_session_transcript)
            .map_err(|error| cbor_decoding_error(&error))?;
        let session_crypto = SessionEncryption::new(
            SessionRole::Mdoc,
            &e_device_private_key,
            &reader_verifying_key.into(),
            &session_transcript,
        )
        .map_err(map_session_encryption_error)?;

        Ok(Self { session_crypto })
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "UniFFI exports byte arrays as owned Vec<u8> values"
    )]
    pub fn decrypt(
        &self,
        message: Vec<u8>,
    ) -> Result<CloseProximityDecryptedMessage, CloseProximitySessionCryptoError> {
        let decrypted_message = self
            .session_crypto
            .decrypt(&message)
            .map_err(map_session_encryption_error)?;

        Ok(CloseProximityDecryptedMessage {
            data: decrypted_message.data,
            status: decrypted_message.status.map(i64::from),
        })
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "UniFFI exports byte arrays as owned Vec<u8> values"
    )]
    pub fn encrypt(&self, plaintext: Vec<u8>, status_code: i64) -> Result<Vec<u8>, CloseProximitySessionCryptoError> {
        let status = SessionStatus::try_from(status_code).map_err(map_session_encryption_error)?;
        self.session_crypto
            .encrypt(&plaintext, status)
            .map_err(map_session_encryption_error)
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "UniFFI exports byte arrays as owned Vec<u8> values"
)]
pub fn close_proximity_get_e_reader_key(
    session_establishment_message: Vec<u8>,
) -> Result<CloseProximityReaderKey, CloseProximitySessionCryptoError> {
    let e_reader_key = extract_e_reader_key(&session_establishment_message).map_err(map_session_encryption_error)?;

    Ok(CloseProximityReaderKey {
        encoded_cose_key: cbor_serialize(&e_reader_key.0).map_err(|error| cbor_decoding_error(&error))?,
    })
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "UniFFI exports byte arrays as owned Vec<u8> values"
)]
pub fn close_proximity_create_qr_session_setup(
    peripheral_server_uuid: Vec<u8>,
) -> Result<CloseProximityQrSessionSetup, CloseProximitySessionCryptoError> {
    if peripheral_server_uuid.len() != BLE_UUID_BYTE_LEN {
        return Err(other_error(format!(
            "peripheral server UUID must be {BLE_UUID_BYTE_LEN} bytes, got {}",
            peripheral_server_uuid.len()
        )));
    }

    let e_device_private_key = SecretKey::random(&mut OsRng);
    let encoded_device_engagement = encode_device_engagement(&e_device_private_key, &peripheral_server_uuid)?;

    Ok(CloseProximityQrSessionSetup {
        e_device_private_key: e_device_private_key.to_bytes().to_vec(),
        encoded_device_engagement,
    })
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "UniFFI exports byte arrays as owned Vec<u8> values"
)]
pub fn close_proximity_build_session_transcript(
    encoded_device_engagement: Vec<u8>,
    encoded_reader_key: Vec<u8>,
) -> Result<Vec<u8>, CloseProximitySessionCryptoError> {
    let device_engagement: DeviceEngagement =
        cbor_deserialize(encoded_device_engagement.as_slice()).map_err(|error| cbor_decoding_error(&error))?;
    let reader_key: CoseKey =
        cbor_deserialize(encoded_reader_key.as_slice()).map_err(|error| cbor_decoding_error(&error))?;
    let session_transcript = SessionTranscript::new_qr(reader_key, Some(device_engagement));

    cbor_serialize(&session_transcript).map_err(|error| cbor_decoding_error(&error))
}

pub fn close_proximity_encode_session_status(status_code: i64) -> Result<Vec<u8>, CloseProximitySessionCryptoError> {
    let status = SessionStatus::try_from(status_code).map_err(map_session_encryption_error)?;
    encode_status(status).map_err(map_session_encryption_error)
}

fn encode_device_engagement(
    e_device_private_key: &SecretKey,
    peripheral_server_uuid: &[u8],
) -> Result<Vec<u8>, CloseProximitySessionCryptoError> {
    let public_key = e_device_private_key.public_key();
    let encoded_point = public_key.to_encoded_point(false);
    let verifying_key =
        VerifyingKey::from_encoded_point(&encoded_point).map_err(|error| session_encryption_error(&error))?;
    let e_device_key: CoseKey = (&verifying_key)
        .try_into()
        .map_err(|error| session_encryption_error(&error))?;
    let security = SecurityKeyed {
        cipher_suite_identifier: CipherSuiteIdentifier::P256,
        e_device_key_bytes: TaggedBytes(e_device_key),
    }
    .into();
    let device_retrieval_method = DeviceRetrievalMethodKeyed {
        r#type: BLE_RETRIEVAL_METHOD_TYPE,
        version: DeviceRetrievalMethodVersion::V1,
        retrieval_options: RetrievalOptions::Ble(CborIntMap(BleOptions {
            peripheral_server_mode: true,
            central_client_mode: false,
            peripheral_server_uuid: Some(ByteBuf::from(peripheral_server_uuid.to_vec())),
            central_client_uuid: None,
            peripheral_server_address: None,
        })),
    }
    .into();
    let device_engagement: DeviceEngagement = Engagement {
        version: EngagementVersion::V1_0,
        security,
        device_retrieval_methods: Some(utils::vec_nonempty![device_retrieval_method]),
    }
    .into();

    cbor_serialize(&device_engagement).map_err(|error| cbor_decoding_error(&error))
}

fn cbor_decoding_error(reason: &impl Display) -> CloseProximitySessionCryptoError {
    CloseProximitySessionCryptoError::CborDecoding {
        reason: reason.to_string(),
    }
}

fn session_encryption_error(reason: &impl Display) -> CloseProximitySessionCryptoError {
    CloseProximitySessionCryptoError::SessionEncryption {
        reason: reason.to_string(),
    }
}

fn other_error(reason: impl Into<String>) -> CloseProximitySessionCryptoError {
    CloseProximitySessionCryptoError::Other { reason: reason.into() }
}

fn map_session_encryption_error(error: SessionEncryptionError) -> CloseProximitySessionCryptoError {
    match error {
        SessionEncryptionError::Cbor(error) => cbor_decoding_error(&error),
        SessionEncryptionError::MissingReaderKey => cbor_decoding_error(&error),
        SessionEncryptionError::UnsupportedStatus(_) => CloseProximitySessionCryptoError::Other {
            reason: error.to_string(),
        },
        SessionEncryptionError::Crypto(_)
        | SessionEncryptionError::InvalidSessionKeyLength
        | SessionEncryptionError::EncryptionFailed
        | SessionEncryptionError::DecryptionFailed
        | SessionEncryptionError::MissingCiphertext => session_encryption_error(&error),
    }
}

#[cfg(test)]
mod tests {
    use mdoc::holder::disclosure::SessionEncryption;
    use mdoc::holder::disclosure::SessionRole;
    use mdoc::holder::disclosure::SessionStatus;
    use mdoc::iso::engagement::BleOptions;
    use mdoc::iso::engagement::DeviceEngagement;
    use mdoc::iso::engagement::RetrievalOptions;
    use mdoc::iso::engagement::SessionTranscript;
    use mdoc::utils::cose::CoseKey;
    use mdoc::utils::serialization::CborIntMap;
    use mdoc::utils::serialization::TaggedBytes;
    use mdoc::utils::serialization::cbor_deserialize;
    use mdoc::utils::serialization::cbor_serialize;
    use p256::ecdsa::VerifyingKey;
    use p256::elliptic_curve::sec1::ToEncodedPoint;

    use super::CloseProximitySessionCrypto;
    use super::close_proximity_build_session_transcript;
    use super::close_proximity_create_qr_session_setup;
    use super::close_proximity_encode_session_status;
    use super::close_proximity_get_e_reader_key;

    fn secret_key(byte: u8) -> p256::SecretKey {
        p256::SecretKey::from_slice(&[byte; 32]).unwrap()
    }

    fn encoded_cose_key(secret_key: &p256::SecretKey) -> Vec<u8> {
        let public_key = secret_key.public_key();
        let encoded_point = public_key.to_encoded_point(false);
        let verifying_key = VerifyingKey::from_encoded_point(&encoded_point).unwrap();
        let cose_key: CoseKey = (&verifying_key).try_into().unwrap();
        cbor_serialize(&cose_key).unwrap()
    }

    fn session_transcript_bytes(encoded_reader_key: &[u8]) -> Vec<u8> {
        let reader_key: CoseKey = mdoc::utils::serialization::cbor_deserialize(encoded_reader_key).unwrap();
        let session_transcript = SessionTranscript::new_qr(reader_key, None);
        cbor_serialize(&session_transcript).unwrap()
    }

    #[test]
    fn creates_qr_session_setup_and_builds_session_transcript() {
        let peripheral_server_uuid = vec![0x10; 16];
        let qr_session_setup = close_proximity_create_qr_session_setup(peripheral_server_uuid.clone()).unwrap();
        let device_engagement: DeviceEngagement =
            cbor_deserialize(qr_session_setup.encoded_device_engagement.as_slice()).unwrap();
        let device_retrieval_method = device_engagement.0.device_retrieval_methods.as_ref().unwrap().first();

        assert_eq!(qr_session_setup.e_device_private_key.len(), 32);
        assert!(matches!(
            &device_retrieval_method.0.retrieval_options,
            RetrievalOptions::Ble(CborIntMap(BleOptions {
                peripheral_server_mode: true,
                central_client_mode: false,
                peripheral_server_uuid: Some(uuid),
                central_client_uuid: None,
                peripheral_server_address: None,
            })) if uuid.as_ref() == peripheral_server_uuid.as_slice()
        ));

        let e_reader_key = secret_key(2);
        let encoded_reader_key = encoded_cose_key(&e_reader_key);
        let encoded_session_transcript = close_proximity_build_session_transcript(
            qr_session_setup.encoded_device_engagement.clone(),
            encoded_reader_key.clone(),
        )
        .unwrap();
        let session_transcript = SessionTranscript::try_from_bytes(&encoded_session_transcript).unwrap();
        let transcript_device_engagement = session_transcript.0.device_engagement_bytes.unwrap().0;
        let transcript_reader_key = session_transcript.0.e_reader_key_bytes.unwrap().0;

        assert_eq!(
            cbor_serialize(&transcript_device_engagement).unwrap(),
            qr_session_setup.encoded_device_engagement
        );
        assert_eq!(cbor_serialize(&transcript_reader_key).unwrap(), encoded_reader_key);
    }

    #[test]
    fn extracts_reader_key_from_session_establishment_message() {
        let e_device_key = secret_key(1);
        let e_reader_key = secret_key(2);
        let encoded_reader_key = encoded_cose_key(&e_reader_key);
        let e_reader_key_bytes = TaggedBytes(
            mdoc::utils::serialization::cbor_deserialize::<CoseKey, _>(encoded_reader_key.as_slice()).unwrap(),
        );
        let session_transcript_bytes = session_transcript_bytes(&encoded_reader_key);
        let session_transcript = SessionTranscript::try_from_bytes(&session_transcript_bytes).unwrap();
        let reader_session = SessionEncryption::new(
            SessionRole::Reader,
            &e_reader_key,
            &e_device_key.public_key(),
            &session_transcript,
        )
        .unwrap();
        let session_establishment_message = reader_session
            .encrypt_initial_message(b"device-request", &e_reader_key_bytes)
            .unwrap();

        let reader_key = close_proximity_get_e_reader_key(session_establishment_message).unwrap();

        assert_eq!(reader_key.encoded_cose_key, encoded_reader_key);
    }

    #[test]
    fn decrypts_reader_messages_and_encrypts_device_messages() {
        let e_device_key = secret_key(1);
        let e_reader_key = secret_key(2);
        let encoded_reader_key = encoded_cose_key(&e_reader_key);
        let e_reader_key_bytes = TaggedBytes(
            mdoc::utils::serialization::cbor_deserialize::<CoseKey, _>(encoded_reader_key.as_slice()).unwrap(),
        );
        let session_transcript_bytes = session_transcript_bytes(&encoded_reader_key);
        let session_transcript = SessionTranscript::try_from_bytes(&session_transcript_bytes).unwrap();

        let holder_crypto = CloseProximitySessionCrypto::new(
            e_device_key.to_bytes().to_vec(),
            encoded_reader_key.clone(),
            session_transcript_bytes,
        )
        .unwrap();
        let reader_session = SessionEncryption::new(
            SessionRole::Reader,
            &e_reader_key,
            &e_device_key.public_key(),
            &session_transcript,
        )
        .unwrap();

        let reader_message = reader_session
            .encrypt_initial_message(b"device-request", &e_reader_key_bytes)
            .unwrap();
        let decrypted_reader_message = holder_crypto.decrypt(reader_message).unwrap();
        assert_eq!(decrypted_reader_message.data, Some(b"device-request".to_vec()));
        assert_eq!(decrypted_reader_message.status, None);

        let device_message = holder_crypto.encrypt(b"device-response".to_vec(), 20).unwrap();
        let decrypted_device_message = reader_session.decrypt(&device_message).unwrap();
        assert_eq!(decrypted_device_message.data, Some(b"device-response".to_vec()));
        assert_eq!(decrypted_device_message.status, Some(SessionStatus::Termination));
    }

    #[test]
    fn encodes_status_only_message() {
        let e_device_key = secret_key(1);
        let e_reader_key = secret_key(2);
        let encoded_reader_key = encoded_cose_key(&e_reader_key);
        let holder_crypto = CloseProximitySessionCrypto::new(
            e_device_key.to_bytes().to_vec(),
            encoded_reader_key.clone(),
            session_transcript_bytes(&encoded_reader_key),
        )
        .unwrap();

        let status_message = close_proximity_encode_session_status(20).unwrap();
        let decrypted_message = holder_crypto.decrypt(status_message).unwrap();

        assert_eq!(decrypted_message.data, None);
        assert_eq!(decrypted_message.status, Some(20));
    }
}
