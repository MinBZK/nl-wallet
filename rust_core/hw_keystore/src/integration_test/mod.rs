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
