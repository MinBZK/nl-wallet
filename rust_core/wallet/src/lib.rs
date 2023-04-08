// Prevent dead code warnings since the lower 4 modules are not exposed publically yet.
// TODO: remove this when these modules are used.
#![allow(dead_code)]

mod account;
pub mod pin;
pub mod wallet;

use account::server::AccountServer;
use platform_support::hw_keystore::PreferredPlatformSigningKey;

pub type Wallet = wallet::Wallet<AccountServer, PreferredPlatformSigningKey>;

pub fn init_wallet() -> Wallet {
    let account_server = AccountServer::new_stub(); // TODO
    let pubkey = account_server.pubkey.clone();

    Wallet::new(account_server, pubkey)
}
