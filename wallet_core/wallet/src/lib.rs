// Prevent dead code warnings since the lower 4 modules are not exposed publically yet.
// TODO: remove this when these modules are used.
#![allow(dead_code)]

mod account_server;
pub mod pin;
mod storage;
pub mod wallet;

use base64::{engine::general_purpose::STANDARD, Engine};
use platform_support::preferred;
use wallet_common::account::jwt::EcdsaDecodingKey;

use crate::account_server::remote::RemoteAccountServer;

// TODO: make these configurable
const ACCOUNT_SERVER_URL: &str = "https://localhost:3000";
const ACCOUNT_SERVER_PUB: &str = ""; // insert WP public key here

pub type Wallet = wallet::Wallet<RemoteAccountServer, preferred::PlatformEcdsaKey>;

pub fn init_wallet() -> Wallet {
    let account_server = RemoteAccountServer::new(ACCOUNT_SERVER_URL.to_string());
    let pubkey = EcdsaDecodingKey::from_sec1(&STANDARD.decode(ACCOUNT_SERVER_PUB).unwrap());

    Wallet::new(account_server, pubkey)
}
