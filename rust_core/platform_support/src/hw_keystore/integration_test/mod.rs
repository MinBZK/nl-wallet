#[cfg(all(feature = "hardware-integration-test"))]
pub mod hardware;

use p256::ecdsa::signature::Verifier;

use crate::hw_keystore::PlatformSigningKey;

// This utility function is used both by the Rust integration test for the "software" feature
// and by integration test performed from Android / iOS for the "hardware" feature.
// This would normally fall under dev-dependencies, however we need it in the main binary
// for the "hardware" integration test.
pub fn sign_and_verify_signature<K: PlatformSigningKey>(payload: &[u8], key_identifier: &str) -> bool {
    // Create a signing key for the identifier
    let key1 = K::signing_key(key_identifier).expect("Could not create signing key");
    // Create another signing key with the same identifier, should use the same private key
    let key2 = K::signing_key(key_identifier).expect("Could not create signing key");

    // Get the public key from the first key
    let public_key = PlatformSigningKey::verifying_key(&key1).expect("Could not get public key");

    // Apply a signature to the payload using the second key
    let signature = key2.try_sign(payload).expect("Could not sign payload");

    // Then verify the signature, which should work if they indeed use the same private key
    public_key.verify(payload, &signature).is_ok()
}
