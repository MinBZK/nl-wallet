use once_cell::sync::Lazy;
use platform_support::hw_keystore::PreferredPlatformSigningKey;
use std::sync::Mutex;
use wallet::{account::client::server::AccountServer, wallet::Wallet};

pub static WALLET: Lazy<Mutex<Wallet<AccountServer, PreferredPlatformSigningKey>>> = Lazy::new(|| {
    let account_server = AccountServer::new_stub(); // TODO
    let pubkey = account_server.pubkey.clone();

    Mutex::new(Wallet::new(account_server, pubkey))
});
