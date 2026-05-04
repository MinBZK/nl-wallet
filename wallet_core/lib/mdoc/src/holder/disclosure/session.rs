//! Core close-proximity session-message encryption for ISO 18013-5 style disclosure sessions.
//!
//! This code lives in `mdoc` rather than `platform_support` because the concepts it operates on
//! are protocol concepts, not platform concepts:
//! - `SessionTranscript`
//! - `EReaderKeyBytes`
//! - the wire shape of session messages
//! - the ECDH/HKDF/AES-GCM rules used by the mdoc session channel
//!
//! `platform_support` keeps the UniFFI-facing `Vec<u8>` wrapper and native glue, but delegates the
//! actual session-message cryptography to this module.
//!
//! The core implementation is role-agnostic and is ready to be used from either side of the
//! close-proximity session: `SessionRole::Mdoc` for the wallet/device side or
//! `SessionRole::Reader` for the verifier side. That did not require a separate reader-specific
//! implementation because the protocol logic is almost identical; the main difference is which
//! directional key and nonce identifier are treated as outbound vs inbound.

use aes_gcm::Aes256Gcm;
use aes_gcm::Nonce;
use aes_gcm::aead::Aead;
use aes_gcm::aead::KeyInit;
use crypto::utils::hkdf;
use crypto::utils::sha256;
use p256::PublicKey;
use p256::SecretKey;
use p256::ecdh;
use parking_lot::Mutex;
use serde::Deserialize;
use serde::Serialize;
use serde_bytes::ByteBuf;
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use serde_with::skip_serializing_none;

use crate::iso::engagement::EReaderKeyBytes;
use crate::iso::engagement::SessionTranscript;
use crate::iso::engagement::SessionTranscriptBytes;
use crate::utils::crypto::CryptoError;
use crate::utils::serialization::CborError;
use crate::utils::serialization::cbor_deserialize;
use crate::utils::serialization::cbor_serialize;

// ISO 18013-5:2021 derives separate directional traffic keys from the same ECDH shared secret and
// transcript-derived salt. These HKDF `info` labels provide the required domain separation:
// `SKDevice` for mdoc->reader traffic and `SKReader` for reader->mdoc traffic.
//
// `SessionEncryption` keeps both keys in one object because that matches how the native
// `SessionEncryption` APIs are exposed today, but the underlying cryptographic model is still
// directional rather than "one symmetric key for the whole session".
const SESSION_KEY_INFO_DEVICE: &str = "SKDevice";
const SESSION_KEY_INFO_READER: &str = "SKReader";
const SESSION_KEY_BYTE_LEN: usize = 32;

/// Identifies which side of the close-proximity session this helper represents.
///
/// The role decides both which derived session key is used for encryption/decryption and which
/// nonce identifier is used for each message direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRole {
    Mdoc,
    Reader,
}

impl SessionRole {
    fn encryption_iv_identifier(self) -> u32 {
        match self {
            SessionRole::Mdoc => 1,
            SessionRole::Reader => 0,
        }
    }

    fn decryption_iv_identifier(self) -> u32 {
        match self {
            SessionRole::Mdoc => 0,
            SessionRole::Reader => 1,
        }
    }
}

/// Session status codes carried alongside encrypted protocol messages.
///
/// These are kept typed in `mdoc` and converted to raw integers only at the UniFFI boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u64)]
pub enum SessionStatus {
    EncryptionError = 10,
    DecodingError = 11,
    Termination = 20,
}

impl TryFrom<i64> for SessionStatus {
    type Error = SessionEncryptionError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            10 => Ok(SessionStatus::EncryptionError),
            11 => Ok(SessionStatus::DecodingError),
            20 => Ok(SessionStatus::Termination),
            _ => Err(SessionEncryptionError::UnsupportedStatus(value)),
        }
    }
}

