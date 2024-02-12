use futures::future::try_join;
use p256::ecdsa::signature::Verifier;

use super::{ConstructibleWithIdentifier, DeletableWithIdentifier, SecureEcdsaKey, SecureEncryptionKey};

// This utility function is used both by the Rust unit tests for the "software_keys" feature
// and by integration test performed in platform_support from Android / iOS.
pub async fn sign_and_verify_signature<K: ConstructibleWithIdentifier + DeletableWithIdentifier + SecureEcdsaKey>(
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
    let public_key = key1.verifying_key().await.expect("Could not get public key");

    // Apply a signature to the payload using the second key
    let signature = key2.try_sign(payload).await.expect("Could not sign payload");

    // Delete both signing keys, as well as the actual private key
    try_join(key1.delete(), key2.delete())
        .await
        .expect("Could not delete private key");

    // Creating a new signing key with a new private key should result in a different public key
    let key3 = K::new(key_identifier);
    let new_public_key = key3.verifying_key().await.expect("Could not get public key");
    assert_ne!(public_key, new_public_key);

    // Then verify the signature, which should work if they indeed use the same private key
    public_key.verify(payload, &signature).is_ok()
}

pub async fn encrypt_and_decrypt_message<
    K: ConstructibleWithIdentifier + DeletableWithIdentifier + SecureEncryptionKey,
>(
    payload: &[u8],
    key_identifier: &str,
) -> bool {
    // Create an encryption key for the identifier
    let encryption_key1 = K::new(key_identifier);
    // Create another encryption key with the same identifier, should use the same key
    let encryption_key2 = K::new(key_identifier);

    // Check if identifiers match
    assert_eq!(encryption_key1.identifier(), key_identifier);
    assert_eq!(encryption_key2.identifier(), key_identifier);

    // Encrypt the payload with the first key
    let encrypted_payload = encryption_key1
        .encrypt(payload)
        .await
        .expect("Could not encrypt message");

    // Decrypt the encrypted message with the second key
    let decrypted_payload = encryption_key2
        .decrypt(&encrypted_payload)
        .await
        .expect("Could not decrypt message");

    // Delete both references to the encryption key, as well as the actual key
    try_join(encryption_key1.delete(), encryption_key2.delete())
        .await
        .expect("Could not delete encryption key");

    // Creating a new encryption key with the same identifier should use a new key
    let encryption_key3 = K::new(key_identifier);
    // Decrypting the payload encrypted with the previous key should not work
    encryption_key3
        .decrypt(&encrypted_payload)
        .await
        .expect_err("Decrypting the payload with a new key should result in an error");

    // Verify payload is indeed encrypted and decrypted payload matches the original
    payload != encrypted_payload && payload == decrypted_payload
}
