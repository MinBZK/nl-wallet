mod account_server;
pub mod pin;
mod storage;
pub mod wallet;

use platform_support::preferred;
use storage::DatabaseStorage;
use wallet_provider::account_server::AccountServer;

pub type Wallet = wallet::Wallet<AccountServer, DatabaseStorage, preferred::PlatformEcdsaKey>;

pub fn init_wallet() -> Wallet {
    let account_server = AccountServer::new_stub(); // TODO
    let storage = DatabaseStorage::default();
    let pubkey = account_server.pubkey.clone();

    Wallet::new(account_server, pubkey, storage)
}
