// Prevent dead code warnings since the lower 4 modules are not exposed in the `api` module yet.
// TODO: remove this when these modules are used.
#![allow(dead_code)]

mod jwt;
mod serialization;
mod utils;

mod wallet;
mod wp;

use once_cell::sync::Lazy;
use p256::ecdsa::SigningKey;
use rand::rngs::OsRng;
use std::sync::Mutex;

use crate::{
    wallet::{HWBoundSigningKey, Wallet},
    wp::AccountServer,
};

pub use crate::wallet::pin::{validate_pin, PinError};

// TODO remove this when an actual hardware-backed implementation exists
impl HWBoundSigningKey for SigningKey {
    fn verifying_key(&self) -> &p256::ecdsa::VerifyingKey {
        SigningKey::verifying_key(self)
    }
}

pub static WALLET: Lazy<Mutex<Wallet<AccountServer, SigningKey>>> = Lazy::new(|| {
    let account_server = AccountServer::new_stub(); // TODO
    let pubkey = account_server.pubkey.clone();

    Mutex::new(Wallet::new(
        account_server,
        pubkey,
        SigningKey::random(&mut OsRng),
    ))
});
