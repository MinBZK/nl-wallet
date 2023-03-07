#[cfg(feature = "hardware")]
pub mod hardware;

use p256::ecdsa::{
    signature::{Signer, Verifier},
    VerifyingKey,
};
use std::sync::{Arc, RwLock};

use crate::hw_keystore::{KeyStore, SigningKey};

// This utility function is used both by the Rust integration test for the "software" feature
// and by integration test performed from Android / iOS for the "hardware" feature.
pub fn sign_and_verify_signature(
    keystore: &Arc<RwLock<impl KeyStore>>,
    payload: &[u8],
    key_identifier: &str,
) -> bool {
    // Create first key for key identifier in separate context
    let key1 = {
        let mut keystore = keystore
            .write()
            .expect("Could not get write lock on KeyStore");
        keystore
            .create_key(key_identifier)
            .expect("Could not create key")
            .clone()
    };

    // Get the key with the same identifier again, should use the same private key
    let keystore = keystore
        .read()
        .expect("Could not get read lock on KeyStore");
    let key2 = keystore.get_key(key_identifier).expect("Could not get key");

    // Get the public key from the first key
    let public_key = key1.verifying_key().expect("Could not get public key");

    // Apply a signature to the payload using the second key
    let signature = key2.try_sign(payload).expect("Could not sign payload");

    // Then verify the signature, which should work if they indeed use the same private key
    VerifyingKey::from(*public_key)
        .verify(payload, &signature)
        .is_ok()
}
