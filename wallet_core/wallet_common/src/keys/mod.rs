use std::error::Error;

use aes_gcm::{aead::Aead, Aes256Gcm, Nonce};
use p256::ecdsa::{Signature, VerifyingKey};

use crate::utils;

#[cfg(feature = "software_keys")]
pub mod software;
#[cfg(any(all(feature = "software_keys", test), feature = "integration_test"))]
pub mod test;

#[trait_variant::make(EcdsaKey: Send)]
pub trait LocalEcdsaKey {
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
impl EcdsaKey for p256::ecdsa::SigningKey {
    type Error = p256::ecdsa::Error;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        Ok(*self.verifying_key())
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        p256::ecdsa::signature::Signer::try_sign(self, msg)
    }
}

#[trait_variant::make(EncryptionKey: Send)]
pub trait LocalEncryptionKey {
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

/// This trait is included with keys that are uniquely identified by an string.
pub trait WithIdentifier {
    fn identifier(&self) -> &str;
}

/// This trait is implemented on keys that are stored in a particular backing store,
/// such as Android's TEE/StrongBox or Apple's SE. These keys can be constructed by
/// an identifier, with the guarantee that only one instance can exist per identifier
/// in the entire process. If the key exists within the backing store, it will be
/// retrieved on first use, otherwise a random key will be created.
///
/// The key can be deleted from the backing store by a method that consumes the type.
/// If the type is simply dropped, it will remain in the backing store.
///
/// The limitation of having only one instance per identifier codifies that there is
/// only ever one owner of this key. If multiple instances with the same identifier
/// could be created, this could lead to undefined behaviour when the owner of one
/// of the types deletes the backing store key.
///
/// NB: Any type that implements `StoredByIdentifier` should probably not implement
///     `Clone`, as this would circumvent the uniqueness of the instance.
#[trait_variant::make(StoredByIdentifier: Send)]
pub trait LocalStoredByIdentifier: WithIdentifier {
    type Error: Error + Send + Sync + 'static;

    /// Creates a unique instance with the specified identifier. If an instance
    /// already exist with this identifier, `None` will be returned.
    fn new_unique(identifier: &str) -> Option<Self>
    where
        Self: Sized;

    /// Delete the key from the backing store and consume the type.
    async fn delete(self) -> Result<(), Self::Error>;
}

#[cfg(any(test, feature = "mock_secure_keys"))]
mod mock {
    use aes_gcm::Aes256Gcm;
    use p256::ecdsa::SigningKey;

    use super::{EphemeralEcdsaKey, SecureEcdsaKey, SecureEncryptionKey};

    impl EphemeralEcdsaKey for SigningKey {}
    impl SecureEcdsaKey for SigningKey {}

    impl SecureEncryptionKey for Aes256Gcm {}
}
