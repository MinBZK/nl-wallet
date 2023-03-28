use jni::{objects::JClass, JNIEnv};

// this is the starting point for integration test performed from Android / iOS.
#[no_mangle]
fn hw_keystore_test_hardware_signature() -> bool {
    use crate::hw_keystore::{hardware::HardwareSigningKey, integration_test::sign_and_verify_signature};

    let payload = b"This is a message that will be signed.";
    let identifier = "key";

    sign_and_verify_signature::<HardwareSigningKey>(payload, identifier)
}

#[no_mangle]
extern "C" fn Java_nl_rijksoverheid_edi_wallet_platform_1support_hw_1keystore_HWKeyStoreInstrumentedTest_hw_1keystore_1test_1hardware_1signature(
    _env: JNIEnv,
    _: JClass,
) -> bool {
    hw_keystore_test_hardware_signature()
}
