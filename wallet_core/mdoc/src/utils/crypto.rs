//! Cryptographic utilities: SHA256, ECDSA, Diffie-Hellman, HKDF, and key conversion functions.

use aes_gcm::{
    aead::{Aead, Nonce},
    Aes256Gcm, Key, KeyInit,
};
use ciborium::value::Value;
use coset::{iana, CoseKeyBuilder, Label};
use derive_more::Debug;
use p256::{ecdh, ecdsa::VerifyingKey, EncodedPoint, PublicKey, SecretKey};
use ring::hmac;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_bytes::ByteBuf;
use x509_parser::nom::AsBytes;

use error_category::ErrorCategory;
use wallet_common::utils::{hkdf, sha256};

use crate::{
    utils::{
        cose::CoseKey,
        serialization::{cbor_serialize, CborError, TaggedBytes},
    },
    CipherSuiteIdentifier, Result, Security, SecurityKeyed, SessionData, SessionTranscript,
};

use super::serialization::cbor_deserialize;

#[derive(thiserror::Error, Debug, ErrorCategory)]
pub enum CryptoError {
    #[error("HKDF failed")]
    #[category(critical)]
    Hkdf,
    #[error("missing coordinate")]
    #[category(critical)]
    KeyMissingCoordinate,
    #[error("wrong key type")]
    #[category(critical)]
    KeyWrongType,
    #[error("missing key ID")]
    #[category(critical)]
    KeyMissingKeyID,
    #[error("unexpected COSE_Key label")]
    #[category(critical)]
    KeyUnepectedCoseLabel,
    #[error("coordinate parse failed")]
    #[category(critical)]
    KeyCoordinateParseFailed,
    #[error("key parse failed: {0}")]
    #[category(pd)]
    KeyParseFailed(#[from] p256::ecdsa::Error),
    #[error("AES encryption/decryption failed")]
    #[category(critical)]
    Aes,
    #[error("AES encryption/decryption failed: missing ciphertext")]
    #[category(critical)]
    MissingCiphertext,
}

/// Computes the SHA256 of the CBOR encoding of the argument.
pub fn cbor_digest<T: Serialize>(val: &T) -> std::result::Result<Vec<u8>, CborError> {
    let digest = sha256(cbor_serialize(val)?.as_ref());
    Ok(digest)
}

/// Using Diffie-Hellman and the HKDF from RFC 5869, compute a HMAC key.
pub fn dh_hmac_key(privkey: &SecretKey, pubkey: &PublicKey, salt: &[u8], info: &str, len: usize) -> Result<hmac::Key> {
    let dh = ecdh::diffie_hellman(privkey.to_nonzero_scalar(), pubkey.as_affine());
    hmac_key(dh.raw_secret_bytes().as_ref(), salt, info, len)
}

/// Using the HKDF from RFC 5869, compute a HMAC key.
pub fn hmac_key(input_key_material: &[u8], salt: &[u8], info: &str, len: usize) -> Result<hmac::Key> {
    let bts = hkdf(input_key_material, sha256(salt).as_slice(), info, len).map_err(|_| CryptoError::Hkdf)?;
    let key = hmac::Key::new(hmac::HMAC_SHA256, &bts);
    Ok(key)
}

impl TryFrom<&VerifyingKey> for CoseKey {
    type Error = CryptoError;
    fn try_from(key: &VerifyingKey) -> std::result::Result<Self, Self::Error> {
        let encoded_point = key.to_encoded_point(false);
        let x = encoded_point.x().ok_or(CryptoError::KeyMissingCoordinate)?.to_vec();
        let y = encoded_point.y().ok_or(CryptoError::KeyMissingCoordinate)?.to_vec();

        let key = CoseKey(CoseKeyBuilder::new_ec2_pub_key(iana::EllipticCurve::P_256, x, y).build());
        Ok(key)
    }
}

impl TryFrom<&CoseKey> for VerifyingKey {
    type Error = CryptoError;
    fn try_from(key: &CoseKey) -> std::result::Result<Self, Self::Error> {
        if key.0.kty != coset::RegisteredLabel::Assigned(iana::KeyType::EC2) {
            return Err(CryptoError::KeyWrongType);
        }

        let keyid = key.0.params.first().ok_or(CryptoError::KeyMissingKeyID)?;
        if *keyid != (Label::Int(-1), Value::Integer(1.into())) {
            return Err(CryptoError::KeyWrongType);
        }

        let x = key.0.params.get(1).ok_or(CryptoError::KeyMissingCoordinate)?;
        if x.0 != Label::Int(-2) {
            return Err(CryptoError::KeyUnepectedCoseLabel);
        }
        let y = key.0.params.get(2).ok_or(CryptoError::KeyMissingCoordinate)?;
        if y.0 != Label::Int(-3) {
            return Err(CryptoError::KeyUnepectedCoseLabel);
        }

        let key = VerifyingKey::from_encoded_point(&EncodedPoint::from_affine_coordinates(
            x.1.as_bytes()
                .ok_or(CryptoError::KeyCoordinateParseFailed)?
                .as_bytes()
                .into(),
            y.1.as_bytes()
                .ok_or(CryptoError::KeyCoordinateParseFailed)?
                .as_bytes()
                .into(),
            false,
        ))
        .map_err(CryptoError::KeyParseFailed)?;
        Ok(key)
    }
}

impl TryFrom<&PublicKey> for Security {
    type Error = CryptoError;

