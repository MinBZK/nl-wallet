//! Cryptographic utilities: SHA256, ECDSA, Diffie-Hellman, HKDF, and key conversion functions.

use anyhow::{anyhow, bail, Context, Result};
use ciborium::value::Value;
use coset::{iana, CoseKeyBuilder, Label};
use p256::{ecdh, NistP256};
use ring::{
    hkdf, hmac,
    rand::{self, SecureRandom},
};
use serde::Serialize;
use sha2::Digest;
use x509_parser::nom::AsBytes;

use crate::{cose::CoseKey, serialization::cbor_serialize};

pub(crate) fn sha256(bts: &[u8]) -> Vec<u8> {
    sha2::Sha256::digest(bts).to_vec()
}

/// Computes the SHA256 of the CBOR encoding of the argument.
pub(crate) fn cbor_digest<T: Serialize>(val: &T) -> Result<Vec<u8>> {
    Ok(sha256(cbor_serialize(val)?.as_ref()))
}

/// Using Diffie-Hellman and the HKDF from RFC 5869, compute a HMAC key.
pub(crate) fn dh_hmac_key(
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
pub(crate) fn hmac_key(
    input_key_material: &[u8],
    salt: &[u8],
    info: &str,
    len: usize,
) -> Result<hmac::Key> {
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
        .map_err(|e| anyhow!("hkdf expand failed: {e}"))?
        .fill(bts.as_mut_slice())
        .map_err(|e| anyhow!("hkdf fill failed: {e}"))?;

    Ok(hmac::Key::new(hmac::HMAC_SHA256, bts.as_slice()))
}

impl TryFrom<&ecdsa::VerifyingKey<NistP256>> for CoseKey {
    type Error = anyhow::Error;
    fn try_from(key: &ecdsa::VerifyingKey<NistP256>) -> std::result::Result<Self, Self::Error> {
        let encoded_point = key.to_encoded_point(false);
        let x = encoded_point.x().ok_or(anyhow!("missing x"))?.to_vec();
        let y = encoded_point.y().ok_or(anyhow!("missing y"))?.to_vec();

        Ok(CoseKey(
            CoseKeyBuilder::new_ec2_pub_key(iana::EllipticCurve::P_256, x, y).build(),
        ))
    }
}

impl TryFrom<&CoseKey> for ecdsa::VerifyingKey<NistP256> {
    type Error = anyhow::Error;
    fn try_from(key: &CoseKey) -> std::result::Result<Self, Self::Error> {
        if key.0.kty != coset::RegisteredLabel::Assigned(iana::KeyType::EC2) {
            bail!("wrong keytype")
        }

        let keyid = key
            .0
            .params
            .get(0)
            .ok_or(anyhow!("missing keyid parameter"))?;
        if *keyid != (Label::Int(-1), Value::Integer(1.into())) {
            bail!("wrong keyid")
        }

        let x = key.0.params.get(1).ok_or(anyhow!("missing x parameter"))?;
        if x.0 != Label::Int(-2) {
            bail!("unexpected label")
        }
        let y = key.0.params.get(2).ok_or(anyhow!("missing y parameter"))?;
        if y.0 != Label::Int(-3) {
            bail!("unexpected label")
        }

        ecdsa::VerifyingKey::<NistP256>::from_encoded_point(
            &ecdsa::EncodedPoint::<NistP256>::from_affine_coordinates(
                x.1.as_bytes()
                    .ok_or(anyhow!("failed to parse x parameter as bytes"))?
                    .as_bytes()
                    .into(),
                y.1.as_bytes()
                    .ok_or(anyhow!("failed to parse y parameter as bytes"))?
                    .as_bytes()
                    .into(),
                false,
            ),
        )
        .context("failed to instantiate public key")
    }
}

pub(crate) fn random_bytes(len: usize) -> Result<Vec<u8>> {
    let mut output = vec![0u8; len];
    rand::SystemRandom::new()
        .fill(&mut output[..])
        .map_err(|_| anyhow!("generating random bytes failed"))?;
    Ok(output)
}
