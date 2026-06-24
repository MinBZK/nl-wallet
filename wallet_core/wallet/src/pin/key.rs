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
//! This module offers the following:
//! - [`new_pin_salt()`] returns a new salt that should be called once when registering to the wallet provider and
//!   stored afterwards, for future invocations of the two functions below.
//! - The [`PinKey<'a>`] struct, which contains the salt and the PIN, and has methods to compute signatures and the
//!   public key (by first converting the user's PIN and salt to an ECDSA private key).

use crypto::keys::EcdsaKey;
use crypto::keys::EphemeralEcdsaKey;
use crypto::utils::KeyBytes;
use crypto::utils::hkdf;
use crypto::utils::random_bytes;
use derive_more::AsRef;
use derive_more::From;
use p256::FieldBytes;
use p256::NistP256;
use p256::SecretKey;
use p256::U256;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use p256::elliptic_curve::Curve;
use p256::elliptic_curve::bigint::ArrayEncoding;
use p256::elliptic_curve::bigint::Limb;
use p256::elliptic_curve::bigint::NonZero;
use p256::elliptic_curve::bigint::U384;
use p256::elliptic_curve::zeroize::Zeroize;
use ring::error::Unspecified as UnspecifiedRingError;
use zeroize::ZeroizeOnDrop;

/// Return a new salt, for use as the first parameter to [`sign_with_pin_key()`] and [`pin_public_key()`].
pub fn new_pin_salt() -> KeyBytes {
    // Note: when passed to the HKDF function, the variable `salt` does not act as the salt but instead as the input key
    // material. The HKDF salt parameter is left empty. From a cryptographic perspective, what we call "salt" here
    // should really be called "key" or "input_key_material" or something, but we also already have a PIN private
    // key and a corresponding PIN public keys. So in the naming of things we would end up with confusingly many
    // "keys".
    random_bytes(32).into()
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

#[derive(Clone, ZeroizeOnDrop, From, AsRef)]
#[as_ref(forward)]
#[from(String, &str)]
pub struct Pin(String);

/// All PIN data needed to compute signatures. Implements [`Signer<Signature>`] such that the ECDSA private key is
/// guaranteed to be dropped from memory when [`PinKey::try_sign()`] returns.
pub struct PinKey<'a> {
    pub pin: &'a Pin,
    pub salt: &'a KeyBytes,
}

impl PinKey<'_> {
    pub fn verifying_key(&self) -> Result<VerifyingKey, PinKeyError> {
        let signing_key = pin_private_key(&self.salt, &self.pin)?;
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
        let key = pin_private_key(&self.salt, &self.pin)?;
        let signature = p256::ecdsa::signature::Signer::sign(&key, msg);

        Ok(signature)
    }
}

impl EphemeralEcdsaKey for PinKey<'_> {}

/// Given a salt and a PIN, derive an ECDSA private key and return it.
fn pin_private_key(salt: &KeyBytes, pin: &Pin) -> Result<SigningKey, UnspecifiedRingError> {
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
    let hkdf = hkdf(salt.as_ref(), b"", pin.as_ref(), 256 / 8 + 8)?;
    let key_bytes = bytes_to_ecdsa_privkey_bytes(hkdf);

    // We need to use `SecretKey::from_bytes`, which places a copy of the private key bytes on the stack
    // without zeroizing them afterwards. So we clear the stack ourselves using `zeroize_stack()`.
    // Looking at `SecretKey::from_bytes`, it takes less than 200 bytes. We clear 1024 bytes to be
    // on the safe side.
    let key = secret_key_from_field_bytes(&key_bytes)?;
    zeroize::zeroize_stack::<1024>();

    Ok(key.into())
}

#[inline(never)]
fn secret_key_from_field_bytes(bytes: &KeyBytes) -> Result<SecretKey, UnspecifiedRingError> {
    // This error is not actually a `ring` error, but there is no need to map it to something more specific,
    // because it does not actually occur: it happens only if the scalar is larger than the P256 modulus,
    // which it isn't by how it is constructed in bytes_to_ecdsa_privkey_bytes().
    SecretKey::from_bytes(FieldBytes::from_slice(bytes.as_ref())).map_err(|_| UnspecifiedRingError)
}

