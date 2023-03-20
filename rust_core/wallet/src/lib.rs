// Prevent dead code warnings since the lower 4 modules are not exposed in the `api` module yet.
// TODO: remove this when these modules are used.
#![allow(dead_code)]

mod jwt;
mod serialization;
mod utils;

mod wallet;
mod wp;

use once_cell::sync::Lazy;
use platform_support::hw_keystore::PlatformSigningKey;
use std::sync::Mutex;

use crate::{wallet::Wallet, wp::AccountServer};

pub use crate::wallet::pin::{validate_pin, PinError};

#[cfg(any(target_os = "android", target_os = "ios"))]
type PlatformSigningKeyImpl = platform_support::hw_keystore::hardware::HardwareSigningKey;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
type PlatformSigningKeyImpl = platform_support::hw_keystore::software::SoftwareSigningKey;

const WALLET_KEY_ID: &str = "wallet";

pub static WALLET: Lazy<Mutex<Wallet<AccountServer, PlatformSigningKeyImpl>>> = Lazy::new(|| {
    let hw_privkey = PlatformSigningKeyImpl::signing_key(WALLET_KEY_ID)
        .expect("Could not fetch or generate wallet key");

    let account_server = AccountServer::new_stub(); // TODO
    let pubkey = account_server.pubkey.clone();

    Mutex::new(Wallet::new(account_server, pubkey, hw_privkey))
});
