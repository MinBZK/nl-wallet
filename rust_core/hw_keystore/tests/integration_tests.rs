extern crate hw_keystore;

#[cfg(feature = "software")]
#[test]
fn test_software_signature() {
    use hw_keystore::software::InMemoryKeyStore;
    use hw_keystore::{AsymmetricKey, KeyStore};
    use p256::{
        ecdsa::{signature::Verifier, Signature, VerifyingKey},
        pkcs8::DecodePublicKey,
        PublicKey,
    };
    use std::rc::Rc;

    let identifier = "key";
    let message = b"This is a message that will be signed.";

    let mut keystore = InMemoryKeyStore::default();
    let key = keystore.get_or_create_key(identifier);
    let key2 = keystore.get_or_create_key(identifier);

    assert!(Rc::ptr_eq(&key, &key2));

    let public_key_bytes = key.public_key();
    let public_key = PublicKey::from_public_key_der(&public_key_bytes).unwrap();

    let signature_bytes = key.sign(message);
    let signature = Signature::from_scalars(
        <[u8; 32]>::try_from(&signature_bytes[..32]).unwrap(),
        <[u8; 32]>::try_from(&signature_bytes[32..]).unwrap(),
    )
    .unwrap();
    assert!(VerifyingKey::from(public_key)
        .verify(message, &signature)
        .is_ok());
}
