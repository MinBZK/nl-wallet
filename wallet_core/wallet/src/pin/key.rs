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

use p256::{
    ecdsa::{Signature, SigningKey, VerifyingKey},
    elliptic_curve::{
        bigint::{Limb, NonZero, U384},
        ops::Reduce,
        Curve,
    },
    NistP256, Scalar, SecretKey, U256,
};
use ring::error::Unspecified as UnspecifiedRingError;

use wallet_common::{
    keys::{EcdsaKey, EphemeralEcdsaKey},
    utils::{hkdf, random_bytes},
};

/// Return a new salt, for use as the first parameter to [`sign_with_pin_key()`] and [`pin_public_key()`].
pub fn new_pin_salt() -> Vec<u8> {
    // Note: when passed to the HKDF function, the variable `salt` does not act as the salt but instead as the input key
    // material. The HKDF salt parameter is left empty. From a cryptographic perspective, what we call "salt" here
    // should really be called "key" or "input_key_material" or something, but we also already have a PIN private
    // key and a corresponding PIN public keys. So in the naming of things we would end up with confusingly many
    // "keys".
    random_bytes(32)
}

#[derive(Debug, thiserror::Error)]
pub enum PinKeyError {
    #[error("HKDF key derivation error")]
    Hkdf(#[from] UnspecifiedRingError),
}

impl From<PinKeyError> for p256::ecdsa::Error {
    fn from(value: PinKeyError) -> Self {
        Self::from_source(value)
    }
}

/// All PIN data needed to compute signatures. Implements [`Signer<Signature>`] such that the ECDSA private key is
/// guaranteed to be dropped from memory when [`PinKey::try_sign()`] returns.
pub struct PinKey<'a> {
    pub pin: &'a str,
    pub salt: &'a [u8],
}

impl<'a> PinKey<'a> {
    pub fn new(pin: &'a str, salt: &'a [u8]) -> Self {
        PinKey { pin, salt }
    }

    pub fn verifying_key(&self) -> Result<VerifyingKey, PinKeyError> {
        let signing_key = pin_private_key(self.salt, self.pin)?;
        let verifying_key = *signing_key.verifying_key();

        Ok(verifying_key)
    }
}

impl EcdsaKey for PinKey<'_> {
    type Error = PinKeyError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.verifying_key()
    }

    async fn try_sign(&self, msg: &[u8]) -> std::result::Result<Signature, PinKeyError> {
        let key = pin_private_key(self.salt, self.pin)?;
        let signature = p256::ecdsa::signature::Signer::sign(&key, msg);

        Ok(signature)
    }
}

impl EphemeralEcdsaKey for PinKey<'_> {}

/// Given a salt and a PIN, derive an ECDSA private key and return it.
fn pin_private_key(salt: &[u8], pin: &str) -> Result<SigningKey, UnspecifiedRingError> {
    // The `salt` parameter is really the IKM (input key material) of the HKDF, see the comment in `new_pin_salt()`.
    // The reason for length 256 / 8 + 8 is as follows. The private key must be a random number between 1 and q - 1,
    // where q is the (prime) order of the ECDSA elliptic curve (its amount of elements). But hkdf() takes bytes not
    // bits as the output length parameter, so we can't specify the upper bound sufficiently granularly; the output may
    // be too large. Just reducing mod q would result in the so-called modulo bias
    // (see e.g.
    // https://research.kudelskisecurity.com/2020/07/28/the-definitive-guide-to-modulo-bias-and-how-to-avoid-it/):
    // the numbers above the upper bound are mapped onto the lower numbers, which therefore are slightly more likely to
    // be chosen.
    // This is often solved by repeatedly rejecting too large values until one obtains a number below the upper bound,
    // but this makes the execution time of the algorithm random, which might lead to time-based side channel
    // vulnerabilities. Instead, we use the following constant-time algorithm: we just reduce the severity of the modulo
    // bias effect to negligibility by making the output of hkdf() sufficienfly larger.
    // Making it larger by 8 bytes, i.e. 32 bits, is conventional.
    let hkdf = hkdf(salt, b"", pin, 256 / 8 + 8)?;
    let scalar = bytes_to_ecdsa_scalar(hkdf);
    Ok(SecretKey::new(Scalar::reduce(scalar).into()).into())
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
        .rem(&NonZero::from_uint(q.sub_mod(&U384::ONE, &q)))
        .add_mod(&U384::ONE, &q);

    u384_to_u256(&int)
}

// The U... bigint types (U256 and U384) offer no API to convert them from one size
// to the other, necessitating these conversion methods.
fn u256_to_u384(x: &U256) -> U384 {
    let mut limbs = x.as_limbs().to_vec();
    limbs.append(&mut vec![Limb(0); (384 - 256) / Limb::BITS]);
    U384::new(limbs.try_into().unwrap())
}

fn u384_to_u256(x: &U384) -> U256 {
    U256::new(x.as_limbs()[..256 / Limb::BITS].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    use p256::{
        ecdsa::signature::Verifier,
        elliptic_curve::bigint::{ArrayEncoding, Random, RandomMod, Wrapping},
    };
    use rand_core::OsRng;

    #[test]
    fn test_conversion() {
        let x = U256::random(&mut OsRng);
        assert_eq!(x, u384_to_u256(&u256_to_u384(&x)));
        assert_eq!(NistP256::ORDER, u384_to_u256(&u256_to_u384(&NistP256::ORDER)));
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
    fn test_pin_private_key() {
        let salt = new_pin_salt();

        let privkey = pin_private_key(salt.as_slice(), "123456").expect("Cannot create private key from PIN");
        let same = pin_private_key(salt.as_slice(), "123456").expect("Cannot create private key from PIN");
        let different_salt =
            pin_private_key(random_bytes(32).as_slice(), "123456").expect("Cannot create private key from PIN");
        let different_pin = pin_private_key(salt.as_slice(), "654321").expect("Cannot create private key from PIN");

        assert_eq!(privkey, same);
        assert_ne!(privkey, different_salt);
        assert_ne!(privkey, different_pin);
    }

    #[tokio::test]
    async fn it_works() {
        let pin = "123456";
        let salt = new_pin_salt();
        let challenge = b"challenge";

        let pin_key = PinKey::new(pin, &salt);
        let public_key = pin_key.verifying_key().expect("Cannot get public key from PIN key");
        let response = pin_key
            .try_sign(challenge)
            .await
            .expect("Cannot sign challenge using PIN key");

        public_key
            .verify(challenge, &response)
            .expect("Cannot verify challenge using PIN public key");
    }
}
