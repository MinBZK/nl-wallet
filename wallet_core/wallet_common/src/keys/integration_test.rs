use p256::ecdsa::signature::Verifier;

use super::{ConstructableWithIdentifier, SecureEcdsaKey, SecureEncryptionKey};

// This utility function is used both by the Rust integration test for the "software-keys" feature
// and by integration test performed in platform_support from Android / iOS.
// This would normally fall under dev-dependencies, however we need it in the main binary
// for the Android / iOS integration test.
pub fn sign_and_verify_signature<K: ConstructableWithIdentifier + SecureEcdsaKey>(
    payload: &[u8],
    key_identifier: &str,
) -> bool {
    // Create a signing key for the identifier
    let key1 = K::new(key_identifier);
    // Create another signing key with the same identifier, should use the same private key
    let key2 = K::new(key_identifier);

    // Check if identifiers match
    assert_eq!(key1.identifier(), key_identifier);
    assert_eq!(key2.identifier(), key_identifier);

    // Get the public key from the first key
    let public_key = key1.verifying_key().expect("Could not get public key");

    // Apply a signature to the payload using the second key
    let signature = key2.try_sign(payload).expect("Could not sign payload");

    // Then verify the signature, which should work if they indeed use the same private key
    public_key.verify(payload, &signature).is_ok()
}

pub fn encrypt_and_decrypt_message<K: SecureEncryptionKey>(payload: &[u8], key_identifier: &str) -> bool {
    // Create an encryption key for the identifier
    let encryption_key1 = K::new(key_identifier);
    // Create another encryption key with the same identifier, should use the same key
    let encryption_key2 = K::new(key_identifier);

    // Check if identifiers match
    assert_eq!(encryption_key1.identifier(), key_identifier);
    assert_eq!(encryption_key2.identifier(), key_identifier);

    // Encrypt the payload with the first key
    let encrypted_payload = encryption_key1.encrypt(payload).expect("Could not encrypt message");

    // Decrypt the encrypted message with the second key
    let decrypted_payload = encryption_key2
        .decrypt(&encrypted_payload)
        .expect("Could not decrypt message");

    // Verify payload is indeed encrypted and decrypted payload matches the original
    payload != encrypted_payload && payload == decrypted_payload
}
