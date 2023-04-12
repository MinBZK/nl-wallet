#[cfg(feature = "hardware-integration-test")]
pub mod hardware;

use p256::ecdsa::signature::Verifier;

use crate::hw_keystore::{PlatformEncryptionKey, PlatformSigningKey};

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
    let public_key = key1.verifying_key().expect("Could not get public key");

    // Apply a signature to the payload using the second key
    let signature = key2.try_sign(payload).expect("Could not sign payload");

    // Then verify the signature, which should work if they indeed use the same private key
    public_key.verify(payload, &signature).is_ok()
}

pub fn encrypt_and_decrypt_message<K: PlatformEncryptionKey>(payload: &[u8], key_identifier: &str) -> bool {
    // Create an encryption key for the identifier
    let encryption_key = K::encryption_key(key_identifier).expect("Could not create encryption key");

    // Encrypt the payload with the key
    let encrypted_payload = encryption_key.encrypt(payload).expect("Could not encrypt message");

    // Decrypt the encrypted message with the key
    let decrypted_payload = encryption_key
        .decrypt(&encrypted_payload)
        .expect("Could not decrypt message");

    // Verify payload is indeed encrypted and decrypted payload matches the original
    payload != encrypted_payload && payload == decrypted_payload
}
