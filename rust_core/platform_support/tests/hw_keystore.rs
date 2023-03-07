extern crate platform_support;

#[cfg(feature = "hardware")]
#[test]
#[should_panic]
fn test_hardware_panic_without_init() {
    use platform_support::hw_keystore::hardware::HardwareKeyStore;

    _ = HardwareKeyStore::key_store();
}

#[cfg(all(feature = "software", feature = "integration-test"))]
#[test]
fn test_software_signature() {
    use platform_support::hw_keystore::{
        integration_test::sign_and_verify_signature, software::InMemoryKeyStore,
    };

    let keystore = InMemoryKeyStore::key_store();
    let payload = b"This is a message that will be signed.";
    let identifier = "key";

    assert!(sign_and_verify_signature(&keystore, payload, identifier));
}
