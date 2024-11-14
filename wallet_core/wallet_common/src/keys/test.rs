use std::mem;

use p256::ecdsa::signature::Verifier;

use super::SecureEcdsaKey;
use super::SecureEncryptionKey;
use super::StoredByIdentifier;

// This utility function is used both by the Rust unit tests for the "software_keys" feature
// and by integration test performed in platform_support from Android / iOS.
pub async fn sign_and_verify_signature<K: StoredByIdentifier + SecureEcdsaKey>(payload: &[u8], key_identifier: &str) {
    // Create a unique signing key for the identifier.
    let key1 = K::new_unique(key_identifier).expect("key is not unique for identifier");

    // Creating another signing key for the identifier should return `None`.
    assert!(
        K::new_unique(key_identifier).is_none(),
        "should not be able to make multiple instances for the same key identifier"
    );

    // Get the public key from the key.
    let public_key1 = key1.verifying_key().await.expect("could not get public key");

    // Drop the first key, then create another key using the same identifier.
    mem::drop(key1);
    let key2 = K::new_unique(key_identifier).expect("key is not unique for identifier");

    // Apply a signature to the payload using that key and verify it against the public key.
    let signature = key2.try_sign(payload).await.expect("could not sign payload");
    assert!(
        public_key1.verify(payload, &signature).is_ok(),
        "signature does not verify against public key"
    );

    // Delete the key, as well as the private key in the backing store.
    key2.delete().await.expect("could not delete private key");

    // Creating a third signing key with a new private key should result in a different public key.
    let key3 = K::new_unique(key_identifier).expect("key is not unique for identifier");
    let public_key3 = key3.verifying_key().await.expect("could not get public key");
    assert_ne!(
        public_key1, public_key3,
        "key should have been deleted, yet public keys match"
    );
}

pub async fn encrypt_and_decrypt_message<K: StoredByIdentifier + SecureEncryptionKey>(
    payload: &[u8],
    key_identifier: &str,
) {
    // Create a unique encryption key for the identifier.
    let key1 = K::new_unique(key_identifier).expect("key is not unique for identifier");

    // Creating another encryption key for the identifier should return `None`.
    assert!(
        K::new_unique(key_identifier).is_none(),
        "should not be able to make multiple instances for the same key identifier"
    );

    // Encrypt the payload with the key.
    let encrypted_payload = key1.encrypt(payload).await.expect("could not encrypt message");

    // Drop the first key, then create another key using the same identifier.
    mem::drop(key1);
    let key2 = K::new_unique(key_identifier).expect("key is not unique for identifier");

    // Decrypt the encrypted message with this key, verify that the payload is
    // indeed encrypted and that the decrypted payload matches the original.
    let decrypted_payload = key2
        .decrypt(&encrypted_payload)
        .await
        .expect("could not decrypt message");
    assert_ne!(payload, encrypted_payload, "payload was not actually encrypted");
    assert_eq!(payload, decrypted_payload, "decrypted payload does not match original");

    // Delete the key, as well as the encryption key in the backing store.
    key2.delete().await.expect("could not delete encryption key");

    // Creating a second encryption key with the same identifier should result in a new key.
    let key3 = K::new_unique(key_identifier).expect("key is not unique for identifier");
    // Decrypting the payload encrypted with the previous key should not work.
    key3.decrypt(&encrypted_payload)
        .await
        .expect_err("decrypting the payload with a new key should result in an error");
}
