//! Cryptographic utilities: SHA256, ECDSA, Diffie-Hellman, HKDF, and key conversion functions.

use ciborium::value::Value;
use coset::{iana, CoseKeyBuilder, Label};
use p256::{ecdh, NistP256};
use rand::Rng;
use ring::{hkdf, hmac};
use serde::Serialize;
use sha2::Digest;
use x509_parser::nom::AsBytes;

use crate::{
    cose::CoseKey,
    serialization::{cbor_serialize, CborError},
    Error, Result,
};

#[derive(thiserror::Error, Debug)]
pub enum CryptoError {
    #[error("HKDF failed")]
    Hkdf,
    #[error("missing x coordinate")]
    KeyMissingCoordinateX,
    #[error("missing y coordinate")]
    KeyMissingCoordinateY,
    #[error("wrong key type")]
    KeyWrongType,
    #[error("missing key ID")]
    KeyMissingKeyID,
    #[error("unexpected COSE_Key label")]
    KeyUnepectedCoseLabel,
    #[error("coordinate parse failed")]
    KeyCoordinateParseFailed,
    #[error("key parse failed: {0}")]
    KeyParseFailed(#[from] ecdsa::Error),
}

pub fn sha256(bts: &[u8]) -> Vec<u8> {
    sha2::Sha256::digest(bts).to_vec()
}

/// Computes the SHA256 of the CBOR encoding of the argument.
pub fn cbor_digest<T: Serialize>(val: &T) -> std::result::Result<Vec<u8>, CborError> {
    Ok(sha256(cbor_serialize(val)?.as_ref()))
}

/// Using Diffie-Hellman and the HKDF from RFC 5869, compute a HMAC key.
pub fn dh_hmac_key(
    privkey: &ecdsa::SigningKey<NistP256>,
    pubkey: &ecdsa::VerifyingKey<NistP256>,
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
    let mut bts = vec![0u8; len];
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, sha256(salt).as_slice());

    struct HkdfLen(usize);
    impl hkdf::KeyType for HkdfLen {
        fn len(&self) -> usize {
            self.0
        }
    }

    salt.extract(input_key_material)
        .expand(&[info.as_bytes()], HkdfLen(len))
        .map_err(|_| CryptoError::Hkdf)?
        .fill(bts.as_mut_slice())
        .map_err(|_| CryptoError::Hkdf)?;

    Ok(hmac::Key::new(hmac::HMAC_SHA256, bts.as_slice()))
}

impl TryFrom<&ecdsa::VerifyingKey<NistP256>> for CoseKey {
    type Error = Error;
    fn try_from(key: &ecdsa::VerifyingKey<NistP256>) -> std::result::Result<Self, Self::Error> {
        let encoded_point = key.to_encoded_point(false);
        let x = encoded_point.x().ok_or(CryptoError::KeyMissingCoordinateX)?.to_vec();
        let y = encoded_point.y().ok_or(CryptoError::KeyMissingCoordinateY)?.to_vec();

        Ok(CoseKey(
            CoseKeyBuilder::new_ec2_pub_key(iana::EllipticCurve::P_256, x, y).build(),
        ))
    }
}

impl TryFrom<&CoseKey> for ecdsa::VerifyingKey<NistP256> {
    type Error = Error;
    fn try_from(key: &CoseKey) -> Result<Self> {
        if key.0.kty != coset::RegisteredLabel::Assigned(iana::KeyType::EC2) {
            return Err(CryptoError::KeyWrongType.into());
        }

        let keyid = key.0.params.get(0).ok_or(CryptoError::KeyMissingKeyID)?;
        if *keyid != (Label::Int(-1), Value::Integer(1.into())) {
            return Err(CryptoError::KeyWrongType.into());
        }

        let x = key.0.params.get(1).ok_or(CryptoError::KeyMissingCoordinateX)?;
        if x.0 != Label::Int(-2) {
            return Err(CryptoError::KeyUnepectedCoseLabel.into());
        }
        let y = key.0.params.get(2).ok_or(CryptoError::KeyMissingCoordinateY)?;
        if y.0 != Label::Int(-3) {
            return Err(CryptoError::KeyUnepectedCoseLabel.into());
        }

        Ok(ecdsa::VerifyingKey::<NistP256>::from_encoded_point(
            &ecdsa::EncodedPoint::<NistP256>::from_affine_coordinates(
                x.1.as_bytes()
                    .ok_or(CryptoError::KeyCoordinateParseFailed)?
                    .as_bytes()
                    .into(),
                y.1.as_bytes()
                    .ok_or(CryptoError::KeyCoordinateParseFailed)?
                    .as_bytes()
                    .into(),
                false,
            ),
        )
        .map_err(CryptoError::KeyParseFailed)?)
    }
}

pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut output = vec![0u8; len];
    rand::thread_rng().fill(&mut output[..]);
    output
}
