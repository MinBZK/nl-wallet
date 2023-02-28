use std::sync::Mutex;

use crate::{
    wallet::{pin::validate_pin, HWBoundSigningKey, Wallet},
    wp::AccountServer,
};
use once_cell::sync::Lazy;
use p256::ecdsa::SigningKey;
use rand::rngs::OsRng;

// TODO remove this when an actual hardware-backed implementation exists
impl HWBoundSigningKey for SigningKey {
    fn verifying_key(&self) -> p256::ecdsa::VerifyingKey {
        SigningKey::verifying_key(&self)
    }
}

const WALLET: Lazy<Mutex<Wallet<AccountServer, SigningKey>>> = Lazy::new(|| {
    let account_server = AccountServer::new_stub(); // TODO
    let pubkey = account_server.pubkey.clone();

    Mutex::new(Wallet::new(
        account_server,
        pubkey,
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