    fn try_from(value: &PublicKey) -> std::result::Result<Self, Self::Error> {
        let cose_key: CoseKey = (&VerifyingKey::from(value)).try_into()?;
        let security = SecurityKeyed {
            cipher_suite_identifier: CipherSuiteIdentifier::P256,
            e_sender_key_bytes: cose_key.into(),
        }
        .into();
        Ok(security)
    }
}

impl TryFrom<&Security> for PublicKey {
    type Error = CryptoError;

    fn try_from(value: &Security) -> std::result::Result<Self, Self::Error> {
        let key: VerifyingKey = (&value.0.e_sender_key_bytes.0).try_into()?;
        Ok(key.into())
    }
}

/// Key for encrypting/decrypting [`SessionData`] instances containing encrypted mdoc disclosure protocol messages.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionKey {
    #[debug(skip)]
    key: ByteBuf,
    user: SessionKeyUser,
}

/// Identifies which agent uses the [`SessionKey`] to encrypt its messages.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKeyUser {
    Reader,
    Device,
}

impl SessionKey {
    /// Return a new [`SessionKey`] derived using Diffie-Hellman and a Key Derivation Function (KDF),
    /// as specified by the standard.
    pub fn new(
        privkey: &SecretKey,
        pubkey: &PublicKey,
        session_transcript: &SessionTranscript,
        user: SessionKeyUser,
    ) -> Result<Self> {
        let dh = ecdh::diffie_hellman(privkey.to_nonzero_scalar(), pubkey.as_affine());
        let salt = sha256(&cbor_serialize(&TaggedBytes(session_transcript))?);
        let user_str = match user {
            SessionKeyUser::Reader => "SKReader",
            SessionKeyUser::Device => "SKDevice",
        };
        let key = hkdf(dh.raw_secret_bytes(), &salt, user_str, 32).map_err(|_| CryptoError::Hkdf)?;
        let key = SessionKey {
            key: ByteBuf::from(key),
            user,
        };
        Ok(key)
    }
}

impl SessionData {
    /// Construct a nonce for AES GCM encryption as specified by the standard.
    fn nonce(user: SessionKeyUser) -> Nonce<Aes256Gcm> {
        let mut nonce = vec![0u8; 12];

        if user == SessionKeyUser::Device {
            nonce[7] = 1; // the 8th byte indicates the user (0 = reader, 1 = device)
        }

        // The final byte is the message count, starting at one.
        // We will support sending a maximum of 1 message per sender.
        nonce[11] = 1;

        *Nonce::<Aes256Gcm>::from_slice(&nonce)
    }

    pub fn serialize_and_encrypt<T: Serialize>(data: &T, key: &SessionKey) -> Result<Self> {
        Self::encrypt(&cbor_serialize(data)?, key)
    }

    pub fn encrypt(data: &[u8], key: &SessionKey) -> Result<Self> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.key.as_bytes()));
        let ciphertext = cipher
            .encrypt(&Self::nonce(key.user), data)
            .map_err(|_| CryptoError::Aes)?;

        Ok(SessionData {
            data: Some(ByteBuf::from(ciphertext)),
            status: None,
        })
    }

    pub fn decrypt(&self, key: &SessionKey) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.key.as_bytes()));
        let plaintext = cipher
            .decrypt(
                &Self::nonce(key.user),
                self.data.as_ref().ok_or(CryptoError::MissingCiphertext)?.as_bytes(),
            )
            .map_err(|_| CryptoError::Aes)?;
        Ok(plaintext)
    }

    pub fn decrypt_and_deserialize<T: DeserializeOwned>(&self, key: &SessionKey) -> Result<T> {
        let parsed = cbor_deserialize(self.decrypt(key)?.as_bytes())?;
        Ok(parsed)
    }
}

#[cfg(test)]
mod test {
    use p256::SecretKey;
    use rand_core::OsRng;

    use serde::{Deserialize, Serialize};

    use crate::{examples::Example, DeviceAuthenticationBytes, SessionData};

    use super::{SessionKey, SessionKeyUser};

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct ToyMessage {
        number: u8,
        string: String,
    }
    impl Default for ToyMessage {
        fn default() -> Self {
            Self {
                number: 42,
                string: "Hello, world!".to_string(),
            }
        }
    }

    #[test]
    fn session_data_encryption() {
        let plaintext = ToyMessage::default();

        let device_privkey = SecretKey::random(&mut OsRng);
        let reader_pubkey = SecretKey::random(&mut OsRng).public_key();

        let key = SessionKey::new(
            &device_privkey,
            &reader_pubkey,
            &DeviceAuthenticationBytes::example().0 .0.session_transcript,
            SessionKeyUser::Device,
        )
        .unwrap();

        let session_data = SessionData::serialize_and_encrypt(&plaintext, &key).unwrap();
        assert!(session_data.data.is_some());
        assert!(session_data.status.is_none());

        let decrypted = session_data.decrypt_and_deserialize(&key).unwrap();
        assert_eq!(plaintext, decrypted);
    }
}
