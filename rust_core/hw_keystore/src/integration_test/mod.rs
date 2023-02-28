#[cfg(feature = "hardware")]
pub mod hardware;

use p256::ecdsa::{
    signature::{Signer, Verifier},
    VerifyingKey,
};

use crate::{KeyStore, SigningKey};

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

    let public_key = key1.verifying_key().expect("Could not get public key");

    let signature = key2.try_sign(payload).expect("Could not sign payload");

    VerifyingKey::from(public_key)
        .verify(payload, &signature)
        .is_ok()
}
