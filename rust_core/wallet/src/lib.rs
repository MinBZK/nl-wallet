// Prevent dead code warnings since the lower 4 modules are not exposed publically yet.
// TODO: remove this when these modules are used.
#![allow(dead_code)]

mod account;
pub mod pin;
mod utils;
mod wallet;

use once_cell::sync::Lazy;
use platform_support::hw_keystore::{PlatformSigningKey, PreferredPlatformSigningKey};
use std::sync::Mutex;

use crate::{account::client::server::AccountServer, wallet::Wallet};

const WALLET_KEY_ID: &str = "wallet";

pub static WALLET: Lazy<Mutex<Wallet<AccountServer, PreferredPlatformSigningKey>>> = Lazy::new(|| {
    let hw_privkey =
        PreferredPlatformSigningKey::signing_key(WALLET_KEY_ID).expect("Could not fetch or generate wallet key");

    let account_server = AccountServer::new_stub(); // TODO
    let pubkey = account_server.pubkey.clone();

    Mutex::new(Wallet::new(account_server, pubkey, hw_privkey))
});
