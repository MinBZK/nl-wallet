use std::error::Error;
use std::hash::Hash;

use aes_gcm::Aes256Gcm;
use aes_gcm::Key;
use aes_gcm::KeySizeUser;
use aes_gcm::Nonce;
use aes_gcm::aead::Aead;
use derive_more::AsRef;
use derive_more::From;
use derive_more::Into;
use nutype::nutype;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;
use rsa::traits::PublicKeyParts;
use serde::Deserialize;
use serde::Serialize;
use serde::de;

use crate::utils;

#[derive(Debug, thiserror::Error)]
pub enum PublicKeyError {
    #[error("invalid RSA key size: {0}")]
    UnsupportedRsaKeySize(usize),
}

#[nutype(derive(Debug, Clone, AsRef, TryFrom, Into, PartialEq, Eq, Hash, Serialize, Deserialize), validate(predicate = |k| k.size() == 256))]
struct Rsa2048PublicKey(rsa::RsaPublicKey);

#[nutype(derive(Debug, Clone, AsRef, TryFrom, Into, PartialEq, Eq, Hash, Serialize, Deserialize), validate(predicate = |k| k.size() == 512))]
struct Rsa4096PublicKey(rsa::RsaPublicKey);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicKey {
    P256(p256::ecdsa::VerifyingKey),
    P384(p384::ecdsa::VerifyingKey),
    P521(p521::ecdsa::VerifyingKey),
    RSA2048(Rsa2048PublicKey),
    RSA4096(Rsa4096PublicKey),
}

impl Hash for PublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            PublicKey::P256(verifying_key) => verifying_key.to_sec1_bytes().hash(state),
            PublicKey::P384(verifying_key) => verifying_key.to_sec1_bytes().hash(state),
            PublicKey::P521(verifying_key) => verifying_key.to_sec1_bytes().hash(state),
            PublicKey::RSA2048(rsa_public_key) => rsa_public_key.as_ref().hash(state),
            PublicKey::RSA4096(rsa_public_key) => rsa_public_key.as_ref().hash(state),
        }
    }
}

impl From<p256::ecdsa::VerifyingKey> for PublicKey {
    fn from(key: p256::ecdsa::VerifyingKey) -> Self {
        Self::P256(key)
    }
}

impl From<p384::ecdsa::VerifyingKey> for PublicKey {
    fn from(key: p384::ecdsa::VerifyingKey) -> Self {
        Self::P384(key)
    }
}

impl From<p521::ecdsa::VerifyingKey> for PublicKey {
    fn from(key: p521::ecdsa::VerifyingKey) -> Self {
        Self::P521(key)
    }
}

impl TryFrom<rsa::RsaPublicKey> for PublicKey {
    type Error = PublicKeyError;

    fn try_from(key: rsa::RsaPublicKey) -> Result<Self, Self::Error> {
        // size is in bytes
        match key.size() {
            256 => Ok(Self::RSA2048(
                Rsa2048PublicKey::try_from(key).expect("should be 2048 bits"),
            )),
            512 => Ok(Self::RSA4096(
                Rsa4096PublicKey::try_from(key).expect("should be 4096 bits"),
            )),
            n => Err(PublicKeyError::UnsupportedRsaKeySize(n)),
        }
    }
}

#[trait_variant::make(EcdsaKeySend: Send)]
pub trait EcdsaKey {
    type Error: Error + Send + Sync + 'static;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error>;

    /// Attempt to sign the given message, returning a digital signature on
    /// success, or an error if something went wrong.
    ///
    /// The main intended use case for signing errors is when communicating
    /// with external signers, e.g. cloud KMS, HSMs, or other hardware tokens.
    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error>;
}

/// Contract for ECDSA private keys which are short-lived and deterministically derived from a PIN.
pub trait EphemeralEcdsaKey: EcdsaKey {}

/// Contract for ECDSA private keys that are stored in some form of secure hardware from which they cannot be extracted,
/// e.g., a HSM, Android's TEE/StrongBox, or Apple's SE.
pub trait SecureEcdsaKey: EcdsaKey {}

// The `SigningKey` is an `EcdsaKey` but not a `SecureEcdsaKey` (except in mock/tests).
impl EcdsaKeySend for p256::ecdsa::SigningKey {
    type Error = p256::ecdsa::Error;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        Ok(*self.verifying_key())
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        p256::ecdsa::signature::Signer::try_sign(self, msg)
    }
}

#[trait_variant::make(Send)]
pub trait EncryptionKey {
    type Error: Error + Send + Sync + 'static;

    async fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error>;
    async fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error>;
}

/// Contract for encryption keys suitable for use in the wallet, e.g. for securely storing the database key.
/// Should be sufficiently secured e.g. through Android's TEE/StrongBox or Apple's SE.
pub trait SecureEncryptionKey: EncryptionKey {}

// `Aes256Gcm` is an `EncryptionKey` but not a `SecureEncryptionKey` (except in mock/tests).
impl EncryptionKey for Aes256Gcm {
    type Error = aes_gcm::Error;

    async fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error> {
        // Generate a random nonce
        let nonce_bytes = utils::random_bytes(12);
        let nonce = Nonce::from_slice(&nonce_bytes); // 96-bits; unique per message

        // Encrypt the provided message
        let encrypted_msg = <Aes256Gcm as Aead>::encrypt(self, nonce, msg)?;

        // concatenate nonce with encrypted payload
        let result = nonce_bytes.into_iter().chain(encrypted_msg).collect();

        Ok(result)
    }

