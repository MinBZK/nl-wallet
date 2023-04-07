#[cfg(all(feature = "software", feature = "integration-test"))]
#[test]
fn test_software_signature() {
    use platform_support::hw_keystore::{integration_test::sign_and_verify_signature, software::SoftwareSigningKey};

    let payload = b"This is a message that will be signed.";
    let identifier = "key";

    assert!(sign_and_verify_signature::<SoftwareSigningKey>(payload, identifier));
}

#[cfg(all(feature = "software", feature = "integration-test"))]
#[test]
fn test_software_encryption() {
    use platform_support::hw_keystore::{integration_test::encrypt_and_decrypt_message, software::SoftwareEncryptionKey};

    let payload = b"This message will be encrypted.";
    let identifier = "key";

    assert!(encrypt_and_decrypt_message::<SoftwareEncryptionKey>(payload, identifier));
}
