//! Cryptographic utilities: SHA256, ECDSA, Diffie-Hellman, HKDF, and key conversion functions.

use cose::CoseKeyConversionError;
use crypto::utils::hkdf;
use crypto::utils::sha256;
use derive_more::Debug;
use error_category::ErrorCategory;
use p256::PublicKey;
use p256::SecretKey;
use p256::ecdh;
use p256::ecdsa::VerifyingKey;
use ring::hmac;
use serde::Serialize;

use crate::CipherSuiteIdentifier;
use crate::Result;
use crate::Security;
use crate::SecurityKeyed;
use crate::utils::cose::CoseKey;
use crate::utils::serialization::CborError;
use crate::utils::serialization::cbor_serialize;

#[derive(thiserror::Error, Debug, ErrorCategory)]
pub enum CryptoError {
    #[error("HKDF failed")]
    #[category(critical)]
    Hkdf,
    #[error("COSE key conversion failed: {0}")]
    #[category(defer)]
    CoseKey(#[from] CoseKeyConversionError),
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
    let key = hmac::Key::new(hmac::HMAC_SHA256, bts.as_ref());
    Ok(key)
}

impl TryFrom<&PublicKey> for Security {
    type Error = CryptoError;

    fn try_from(value: &PublicKey) -> std::result::Result<Self, Self::Error> {
        let cose_key: CoseKey = (&VerifyingKey::from(value)).try_into().map_err(CryptoError::from)?;
        let security = SecurityKeyed {
            cipher_suite_identifier: CipherSuiteIdentifier::P256,
            e_device_key_bytes: cose_key.into(),
        }
        .into();
        Ok(security)
    }
}

impl TryFrom<&Security> for PublicKey {
    type Error = CryptoError;

    fn try_from(value: &Security) -> std::result::Result<Self, Self::Error> {
        let key: VerifyingKey = (&value.0.e_device_key_bytes.0).try_into().map_err(CryptoError::from)?;
        Ok(key.into())
    }
}