    async fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error> {
        // Re-create the nonce from the first 12 bytes
        let nonce = Nonce::from_slice(&msg[..12]);

        // Decrypt the provided message with the retrieved nonce
        <Aes256Gcm as Aead>::decrypt(self, nonce, &msg[12..])
    }
}

/// This trait is included with keys that are uniquely identified by a string.
pub trait WithIdentifier {
    fn identifier(&self) -> &str;
}

pub trait WithVerifyingKey {
    type Error: Error + Send + Sync + 'static;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error>;
}

impl<T: EcdsaKey> WithVerifyingKey for T {
    type Error = T::Error;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.verifying_key().await
    }
}

/// Contract for ECDSA private keys suitable for credentials.
/// Should be sufficiently secured e.g. through a HSM, or Android's TEE/StrongBox or Apple's SE.
pub trait CredentialEcdsaKey: WithVerifyingKey + WithIdentifier {
    // from WithIdentifier: identifier()
    // from WithVerifyingKey: verifying_key()
}

/// A newtype around `Vec<u8>` that represent an assertion generated by Apple AppAttest.
/// It is to be treated as opaque bytes until received by the server.
#[derive(Debug, Clone, From, Into, AsRef, Serialize, Deserialize)]
#[as_ref(forward)]
pub struct AppleAssertion(Vec<u8>);

/// Represents a symmetric encryption key.
///
/// The `SymmetricKey` struct is used to encapsulate the raw bytes of a symmetric key,
/// which can be used in cryptographic operations such as encryption and decryption.
/// It can be deserialized from a hex-encoded string, e.g. `"01020304"`.
///
/// # Attributes
/// - `bytes` (`Vec<u8>`): A vector of bytes representing the symmetric key.
///
/// # Example
/// ```rust
/// use crypto::SymmetricKey;
///
/// let key_bytes = vec![0x01, 0x02, 0x03, 0x04];
/// let symmetric_key: SymmetricKey = key_bytes.into();
/// ```
#[derive(Clone, From, Into, AsRef)]
pub struct SymmetricKey {
    bytes: Vec<u8>,
}

impl SymmetricKey {
    pub fn key<B>(&self) -> &Key<B>
    where
        B: KeySizeUser,
    {
        Key::<B>::from_slice(self.bytes.as_slice())
    }
}

impl<'de> Deserialize<'de> for SymmetricKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)
            .map(hex::decode)?
            .map(Into::into)
            .map_err(de::Error::custom)
    }
}

#[cfg(any(test, feature = "mock_secure_keys"))]
mod mock_secure_keys {
    use aes_gcm::Aes256Gcm;
    use p256::ecdsa::SigningKey;

    use super::EphemeralEcdsaKey;
    use super::SecureEcdsaKey;
    use super::SecureEncryptionKey;

    impl EphemeralEcdsaKey for SigningKey {}
    impl SecureEcdsaKey for SigningKey {}

    impl SecureEncryptionKey for Aes256Gcm {}
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;
    use std::hash::Hasher;

    use ecdsa::elliptic_curve::Generate;
    use rand::thread_rng;
    use rsa::RsaPrivateKey;
    use rstest::rstest;

    use super::PublicKey;

    fn hash(key: &PublicKey) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    // The Hash-Eq contract: k1 == k2 implies hash(k1) == hash(k2).
    #[rstest]
    #[case::p256(PublicKey::P256(*p256::ecdsa::SigningKey::generate().verifying_key()))]
    #[case::p384(PublicKey::P384(*p384::ecdsa::SigningKey::generate().verifying_key()))]
    #[case::p521(PublicKey::P521(*p521::ecdsa::SigningKey::generate().verifying_key()))]
    #[case::rsa2048(PublicKey::try_from(RsaPrivateKey::new(&mut thread_rng(), 2048).unwrap().to_public_key()).unwrap())]
    #[case::rsa4096(PublicKey::try_from(RsaPrivateKey::new(&mut thread_rng(), 4096).unwrap().to_public_key()).unwrap())]
    fn hash_eq_contract(#[case] key: PublicKey) {
        assert_eq!(key, key.clone());
        assert_eq!(hash(&key), hash(&key.clone()));
    }

    #[test]
    fn different_ecdsa_variants_are_not_equal() {
        let p256_key = PublicKey::P256(*p256::ecdsa::SigningKey::generate().verifying_key());
        let p384_key = PublicKey::P384(*p384::ecdsa::SigningKey::generate().verifying_key());
        assert_ne!(p256_key, p384_key);
        assert_ne!(hash(&p256_key), hash(&p384_key));
    }

    #[test]
    fn different_rsa_variants_are_not_equal() {
        let rsa2048_key = PublicKey::RSA2048(
            RsaPrivateKey::new(&mut thread_rng(), 2048)
                .unwrap()
                .to_public_key()
                .try_into()
                .unwrap(),
        );
        let rsa4096_key = PublicKey::RSA4096(
            RsaPrivateKey::new(&mut thread_rng(), 4096)
                .unwrap()
                .to_public_key()
                .try_into()
                .unwrap(),
        );
        assert_ne!(rsa2048_key, rsa4096_key);
        assert_ne!(hash(&rsa2048_key), hash(&rsa4096_key));
    }
}
