use wallet_common::keys::test;

use crate::hw_keystore::hardware::{HardwareEcdsaKey, HardwareEncryptionKey};

// this is the starting point for the ECDSA key integration test performed from Android / iOS.
#[no_mangle]
fn hw_keystore_test_hardware_signature() {
    let payload = b"This is a message that will be signed.";
    let identifier = "hw_keystore_test_hardware_signature";

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(test::sign_and_verify_signature::<HardwareEcdsaKey>(payload, identifier))
}

// this is the starting point for the encryption key integration test performed from Android / iOS.
#[no_mangle]
fn hw_keystore_test_hardware_encryption() {
    let payload = b"This is a message that will be encrypted.";
    let identifier = "hw_keystore_test_hardware_encryption";

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(test::encrypt_and_decrypt_message::<HardwareEncryptionKey>(
        payload, identifier,
    ))
}

#[cfg(target_os = "android")]
mod android {
    use jni::{objects::JClass, JNIEnv};

    #[rustfmt::skip]
    #[no_mangle]
    extern "C" fn Java_nl_rijksoverheid_edi_wallet_platform_1support_keystore_signing_SigningKeyBridgeInstrumentedTest_hw_1keystore_1test_1hardware_1signature(
        _env: JNIEnv,
        _: JClass,
    ) {
        super::hw_keystore_test_hardware_signature()
    }

    #[rustfmt::skip]
    #[no_mangle]
    extern "C" fn Java_nl_rijksoverheid_edi_wallet_platform_1support_keystore_encryption_EncryptionKeyBridgeInstrumentedTest_hw_1keystore_1test_1hardware_1encryption(
        _env: JNIEnv,
        _: JClass,
    ) {
        super::hw_keystore_test_hardware_encryption()
    }
}
