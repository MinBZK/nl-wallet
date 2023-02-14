//! An implementation of the algorithm from section 6.2.3, "Establishment of the PIN public key", of the SAD
//! of the NL wallet.
//!
//! This deterministic algorithm converts a salt and the user's PIN to an ECDSA private key. The corresponding public
//! key is registered at the wallet provider's Account Server (AS). When the wallet wishes to authenticate to the AS,
//! the AS sends a challenge. The wallet then uses this algorithm to (1) convert the salt and PIN to the ECDSA private
//! key, and (2) use that to sign the AS's challenge. The AS accepts if the signature verifies against the user's public
//! key.
//!
//! The usage of ECDSA in this fashion makes this protocol resistant against replay attacks: an observer of the
//! communication between the wallet and AS is unable to use what it sees to authenticate towards the AS as the user.
//!
//! This module offers three functions:
//! - [`new_pin_salt()`] returns a new salt that should be called once when registering to the wallet provider and
//!   stored afterwards, for future invocations of the two functions below.
//! - The [`PinKey<'a>`] struct, which contains the salt and the PIN, and has methods to compute signatures and the
//!   public key (by first converting the user's PIN and salt to an ECDSA private key).

use anyhow::{anyhow, Result};
use p256::{
    ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey},
    elliptic_curve::{
        bigint::{Limb, U384},
        ops::Reduce,
        Curve,
    },
    NistP256, Scalar, SecretKey, U256,
};
use ring::hkdf;

use crate::utils::random_bytes;

use super::HWBoundSigningKey;

/// All PIN data needed to compute signatures. Implements [`HWBoundSigningKey`] such that the ECDSA private key is
/// guaranteed to be dropped from memory when [`PinKey::try_sign()`] returns.
pub struct PinKey<'a> {
    pub pin: &'a str,
    pub salt: &'a [u8],
}

impl<'a> Signer<Signature> for PinKey<'a> {
    fn try_sign(&self, msg: &[u8]) -> std::result::Result<Signature, p256::ecdsa::Error> {
        Ok(pin_private_key(&self.salt, &self.pin)
            .map_err(p256::ecdsa::Error::from_source)?
            .sign(msg))
    }
}

impl<'a> HWBoundSigningKey for PinKey<'a> {
    fn verifying_key(&self) -> VerifyingKey {
        pin_private_key(&self.salt, &self.pin)
            .expect("pin private key computation failed")
            .verifying_key()
    }
}

/// Return a new salt, for use as the first parameter to [`sign_with_pin_key()`] and [`pin_public_key()`].
pub fn new_pin_salt() -> Vec<u8> {
    // Note: when passed to the HKDF function, the variable `salt` does not act as the salt but instead as the input key
    // material. The HKDF salt parameter is left empty. From a cryptographic perspective, what we call "salt" here should
    // really be called "key" or "input_key_material" or something, but we also already have a PIN private key and a
    // corresponding PIN public keys. So in the naming of things we would end up with confusingly many "keys".
    random_bytes(32)
}

/// Given a salt and a PIN, derive an ECDSA private key and return the corresponding public key.
fn pin_public_key(salt: &[u8], pin: &str) -> Result<VerifyingKey> {
    Ok(pin_private_key(salt, pin)?.verifying_key())
}

/// Given a salt and a PIN, derive an ECDSA private key and return it.
fn pin_private_key(salt: &[u8], pin: &str) -> Result<SigningKey> {
    // The `salt` parameter is really the IKM (input key material) of the HKDF, see the comment in `new_pin_salt()`.
    let hkdf = hkdf(salt, b"", pin, 256 / 8 + 8)?;
    let scalar = bytes_to_ecdsa_scalar(hkdf);
    Ok(SecretKey::new(Scalar::from_uint_reduced(scalar).into()).into())
}

