//! Cryptographic utilities: SHA256, ECDSA, Diffie-Hellman, HKDF, and key conversion functions.

use ciborium::value::Value;
use coset::CoseKeyBuilder;
use coset::Label;
use coset::iana;
use derive_more::Debug;
use nom::AsBytes;
use p256::EncodedPoint;
use p256::PublicKey;
use p256::SecretKey;
use p256::ecdh;
use p256::ecdsa::VerifyingKey;
use ring::hmac;
use serde::Serialize;

use crypto::utils::hkdf;
use crypto::utils::sha256;
use error_category::ErrorCategory;

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
