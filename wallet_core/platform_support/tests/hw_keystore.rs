#[cfg(feature = "software")]
#[test]
fn test_software_signature() {
    use platform_support::integration_test::hw_keystore::sign_and_verify_signature;
    use wallet_common::account::software_keys::SoftwareEcdsaKey;

    let payload = b"This is a message that will be signed.";
    let identifier = "key";

    assert!(sign_and_verify_signature::<SoftwareEcdsaKey>(payload, identifier));
}

#[cfg(feature = "software")]
#[test]
fn test_software_encryption() {
    use platform_support::integration_test::hw_keystore::encrypt_and_decrypt_message;
    use wallet_common::account::software_keys::SoftwareEncryptionKey;

    let payload = b"This message will be encrypted.";
    let identifier = "key";

    assert!(encrypt_and_decrypt_message::<SoftwareEncryptionKey>(
        payload, identifier
    ));
}
