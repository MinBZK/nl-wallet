use std::sync::Mutex;

use crate::{
    utils::random_bytes,
    wallet::{pin::validate_pin, HWBoundSigningKey, Wallet},
    wp::AccountServer,
};
use once_cell::sync::Lazy;
use p256::{ecdsa::SigningKey, pkcs8::EncodePrivateKey};
use rand::rngs::OsRng;

// TODO remove this when an actual hardware-backed implementation exists
impl HWBoundSigningKey for SigningKey {
    fn verifying_key(&self) -> p256::ecdsa::VerifyingKey {
        SigningKey::verifying_key(&self)
    }
}

const WALLET: Lazy<Mutex<Wallet<SigningKey>>> = Lazy::new(|| {
    let account_server_privkey = SigningKey::random(&mut OsRng);
    let account_server_pubkey = account_server_privkey
        .verifying_key()
        .to_encoded_point(false)
        .as_bytes()
        .to_vec();
    let account_server = AccountServer::new(
        account_server_privkey
            .to_pkcs8_der()
            .unwrap()
            .as_bytes()
            .to_vec(),
        random_bytes(32),
        "stub_account_server".into(),
    )
    .unwrap();

    Mutex::new(Wallet::new(
        account_server,
        account_server_pubkey,
        SigningKey::random(&mut OsRng),
    ))
});

pub fn is_valid_pin(pin: String) -> bool {
    validate_pin(&pin).is_ok()
}

pub fn register(pin: String) {
    // TODO return error instead of panicking
    WALLET
        .lock()
        .expect("wallet lock failed")
        .register(pin)
        .expect("register() failed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_valid_pin() {
        assert!(is_valid_pin("142032".to_owned()));
    }

    #[test]
    fn check_invalid_pin() {
        assert!(!is_valid_pin("sdfioj".to_owned()));
    }
}
