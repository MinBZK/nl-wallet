use p256::ecdsa::signature::Verifier;

use super::{SecureEcdsaKey, SecureEncryptionKey, StoredByIdentifier};

// This utility function is used both by the Rust unit tests for the "software_keys" feature
// and by integration test performed in platform_support from Android / iOS.
pub async fn sign_and_verify_signature<K: StoredByIdentifier + SecureEcdsaKey>(
    payload: &[u8],
    key_identifier: &str,
) -> bool {
    // Create a unique signing key for the identifier.
    let key1 = K::new_unique(key_identifier).expect("key is not unique for identifier");

    // Creating another signing key for the identifier should return `None`.
    assert!(K::new_unique(key_identifier).is_none());

    // Get the public key from the key.
    let public_key1 = key1.verifying_key().await.expect("Could not get public key");

    // Apply a signature to the payload using the key.
    let signature = key1.try_sign(payload).await.expect("Could not sign payload");

    // Delete the key, as well as the private key in the backing store.
    key1.delete().await.expect("Could not delete private key");

    // Creating a second signing key with a new private key should result in a different public key.
    let key2 = K::new_unique(key_identifier).expect("key is not unique for identifier");
    let public_key2 = key2.verifying_key().await.expect("Could not get public key");
    assert_ne!(public_key1, public_key2);

    // Finally verify the signature against the public key.
    public_key1.verify(payload, &signature).is_ok()
}

pub async fn encrypt_and_decrypt_message<K: StoredByIdentifier + SecureEncryptionKey>(
    payload: &[u8],
    key_identifier: &str,
) -> bool {
    // Create a unique encryption key for the identifier.
    let key1 = K::new_unique(key_identifier).expect("key is not unique for identifier");

    // Encrypt the payload with the key.
    let encrypted_payload = key1.encrypt(payload).await.expect("Could not encrypt message");

    // Decrypt the encrypted message with the key.
    let decrypted_payload = key1
        .decrypt(&encrypted_payload)
        .await
        .expect("Could not decrypt message");

    // Delete the key, as well as the encryption key in the backing store.
    key1.delete().await.expect("Could not delete encryption key");

    // Creating a second encryption key with the same identifier should result in a new key.
    let key2 = K::new_unique(key_identifier).expect("key is not unique for identifier");
    // Decrypting the payload encrypted with the previous key should not work.
    key2.decrypt(&encrypted_payload)
        .await
        .expect_err("Decrypting the payload with a new key should result in an error");

    // Verify that the payload is indeed encrypted and
    // that the decrypted payload matches the original.
    payload != encrypted_payload && payload == decrypted_payload
}
