#[cfg(feature = "hardware")]
pub mod hardware;

use p256::ecdsa::{
    signature::{Signer, Verifier},
    VerifyingKey,
};

use crate::{KeyStore, SigningKey};

// This utility function is used both by the Rust integration test for the "software" feature
// and by integration test performed from Android / iOS for the "hardware" feature.
pub fn sign_and_verify_signature(
    keystore: &mut impl KeyStore,
    payload: &[u8],
    key_identifier: &str,
) -> bool {
    // Create two keys from the same key store, should use the same private key
    let key1 = keystore
        .get_or_create_key(key_identifier)
        .expect("Could not get key");
    let key2 = keystore
        .get_or_create_key(key_identifier)
        .expect("Could not get key");

    // Get the public key from the first key
    let public_key = key1.verifying_key().expect("Could not get public key");

    // Apply a signature to the payload using the second key
    let signature = key2.try_sign(payload).expect("Could not sign payload");

    // Then verify the signature, which should work if they indeed use the same private key
    VerifyingKey::from(public_key)
        .verify(payload, &signature)
        .is_ok()
}
