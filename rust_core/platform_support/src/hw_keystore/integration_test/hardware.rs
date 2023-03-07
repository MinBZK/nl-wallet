// this is the starting point for integration test performed from Android / iOS.
#[no_mangle]
fn hw_keystore_test_hardware_signature() -> bool {
    use crate::hw_keystore::{
        hardware::HardwareKeyStore, integration_test::sign_and_verify_signature,
    };

    let keystore = HardwareKeyStore::key_store();
    let payload = b"This is a message that will be signed.";
    let identifier = "key";

    sign_and_verify_signature(&keystore, payload, identifier)
}
