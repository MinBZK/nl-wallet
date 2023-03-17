extern crate platform_support;

#[cfg(all(feature = "software", feature = "integration-test"))]
#[test]
fn test_software_signature() {
    use platform_support::hw_keystore::{
        integration_test::sign_and_verify_signature, software::SoftwareSigningKey,
    };

    let payload = b"This is a message that will be signed.";
    let identifier = "key";

    assert!(sign_and_verify_signature::<SoftwareSigningKey>(
        payload, identifier
    ));
}
