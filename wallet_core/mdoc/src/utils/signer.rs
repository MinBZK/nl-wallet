use std::{collections::HashMap, error::Error, sync::Mutex};

use once_cell::sync::Lazy;
use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait ConstructableWithIdentifier {
    fn new(identifier: &str) -> Self
    where
        Self: Sized;

    fn identifier(&self) -> &str;
}

pub trait EcdsaKey: Signer<Signature> {
    type Error: Error + Send + Sync + 'static;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error>;
}

pub trait SecureEcdsaKey: EcdsaKey {}

/// Contract for ECDSA private keys suitable for use in the wallet, e.g. as the authentication key for the WP.
/// Should be sufficiently secured e.g. through Android's TEE/StrongBox or Apple's SE.
/// Handles to private keys are requested through [`ConstructableWithIdentifier::new()`].
pub trait MdocEcdsaKey: ConstructableWithIdentifier + SecureEcdsaKey + Serialize + DeserializeOwned {
    const KEY_TYPE: &'static str;

    // from ConstructableWithIdentifier: new(), identifier()
    // from SecureSigningKey: verifying_key(), try_sign() and sign() methods
}

//// Software ECDSA key

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareEcdsaKey {
    identifier: String,
}

// static for storing identifier -> signing key mapping, will only every grow
static SIGNING_KEYS: Lazy<Mutex<HashMap<String, SigningKey>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// SigningKey from p256::ecdsa almost conforms to the EcdsaKey trait,
// so we can forward the try_sign method and verifying_key methods.
impl Signer<Signature> for SoftwareEcdsaKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, p256::ecdsa::Error> {
        SIGNING_KEYS
            .lock()
            .expect("Could not get lock on SIGNING_KEYS")
            .get(&self.identifier)
            .unwrap()
            .try_sign(msg)
    }
}
impl EcdsaKey for SoftwareEcdsaKey {
    type Error = p256::ecdsa::Error;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        Ok(SIGNING_KEYS
            .lock()
            .expect("Could not get lock on SIGNING_KEYS")
            .get(&self.identifier)
            .unwrap()
            .verifying_key())
    }
}
impl ConstructableWithIdentifier for SoftwareEcdsaKey {
    fn new(identifier: &str) -> Self
    where
        Self: Sized,
    {
        // obtain lock on SIGNING_KEYS static hashmap
        let mut signing_keys = SIGNING_KEYS.lock().expect("Could not get lock on SIGNING_KEYS");
        // insert new random signing key, if the key is not present
        if !signing_keys.contains_key(identifier) {
            signing_keys.insert(identifier.to_string(), SigningKey::random(&mut OsRng));
        }

        SoftwareEcdsaKey {
            identifier: identifier.to_string(),
        }
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use p256::ecdsa::SigningKey;

    use super::{EcdsaKey, MdocEcdsaKey, SecureEcdsaKey, SoftwareEcdsaKey, SIGNING_KEYS};

    impl EcdsaKey for SigningKey {
        type Error = p256::ecdsa::Error;

        fn verifying_key(&self) -> Result<p256::ecdsa::VerifyingKey, Self::Error> {
            Ok(self.verifying_key())
        }
    }
    impl SecureEcdsaKey for SigningKey {}

    impl SecureEcdsaKey for SoftwareEcdsaKey {}
    impl MdocEcdsaKey for SoftwareEcdsaKey {
        const KEY_TYPE: &'static str = "software";
    }
    /// Insert a given existing key in the map of [`SoftwareEcdsaKey`]s, for use in testing
    /// (e.g. with the keys in ISO 23220).
    impl SoftwareEcdsaKey {
        pub fn insert(identifier: &str, key: SigningKey) {
            SIGNING_KEYS
                .lock()
                .expect("Could not get lock on SIGNING_KEYS")
                .insert(identifier.to_string(), key);
        }
    }
}

#[cfg(test)]
mod tests {
    use ecdsa::signature::{Signer, Verifier};

    use super::{ConstructableWithIdentifier, EcdsaKey, SoftwareEcdsaKey};

    #[test]
    fn software_key_works() {
        let msg = b"Hello, world!";
        let privkey = SoftwareEcdsaKey::new("mykey");
        let signature = Signer::sign(&privkey, msg);
        EcdsaKey::verifying_key(&privkey)
            .unwrap()
            .verify(msg, &signature)
            .unwrap();
    }
}