/// Convert the specified bytes to a number suitable for use as an ECDSA private key: an (almost) uniformly distributed
/// random number between 0 and q-1 (inclusive), where q is the order of the ECDSA elliptic curve. This is done by
/// parsing the input bytes to an integer I and returning  `1 + I mod (q-1)`.
fn bytes_to_ecdsa_scalar(mut bts: Vec<u8>) -> U256 {
    // If this is not the case, the output won't be distributed sufficiently close to uniformly random.
    assert!(bts.len() >= 256 / 8 + 8);

    // For parsing the HKDF output as big-endian bytes to an integer, prepend zeroes so that it becomes
    // the size required by the U384 type (384 bits).
    let mut vec = vec![0u8; 384 / 8 - bts.len()];
    vec.append(&mut bts);
    let bts = vec.as_slice();

    let q = u256_to_u384(&NistP256::ORDER);
    let int = U384::from_be_slice(bts)
        .reduce(&q.sub_mod(&U384::ONE, &q))
        .unwrap()
        .add_mod(&U384::ONE, &q);

    u384_to_u256(&int)
}

/// Compute the HKDF from [RFC 5869](https://tools.ietf.org/html/rfc5869).
fn hkdf(input_key_material: &[u8], salt: &[u8], info: &str, len: usize) -> Result<Vec<u8>> {
    struct HkdfLen(usize);
    impl hkdf::KeyType for HkdfLen {
        fn len(&self) -> usize {
            self.0
        }
    }

    let mut bts = vec![0u8; len];
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, salt);

    salt.extract(input_key_material)
        .expand(&[info.as_bytes()], HkdfLen(len))
        .map_err(|e| anyhow!("hkdf expand failed: {e}"))?
        .fill(bts.as_mut_slice())
        .map_err(|e| anyhow!("hkdf fill failed: {e}"))?;

    Ok(bts)
}

// The U... bigint types (U256 and U384) offer no API to convert them from one size
// to the other, necessitating these conversion methods.
fn u256_to_u384(x: &U256) -> U384 {
    let mut limbs = x.limbs().to_vec();
    limbs.append(&mut vec![Limb(0); (384 - 256) / Limb::BIT_SIZE]);
    U384::new(limbs.try_into().unwrap())
}
fn u384_to_u256(x: &U384) -> U256 {
    U256::new(x.limbs()[..256 / Limb::BIT_SIZE].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use crate::{
        utils::random_bytes,
        wallet::pin_key::{
            bytes_to_ecdsa_scalar, new_pin_salt, pin_private_key, pin_public_key, u256_to_u384,
            u384_to_u256,
        },
    };
    use anyhow::Result;
    use p256::{
        ecdsa::signature::{Signer, Verifier},
        elliptic_curve::{
            bigint::{ArrayEncoding, NonZero, Random, RandomMod, Wrapping},
            rand_core::OsRng,
            Curve,
        },
        NistP256, U256,
    };

    #[test]
    fn test_conversion() {
        let x = U256::random(&mut OsRng);
        assert_eq!(x, u384_to_u256(&u256_to_u384(&x)));
        assert_eq!(
            NistP256::ORDER,
            u384_to_u256(&u256_to_u384(&NistP256::ORDER))
        );
    }

    #[test]
    fn test_bytes_to_pin_scalar() {
        // If x < NistP256::ORDER - 1, then bytes_to_pin_scalar() applied to the bytes of x should return x + 1.
        let x = U256::random_mod(
            &mut OsRng,
            &NonZero::new((Wrapping(NistP256::ORDER) - Wrapping(U256::from(2u8))).0).unwrap(),
        );
        let scalar = bytes_to_ecdsa_scalar(u256_to_u384(&x).to_be_byte_array().to_vec());
        assert_eq!(Wrapping(x) + Wrapping(U256::ONE), Wrapping(scalar));
    }

    #[test]
    fn test_pin_private_key() -> Result<()> {
        let salt = new_pin_salt();

        let privkey = pin_private_key(salt.as_slice(), "123456")?;
        let same = pin_private_key(salt.as_slice(), "123456")?;
        let different_salt = pin_private_key(random_bytes(32).as_slice(), "123456")?;
        let different_pin = pin_private_key(salt.as_slice(), "654321")?;

        assert_eq!(privkey, same);
        assert_ne!(privkey, different_salt);
        assert_ne!(privkey, different_pin);

        Ok(())
    }

    #[test]
    fn it_works() -> Result<()> {
        let pin = "123456";
        let salt = new_pin_salt();
        let challenge = b"challenge";

        let public_key = pin_public_key(salt.as_slice(), pin)?;
        let response = pin_private_key(salt.as_slice(), pin)?.try_sign(challenge)?;

        public_key.verify(challenge, &response)?;

        Ok(())
    }
}
