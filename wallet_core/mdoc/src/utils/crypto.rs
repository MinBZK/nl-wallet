//! Cryptographic utilities: SHA256, ECDSA, Diffie-Hellman, HKDF, and key conversion functions.

use ciborium::value::Value;
use coset::{iana, CoseKeyBuilder, Label};
use p256::{
    ecdh,
    ecdsa::{SigningKey, VerifyingKey},
    EncodedPoint,
};
use ring::hmac;
use serde::Serialize;
use x509_parser::nom::AsBytes;

use wallet_common::utils::{hkdf, sha256};

use crate::{
    utils::{
        cose::CoseKey,
        serialization::{cbor_serialize, CborError},
    },
    Error, Result,
};

#[derive(thiserror::Error, Debug)]
pub enum CryptoError {
    #[error("HKDF failed")]
    Hkdf,
    #[error("missing coordinate")]
    KeyMissingCoordinate,
    #[error("wrong key type")]
    KeyWrongType,
    #[error("missing key ID")]
    KeyMissingKeyID,
    #[error("unexpected COSE_Key label")]
    KeyUnepectedCoseLabel,
    #[error("coordinate parse failed")]
    KeyCoordinateParseFailed,
    #[error("key parse failed: {0}")]
    KeyParseFailed(#[from] p256::ecdsa::Error),
}

/// Computes the SHA256 of the CBOR encoding of the argument.
pub fn cbor_digest<T: Serialize>(val: &T) -> std::result::Result<Vec<u8>, CborError> {
    let digest = sha256(cbor_serialize(val)?.as_ref());
    Ok(digest)
}

/// Using Diffie-Hellman and the HKDF from RFC 5869, compute a HMAC key.
pub fn dh_hmac_key(
    privkey: &SigningKey,
    pubkey: &VerifyingKey,
    salt: &[u8],
    info: &str,
    len: usize,
) -> Result<hmac::Key> {
    let dh = ecdh::diffie_hellman(privkey.as_nonzero_scalar(), pubkey.as_affine());
    hmac_key(dh.raw_secret_bytes().as_ref(), salt, info, len)
}

// TODO support no salt
/// Using the HKDF from RFC 5869, compute a HMAC key.
pub fn hmac_key(input_key_material: &[u8], salt: &[u8], info: &str, len: usize) -> Result<hmac::Key> {
    let bts = hkdf(input_key_material, sha256(salt).as_slice(), info, len).map_err(|_| CryptoError::Hkdf)?;
    let key = hmac::Key::new(hmac::HMAC_SHA256, &bts);
    Ok(key)
}

impl TryFrom<&VerifyingKey> for CoseKey {
    type Error = Error;
    fn try_from(key: &VerifyingKey) -> std::result::Result<Self, Self::Error> {
        let encoded_point = key.to_encoded_point(false);
        let x = encoded_point.x().ok_or(CryptoError::KeyMissingCoordinate)?.to_vec();
        let y = encoded_point.y().ok_or(CryptoError::KeyMissingCoordinate)?.to_vec();

        let key = CoseKey(CoseKeyBuilder::new_ec2_pub_key(iana::EllipticCurve::P_256, x, y).build());
        Ok(key)
    }
}

impl TryFrom<&CoseKey> for VerifyingKey {
    type Error = Error;
    fn try_from(key: &CoseKey) -> Result<Self> {
        if key.0.kty != coset::RegisteredLabel::Assigned(iana::KeyType::EC2) {
            return Err(CryptoError::KeyWrongType.into());
        }

        let keyid = key.0.params.get(0).ok_or(CryptoError::KeyMissingKeyID)?;
        if *keyid != (Label::Int(-1), Value::Integer(1.into())) {
            return Err(CryptoError::KeyWrongType.into());
        }

        let x = key.0.params.get(1).ok_or(CryptoError::KeyMissingCoordinate)?;
        if x.0 != Label::Int(-2) {
            return Err(CryptoError::KeyUnepectedCoseLabel.into());
        }
        let y = key.0.params.get(2).ok_or(CryptoError::KeyMissingCoordinate)?;
        if y.0 != Label::Int(-3) {
            return Err(CryptoError::KeyUnepectedCoseLabel.into());
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
