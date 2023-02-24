extern crate hw_keystore;

#[cfg(feature = "hardware")]
#[test]
#[should_panic]
fn test_hardware_panic_without_init() {
    use hw_keystore::hardware::HardwareKeyStore;

    _ = HardwareKeyStore::new();
}

#[cfg(all(feature = "software", feature = "integration-test"))]
#[test]
fn test_software_signature() {
    use hw_keystore::integration_test::sign_and_verify_signature;
    use hw_keystore::software::InMemoryKeyStore;

    let mut keystore = InMemoryKeyStore::new();
    let payload = b"This is a message that will be signed.";
    let identifier = "key";

    assert!(sign_and_verify_signature(
        &mut keystore,
        payload,
        identifier
    ));
}