impl From<SessionStatus> for i64 {
    fn from(value: SessionStatus) -> Self {
        value as i64
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDecryptedMessage {
    pub data: Option<Vec<u8>>,
    pub status: Option<SessionStatus>,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionEncryptionError {
    #[error("CBOR error: {0}")]
    Cbor(#[from] CborError),
    #[error("cryptographic error: {0}")]
    Crypto(#[from] CryptoError),
    #[error("session establishment message did not contain eReaderKey")]
    MissingReaderKey,
    #[error("data cannot be empty in initial message")]
    EmptyInitialMessage,
    #[error("invalid session key length")]
    InvalidSessionKeyLength,
    #[error("failed to encrypt session message")]
    EncryptionFailed,
    #[error("failed to decrypt session message")]
    DecryptionFailed,
    #[error("session message did not contain ciphertext")]
    MissingCiphertext,
    #[error("unsupported session status code: {0}")]
    UnsupportedStatus(i64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[skip_serializing_none]
struct SessionData {
    #[serde(default, rename = "eReaderKey")]
    e_reader_key: Option<EReaderKeyBytes>,
    #[serde(default)]
    data: Option<ByteBuf>,
    #[serde(default)]
    status: Option<SessionStatus>,
}

#[derive(Debug)]
struct SessionCounters {
    encrypted_counter: u32,
    decrypted_counter: u32,
}

impl Default for SessionCounters {
    fn default() -> Self {
        Self {
            encrypted_counter: 1,
            decrypted_counter: 1,
        }
    }
}

/// Stateful session-message encryptor/decryptor for one side of a close-proximity disclosure
/// session.
///
/// This owns both directional session keys and keeps per-direction counters so nonce construction
/// remains deterministic across multiple messages.
#[derive(Debug)]
pub struct SessionEncryption {
    role: SessionRole,
    sk_self: Vec<u8>,
    sk_remote: Vec<u8>,
    counters: Mutex<SessionCounters>,
}

impl SessionEncryption {
    /// Derive both directional session keys from the shared secret and transcript.
    ///
    /// We derive both keys up front and keep them together so the caller does not have to manage
    /// separate "reader key" and "device key" objects. This preserves the ISO directional-key
    /// model and the old `mdoc` behavior while keeping the API closer to the current native
    /// `SessionEncryption` wrappers.
    pub fn new(
        role: SessionRole,
        e_self_key: &SecretKey,
        remote_public_key: &PublicKey,
        session_transcript: &SessionTranscript,
    ) -> Result<Self, SessionEncryptionError> {
        let shared_secret = ecdh::diffie_hellman(e_self_key.to_nonzero_scalar(), remote_public_key.as_affine());
        let salt = session_transcript_salt(session_transcript)?;
        let device_sk = hkdf(
            shared_secret.raw_secret_bytes().as_ref(),
            salt.as_slice(),
            SESSION_KEY_INFO_DEVICE,
            SESSION_KEY_BYTE_LEN,
        )
        .map_err(|_| SessionEncryptionError::Crypto(CryptoError::Hkdf))?;
        let reader_sk = hkdf(
            shared_secret.raw_secret_bytes().as_ref(),
            salt.as_slice(),
            SESSION_KEY_INFO_READER,
            SESSION_KEY_BYTE_LEN,
        )
        .map_err(|_| SessionEncryptionError::Crypto(CryptoError::Hkdf))?;

        let (sk_self, sk_remote) = match role {
            SessionRole::Mdoc => (device_sk, reader_sk),
            SessionRole::Reader => (reader_sk, device_sk),
        };

        Ok(Self {
            role,
            sk_self,
            sk_remote,
            counters: Mutex::new(SessionCounters::default()),
        })
    }

    pub fn encrypt(&self, plaintext: &[u8], status: SessionStatus) -> Result<Vec<u8>, SessionEncryptionError> {
        self.encode_message(Some(plaintext), Some(status), None)
    }

    /// Encode the reader's first encrypted message, which also carries the tagged `eReaderKey`
    /// used by the device to finish reconstructing the session transcript.
    pub fn encrypt_initial_message(
        &self,
        plaintext: &[u8],
        e_reader_key: &EReaderKeyBytes,
    ) -> Result<Vec<u8>, SessionEncryptionError> {
        self.encode_message(Some(plaintext), None, Some(e_reader_key.clone()))
    }

    pub fn decrypt(&self, message: &[u8]) -> Result<SessionDecryptedMessage, SessionEncryptionError> {
        let payload: SessionData = cbor_deserialize(message)?;

        let data = match payload.data {
            Some(ciphertext) => {
                let mut counters = self.counters.lock();
                let iv = session_iv(self.role.decryption_iv_identifier(), counters.decrypted_counter);
                let plaintext = decrypt_session_data(&self.sk_remote, iv, ciphertext.as_ref())?;
                counters.decrypted_counter += 1;
                Some(plaintext)
            }
            None => None,
        };

        Ok(SessionDecryptedMessage {
            data,
            status: payload.status,
        })
    }

    fn encode_message(
        &self,
        plaintext: Option<&[u8]>,
        status: Option<SessionStatus>,
        e_reader_key: Option<EReaderKeyBytes>,
    ) -> Result<Vec<u8>, SessionEncryptionError> {
        if e_reader_key.is_some() && plaintext.is_none() {
            return Err(SessionEncryptionError::EmptyInitialMessage);
        }

        let data = match plaintext {
            Some(plaintext) => {
                let mut counters = self.counters.lock();
                let iv = session_iv(self.role.encryption_iv_identifier(), counters.encrypted_counter);
                let ciphertext = encrypt_session_data(&self.sk_self, iv, plaintext)?;
                counters.encrypted_counter += 1;
                Some(ByteBuf::from(ciphertext))
            }
            None => None,
        };

        let payload = SessionData {
            e_reader_key,
            data,
            status,
        };

        Ok(cbor_serialize(&payload)?)
    }
}

/// Extract the tagged `eReaderKey` from the first reader message.
///
/// This is kept in the core protocol crate because the wire shape is part of the session-message
/// protocol itself, even though `platform_support` exposes it as raw bytes over UniFFI.
pub fn extract_e_reader_key(session_establishment_message: &[u8]) -> Result<EReaderKeyBytes, SessionEncryptionError> {
    let payload: SessionData = cbor_deserialize(session_establishment_message)?;
    payload.e_reader_key.ok_or(SessionEncryptionError::MissingReaderKey)
}

/// Encode a status-only session message without ciphertext.
///
/// ISO session status messages are sent as CBOR `SessionData` objects carrying only the `status`
/// field. This helper keeps that wire detail in `mdoc` so native code does not need to duplicate
/// the payload shape.
pub fn encode_status(status: SessionStatus) -> Result<Vec<u8>, SessionEncryptionError> {
    cbor_serialize(&SessionData {
        e_reader_key: None,
        data: None,
        status: Some(status),
    })
    .map_err(Into::into)
}

fn session_transcript_salt(session_transcript: &SessionTranscript) -> Result<Vec<u8>, SessionEncryptionError> {
    // ISO 18013-5:2021 derives the HKDF salt from `SessionTranscriptBytes`, i.e.
    // `#6.24(bstr .cbor SessionTranscript)`, not from the raw `SessionTranscript` CBOR itself.
    // Hashing the wrong representation would produce different session keys.
    //
    // Multipaz takes encoded transcript bytes at its API boundary. We keep the typed conversion
    // here so the ISO-required `SessionTranscriptBytes` representation stays explicit in `mdoc`.
    let transcript_bytes = SessionTranscriptBytes::from(session_transcript.clone());
    Ok(sha256(cbor_serialize(&transcript_bytes)?.as_slice()))
}

fn encrypt_session_data(key: &[u8], iv: [u8; 12], plaintext: &[u8]) -> Result<Vec<u8>, SessionEncryptionError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| SessionEncryptionError::InvalidSessionKeyLength)?;
    cipher
        .encrypt(Nonce::from_slice(&iv), plaintext)
        .map_err(|_| SessionEncryptionError::EncryptionFailed)
}

fn decrypt_session_data(key: &[u8], iv: [u8; 12], ciphertext: &[u8]) -> Result<Vec<u8>, SessionEncryptionError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| SessionEncryptionError::InvalidSessionKeyLength)?;
    cipher
        .decrypt(Nonce::from_slice(&iv), ciphertext)
        .map_err(|_| SessionEncryptionError::DecryptionFailed)
}

fn session_iv(identifier: u32, counter: u32) -> [u8; 12] {
    let mut iv = [0u8; 12];
    iv[4..8].copy_from_slice(&identifier.to_be_bytes());
    iv[8..12].copy_from_slice(&counter.to_be_bytes());
    iv
}

#[cfg(test)]
mod tests {
    use p256::SecretKey;
    use p256::ecdsa::VerifyingKey;
    use p256::elliptic_curve::sec1::ToEncodedPoint;

    use super::EReaderKeyBytes;
    use super::SessionEncryption;
    use super::SessionRole;
    use super::SessionStatus;
    use super::extract_e_reader_key;
    use crate::iso::engagement::SessionTranscript;
    use crate::utils::cose::CoseKey;
    use crate::utils::serialization::TaggedBytes;

    fn secret_key(byte: u8) -> p256::SecretKey {
        SecretKey::from_slice(&[byte; 32]).unwrap()
    }

    fn reader_key_bytes(secret_key: &p256::SecretKey) -> EReaderKeyBytes {
        let public_key = secret_key.public_key();
        let encoded_point = public_key.to_encoded_point(false);
        let verifying_key = VerifyingKey::from_encoded_point(&encoded_point).unwrap();
        let cose_key: CoseKey = (&verifying_key).try_into().unwrap();
        TaggedBytes(cose_key)
    }

    #[test]
    fn extracts_reader_key_from_session_establishment_message() {
        let e_device_key = secret_key(1);
        let e_reader_key = secret_key(2);
        let e_reader_key_bytes = reader_key_bytes(&e_reader_key);
        let session_transcript = SessionTranscript::new_qr(e_reader_key_bytes.0.clone(), None);
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

        let extracted_key = extract_e_reader_key(&session_establishment_message).unwrap();

        assert_eq!(extracted_key, e_reader_key_bytes);
    }

    #[test]
    fn decrypts_reader_messages_and_encrypts_device_messages() {
        let e_device_key = secret_key(1);
        let e_reader_key = secret_key(2);
        let e_reader_key_bytes = reader_key_bytes(&e_reader_key);
        let session_transcript = SessionTranscript::new_qr(e_reader_key_bytes.0.clone(), None);

        let holder_session = SessionEncryption::new(
            SessionRole::Mdoc,
            &e_device_key,
            &e_reader_key.public_key(),
            &session_transcript,
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
        let decrypted_reader_message = holder_session.decrypt(&reader_message).unwrap();
        assert_eq!(decrypted_reader_message.data, Some(b"device-request".to_vec()));
        assert_eq!(decrypted_reader_message.status, None);

        let device_message = holder_session
            .encrypt(b"device-response", SessionStatus::Termination)
            .unwrap();
        let decrypted_device_message = reader_session.decrypt(&device_message).unwrap();
        assert_eq!(decrypted_device_message.data, Some(b"device-response".to_vec()));
        assert_eq!(decrypted_device_message.status, Some(SessionStatus::Termination));
    }

    #[test]
    fn decrypts_status_only_message() {
        let e_device_key = secret_key(1);
        let e_reader_key = secret_key(2);
        let e_reader_key_bytes = reader_key_bytes(&e_reader_key);
        let session_transcript = SessionTranscript::new_qr(e_reader_key_bytes.0.clone(), None);

        let holder_session = SessionEncryption::new(
            SessionRole::Mdoc,
            &e_device_key,
            &e_reader_key.public_key(),
            &session_transcript,
        )
        .unwrap();
        let reader_session = SessionEncryption::new(
            SessionRole::Reader,
            &e_reader_key,
            &e_device_key.public_key(),
            &session_transcript,
        )
        .unwrap();

        let status_only_message = reader_session
            .encode_message(None, Some(SessionStatus::Termination), None)
            .unwrap();
        let decrypted_message = holder_session.decrypt(&status_only_message).unwrap();
        assert_eq!(decrypted_message.data, None);
        assert_eq!(decrypted_message.status, Some(SessionStatus::Termination));
    }
}
