extern crate hw_keystore;

#[cfg(feature = "hardware")]
#[test]
#[should_panic]
fn test_hardware_panic_without_init() {
    use hw_keystore::hardware::HardwareKeyStore;

    _ = HardwareKeyStore::new();
}

#[cfg(feature = "software")]
fn sign_and_verify_signature(
    keystore: &mut impl hw_keystore::KeyStore,
    payload: &[u8],
    key_identifier: &str,
) -> bool {
    use hw_keystore::AsymmetricKey;
    use p256::{
        ecdsa::{signature::Verifier, Signature, VerifyingKey},
        pkcs8::DecodePublicKey,
        PublicKey,
    };

    let key1 = keystore.get_or_create_key(key_identifier);
    let key2 = keystore.get_or_create_key(key_identifier);

    let public_key_bytes = key1.public_key();
    let public_key = PublicKey::from_public_key_der(&public_key_bytes).expect("Invalid public key");

    let signature_bytes = key2.sign(payload);
    let signature = Signature::from_scalars(
        <[u8; 32]>::try_from(&signature_bytes[..32]).unwrap(),
        <[u8; 32]>::try_from(&signature_bytes[32..]).unwrap(),
    )
    .expect("Invalid signature");

    VerifyingKey::from(public_key)
        .verify(payload, &signature)
        .is_ok()
}

#[cfg(feature = "software")]
#[test]
fn test_software_signature() {
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
