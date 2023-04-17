// Prevent dead code warnings since the lower 4 modules are not exposed publically yet.
// TODO: remove this when these modules are used.
#![allow(dead_code)]

mod account_server;
pub mod pin;
pub mod wallet;

use platform_support::preferred;
use wallet_provider::account_server::AccountServer;

pub type Wallet = wallet::Wallet<AccountServer, preferred::PlatformEcdsaKey>;

pub fn init_wallet() -> Wallet {
    let account_server = AccountServer::new_stub(); // TODO
    let pubkey = account_server.pubkey.clone();

    Wallet::new(account_server, pubkey)
}
