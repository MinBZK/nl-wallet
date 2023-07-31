#[cfg(feature = "software-keys")]
#[test]
fn test_software_signature() {
    use wallet_common::keys::integration_test::sign_and_verify_signature;
    use wallet_common::keys::software_keys::SoftwareEcdsaKey;

    let payload = b"This is a message that will be signed.";
    let identifier = "key";

    assert!(sign_and_verify_signature::<SoftwareEcdsaKey>(payload, identifier));
}

#[cfg(feature = "software-keys")]
#[test]
fn test_software_encryption() {
    use wallet_common::keys::{integration_test::encrypt_and_decrypt_message, software_keys::SoftwareEncryptionKey};

    let payload = b"This message will be encrypted.";
    let identifier = "key";

    assert!(encrypt_and_decrypt_message::<SoftwareEncryptionKey>(
        payload, identifier
    ));
}
