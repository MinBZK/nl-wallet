use p256::ecdsa::signature::Verifier;

use crate::hw_keystore::{PlatformEcdsaKey, PlatformEncryptionKey};

// This utility function is used both by the Rust integration test for the "software" feature
// and by integration test performed from Android / iOS for the "hardware" feature.
// This would normally fall under dev-dependencies, however we need it in the main binary
// for the "hardware" integration test.
pub fn sign_and_verify_signature<K: PlatformEcdsaKey>(payload: &[u8], key_identifier: &str) -> bool {
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

#[cfg(feature = "hardware-integration-test")]
mod hardware {
    use jni::{objects::JClass, JNIEnv};

    use super::{encrypt_and_decrypt_message, sign_and_verify_signature};
    use crate::hw_keystore::hardware::{HardwareEcdsaKey, HardwareEncryptionKey};

    // this is the starting point for the ECDSA key integration test performed from Android / iOS.
    #[no_mangle]
    fn hw_keystore_test_hardware_signature() -> bool {
        let payload = b"This is a message that will be signed.";
        let identifier = "key";

        sign_and_verify_signature::<HardwareEcdsaKey>(payload, identifier)
    }

    #[no_mangle]
    extern "C" fn Java_nl_rijksoverheid_edi_wallet_platform_1support_keystore_HWKeyStoreBridgeInstrumentedTest_hw_1keystore_1test_1hardware_1signature(
        _env: JNIEnv,
        _: JClass,
    ) -> bool {
        hw_keystore_test_hardware_signature()
    }

    // this is the starting point for the encryption key integration test performed from Android / iOS.
    #[no_mangle]
    fn hw_keystore_test_hardware_encryption() -> bool {
        let payload = b"This is a message that will be encrypted.";
        let identifier = "key";

        encrypt_and_decrypt_message::<HardwareEncryptionKey>(payload, identifier)
    }

    #[no_mangle]
    extern "C" fn Java_nl_rijksoverheid_edi_wallet_platform_1support_keystore_HWKeyStoreBridgeInstrumentedTest_hw_1keystore_1test_1hardware_1encryption(
        _env: JNIEnv,
        _: JClass,
    ) -> bool {
        hw_keystore_test_hardware_encryption()
    }
}
