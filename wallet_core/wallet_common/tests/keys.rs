#[cfg(feature = "software-keys")]
#[tokio::test]
async fn test_software_signature() {
    use wallet_common::keys::{integration_test::sign_and_verify_signature, software::SoftwareEcdsaKey};

    let payload = b"This is a message that will be signed.";
    let identifier = "key";

    assert!(sign_and_verify_signature::<SoftwareEcdsaKey>(payload, identifier).await);
}

#[cfg(feature = "software-keys")]
#[tokio::test]
async fn test_software_encryption() {
    use wallet_common::keys::{integration_test::encrypt_and_decrypt_message, software::SoftwareEncryptionKey};

    let payload = b"This message will be encrypted.";
    let identifier = "key";

    assert!(encrypt_and_decrypt_message::<SoftwareEncryptionKey>(payload, identifier).await);
}