/// Convert the specified bytes to a number suitable for use as an ECDSA private key: an (almost) uniformly distributed
/// random number between 0 and q-1 (inclusive), where q is the order of the ECDSA elliptic curve. This is done by
/// parsing the input bytes to an integer `i` and returning  `i mod (q-1) + 1`.
fn bytes_to_ecdsa_privkey_bytes(bts: KeyBytes) -> KeyBytes {
    // If this is not the case, the output won't be distributed sufficiently close to uniformly random.
    assert!(bts.as_ref().len() >= 256 / 8 + 8);

    // By construction `bts` does not fit into an U256, so we do the calculations using U384 instances.
    // Before converting our bytes to U384, ensure we have exactly 384 bits by prepending zeroes.
    let bts = {
        let len = 384 / 8;
        let prefix_len = len - bts.as_ref().len();
        let mut padded = vec![0u8; len];
        padded[prefix_len..].copy_from_slice(bts.as_ref());
        drop(bts); // zeroize bts
        KeyBytes::from(padded)
    };

    // Now we compute `i mod (q-1) + 1`, explicitly calling zeroize() on each intermediate where necessary.
    let mut i = U384::from_be_slice(bts.as_ref()); // Convert to U384
    drop(bts);

    // We'll need this below.
    let q = u256_to_u384(&NistP256::ORDER);

    // reduced := i mod (q-1)
    let mut reduced = i.rem(&NonZero::from_uint(q.sub_mod(&U384::ONE, &q)));
    i.zeroize();

    // plus_one := reduced + 1 = i mod (q-1) + 1
    let mut plus_one = reduced.add_mod(&U384::ONE, &q);
    reduced.zeroize();

    // Convert back to bytes
    let mut result_array = plus_one.to_be_byte_array();
    plus_one.zeroize();

    // Our scalar is now ≤ q-1 < 2^256 by construction, so its upper 16 bytes are always zero.
    // Discard them so we return 256 bits, the appropriate size for constructing a P256 private key.
    let result = KeyBytes::from(result_array[16..].to_vec());
    result_array.zeroize();

    result
}

// The U... bigint types (U256 and U384) offer no API to convert them from one size
// to the other, necessitating this conversion method.
fn u256_to_u384(x: &U256) -> U384 {
    let mut limbs = x.as_limbs().to_vec();
    limbs.append(&mut vec![Limb(0); (384 - 256) / Limb::BITS]);
    U384::new(limbs.try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use futures::FutureExt;
    use p256::ecdsa::signature::Verifier;
    use p256::elliptic_curve::bigint::ArrayEncoding;
    use p256::elliptic_curve::bigint::RandomMod;
    use p256::elliptic_curve::bigint::Wrapping;
    use rand_core::OsRng;

    use super::*;

    #[test]
    fn test_bytes_to_ecdsa_privkey_bytes() {
        // If x < NistP256::ORDER - 1, then bytes_to_ecdsa_privkey_bytes() applied to the bytes of x
        // should return x + 1.
        let x = U256::random_mod(
            &mut OsRng,
            &NonZero::new((Wrapping(NistP256::ORDER) - Wrapping(U256::from(2u8))).0).unwrap(),
        );
        let scalar_bytes = bytes_to_ecdsa_privkey_bytes(u256_to_u384(&x).to_be_byte_array().to_vec().into());
        let scalar = U256::from_be_slice(scalar_bytes.as_ref());
        assert_eq!(Wrapping(x) + Wrapping(U256::ONE), Wrapping(scalar));

        // x = ORDER - 1: (ORDER-1) mod (ORDER-1) = 0, so result = 0 + 1 = 1.
        let x = (Wrapping(NistP256::ORDER) - Wrapping(U256::ONE)).0;
        let scalar_bytes = bytes_to_ecdsa_privkey_bytes(u256_to_u384(&x).to_be_byte_array().to_vec().into());
        let scalar = U256::from_be_slice(scalar_bytes.as_ref());
        assert_eq!(scalar, U256::ONE);

        // Larger values of x just cause the result to increment due to the modular nature of the computation in
        // bytes_to_ecdsa_privkey_bytes().
        let x = NistP256::ORDER;
        let scalar_bytes = bytes_to_ecdsa_privkey_bytes(u256_to_u384(&x).to_be_byte_array().to_vec().into());
        let scalar = U256::from_be_slice(scalar_bytes.as_ref());
        assert_eq!(scalar, U256::from(2u8));
    }

    #[test]
    fn test_pin_private_key() {
        let salt = new_pin_salt();

        let privkey = pin_private_key(&salt, &"123456".into()).expect("Cannot create private key from PIN");
        let same = pin_private_key(&salt, &"123456".into()).expect("Cannot create private key from PIN");
        let different_salt =
            pin_private_key(&random_bytes(32).into(), &"123456".into()).expect("Cannot create private key from PIN");
        let different_pin = pin_private_key(&salt, &"654321".into()).expect("Cannot create private key from PIN");

        assert_eq!(privkey, same);
        assert_ne!(privkey, different_salt);
        assert_ne!(privkey, different_pin);
    }

    #[test]
    fn it_works() {
        let pin = "123456";
        let salt = new_pin_salt();
        let challenge = b"challenge";

        let pin_key = PinKey {
            pin: &pin.into(),
            salt: &salt,
        };
        let public_key = pin_key.verifying_key().expect("Cannot get public key from PIN key");
        let response = pin_key
            .try_sign(challenge)
            .now_or_never()
            .unwrap()
            .expect("Cannot sign challenge using PIN key");

        public_key
            .verify(challenge, &response)
            .expect("Cannot verify challenge using PIN public key");
    }
}
