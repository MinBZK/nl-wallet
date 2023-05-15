pub mod account_server;
pub mod pin;
pub mod storage;
pub mod wallet;

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};

use platform_support::preferred;
use wallet_common::account::jwt::EcdsaDecodingKey;

use crate::{account_server::remote::RemoteAccountServerClient, storage::DatabaseStorage};

// TODO: make these configurable
const ACCOUNT_SERVER_URL: &str = "https://localhost:3000";
const ACCOUNT_SERVER_PUB: &str = ""; // insert WP public key here

pub type Wallet = wallet::Wallet<RemoteAccountServerClient, DatabaseStorage, preferred::PlatformEcdsaKey>;

pub async fn init_wallet() -> Result<Wallet> {
    let account_server = RemoteAccountServerClient::new(ACCOUNT_SERVER_URL.to_string());
    let storage = DatabaseStorage::default();
    let pubkey = EcdsaDecodingKey::from_sec1(&STANDARD.decode(ACCOUNT_SERVER_PUB).unwrap());

    Wallet::new(account_server, pubkey, storage).await
}
