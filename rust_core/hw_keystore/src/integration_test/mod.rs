#[cfg(feature = "hardware")]
pub mod hardware;

use crate::{AsymmetricKey, KeyStore};
use p256::{
    ecdsa::{signature::Verifier, Signature, VerifyingKey},
    pkcs8::DecodePublicKey,
    PublicKey,
};

pub fn sign_and_verify_signature(
    keystore: &mut impl KeyStore,
    payload: &[u8],
    key_identifier: &str,
) -> bool {
    let key1 = keystore
        .get_or_create_key(key_identifier)
        .expect("Could not get key");
    let key2 = keystore
        .get_or_create_key(key_identifier)
        .expect("Could not get key");

    let public_key_bytes = key1.public_key().expect("Could not get public key");
    let public_key = PublicKey::from_public_key_der(&public_key_bytes).expect("Invalid public key");

    let signature_bytes = key2.sign(payload).expect("Could not sign payload");
    let signature = Signature::from_der(&signature_bytes).expect("Invalid signature");

    VerifyingKey::from(public_key)
        .verify(payload, &signature)
        .is_ok()
}
